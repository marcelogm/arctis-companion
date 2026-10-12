use std::os::fd::{AsFd, BorrowedFd};

use crate::discovery::Dongle;
use crate::port::{OpenError, Opener, Port};

#[derive(Debug)]
pub struct OpenDongle {
    pub physical: String,
    pub command: Option<Port>,
    pub events: Port,
}

impl OpenDongle {
    pub fn open(opener: &mut impl Opener, dongle: &Dongle) -> Result<Self, OpenError> {
        let events = opener.open(&dongle.events)?;
        let command = opener.open(&dongle.command)?;
        Ok(Self {
            physical: dongle.physical.clone(),
            command: Some(command),
            events,
        })
    }

    #[must_use]
    pub fn fds(&self) -> Vec<(Source, BorrowedFd<'_>)> {
        let events = (Source::Events, self.events.as_fd());
        let command = self
            .command
            .as_ref()
            .map(|port| (Source::Command, port.as_fd()));
        [Some(events), command].into_iter().flatten().collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Mailbox,
    Events,
    Command,
}
