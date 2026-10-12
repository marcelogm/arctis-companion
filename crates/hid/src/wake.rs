use std::io;
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

use arctis_core::messages::device::Change;
use rustix::io::Errno;
use rustix::pipe::PipeFlags;

const DRAIN_CAPACITY: usize = 64;

enum Message {
    Change(Change),
    Quit,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Mail {
    pub changes: Vec<Change>,
    pub quit: bool,
}

#[derive(Clone)]
pub struct DeviceChanges {
    sender: Sender<Message>,
    bell: Arc<OwnedFd>,
}

impl DeviceChanges {
    #[must_use]
    pub fn send(&self, change: Change) -> bool {
        self.post(Message::Change(change))
    }

    pub fn quit(&self) {
        let _ = self.post(Message::Quit);
    }

    fn post(&self, message: Message) -> bool {
        let sent = self.sender.send(message).is_ok();
        ring(&self.bell);
        sent
    }
}

pub struct Mailbox {
    receiver: Receiver<Message>,
    doorbell: OwnedFd,
}

impl Mailbox {
    #[must_use]
    pub fn take(&self) -> Mail {
        silence(&self.doorbell);
        let mut mail = Mail::default();
        while !mail.quit {
            match self.receiver.try_recv() {
                Ok(Message::Change(change)) => mail.changes.push(change),
                Ok(Message::Quit) | Err(TryRecvError::Disconnected) => mail.quit = true,
                Err(TryRecvError::Empty) => break,
            }
        }
        mail
    }
}

impl AsFd for Mailbox {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.doorbell.as_fd()
    }
}

pub fn mailbox() -> io::Result<(DeviceChanges, Mailbox)> {
    let (doorbell, bell) = rustix::pipe::pipe_with(PipeFlags::CLOEXEC | PipeFlags::NONBLOCK)?;
    let (sender, receiver) = mpsc::channel();
    Ok((
        DeviceChanges {
            sender,
            bell: Arc::new(bell),
        },
        Mailbox { receiver, doorbell },
    ))
}

fn ring(bell: &OwnedFd) {
    let _ = rustix::io::write(bell, &[1]);
}

fn silence(doorbell: &OwnedFd) {
    let mut buffer = [0; DRAIN_CAPACITY];
    loop {
        match rustix::io::read(doorbell, &mut buffer) {
            Ok(length) if length > 0 => {}
            Err(Errno::INTR) => {}
            _ => return,
        }
    }
}
