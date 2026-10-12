use std::cell::Cell;
use std::io;
use std::rc::Rc;
use std::thread::{self, JoinHandle};

use arctis_core::messages::audio::{AudioCommand, AudioEvent};
use pipewire as pw;

use crate::worker::{self, Link, Message};

const THREAD_NAME: &str = "audio";

#[derive(Clone)]
pub struct AudioCommands(pw::channel::Sender<Message>);

impl AudioCommands {
    #[must_use]
    pub fn send(&self, command: AudioCommand) -> bool {
        self.0.send(Message::Command(command)).is_ok()
    }
}

pub struct AudioHandle {
    pub commands: AudioCommands,
    pub events: async_channel::Receiver<AudioEvent>,
    thread: Option<JoinHandle<()>>,
}

impl AudioHandle {
    pub fn spawn() -> io::Result<Self> {
        crate::init();
        let (commands, mailbox) = pw::channel::channel();
        let (publisher, events) = async_channel::unbounded();
        let myself = commands.clone();
        let thread = thread::Builder::new()
            .name(THREAD_NAME.to_owned())
            .spawn(move || {
                let link = Link {
                    events: publisher,
                    myself,
                    announced: Rc::new(Cell::new(None)),
                };
                worker::run(mailbox, &link);
            })?;
        Ok(Self {
            commands: AudioCommands(commands),
            events,
            thread: Some(thread),
        })
    }
}

impl Drop for AudioHandle {
    fn drop(&mut self) {
        let _ = self.commands.0.send(Message::Quit);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
