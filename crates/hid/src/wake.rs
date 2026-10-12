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
    wake_writer: Arc<OwnedFd>,
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
        wake(&self.wake_writer);
        sent
    }
}

pub struct Mailbox {
    receiver: Receiver<Message>,
    wake_reader: OwnedFd,
}

impl Mailbox {
    #[must_use]
    pub fn take(&self) -> Mail {
        drain_wakes(&self.wake_reader);
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
        self.wake_reader.as_fd()
    }
}

pub fn mailbox() -> io::Result<(DeviceChanges, Mailbox)> {
    let (wake_reader, wake_writer) =
        rustix::pipe::pipe_with(PipeFlags::CLOEXEC | PipeFlags::NONBLOCK)?;
    let (sender, receiver) = mpsc::channel();
    Ok((
        DeviceChanges {
            sender,
            wake_writer: Arc::new(wake_writer),
        },
        Mailbox {
            receiver,
            wake_reader,
        },
    ))
}

fn wake(wake_writer: &OwnedFd) {
    let _ = rustix::io::write(wake_writer, &[1]);
}

fn drain_wakes(wake_reader: &OwnedFd) {
    let mut buffer = [0; DRAIN_CAPACITY];
    loop {
        match rustix::io::read(wake_reader, &mut buffer) {
            Ok(length) if length > 0 => {}
            Err(Errno::INTR) => {}
            _ => return,
        }
    }
}
