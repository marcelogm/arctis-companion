use std::os::fd::OwnedFd;
use std::path::Path;

use arctis_core::protocol::Frame;
use arctis_core::session::Interface;
use arctis_core::testkit::{Emission, FakeHeadset, Flap, frame_from_report};
use arctis_core::time::Millis;
use arctis_hid::port::{OpenError, Port};
use rustix::net::Shutdown;

use crate::sockets::{pair, receive, send};
use crate::sysfs::{COMMAND_NODE, EVENTS_NODE, FakeSysfs, NodeSpec};

const FAILED_WRITE_FLAP: u64 = 60;

enum CommandNode {
    Closed,
    Open(OwnedFd),
    Missing {
        back_at: Option<Millis>,
        readable_at: Millis,
        stale_frame: bool,
    },
}

enum Stop {
    Emission(Millis),
    Deadline(Millis),
    Horizon,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Advance {
    Delivered,
    TimedOut,
}

pub struct Bench {
    pub now: Millis,
    pub horizon: Millis,
    pub reached: bool,
    pub headset: FakeHeadset,
    pub writes: Vec<(Millis, Frame)>,
    pub sysfs: FakeSysfs,
    inbox: Vec<Emission>,
    plugged: bool,
    denied: bool,
    command: CommandNode,
    events_peer: Option<OwnedFd>,
    severed: Vec<OwnedFd>,
}

impl Bench {
    pub fn new(headset: FakeHeadset) -> Self {
        Self {
            now: Millis(0),
            horizon: Millis(0),
            reached: false,
            headset,
            writes: Vec::new(),
            sysfs: FakeSysfs::new(),
            inbox: Vec::new(),
            plugged: false,
            denied: false,
            command: CommandNode::Closed,
            events_peer: None,
            severed: Vec::new(),
        }
    }

    pub fn plug(&mut self) {
        self.denied = false;
        if !self.plugged {
            self.sysfs.add_nova_5();
            self.plugged = true;
            self.command = CommandNode::Closed;
        }
    }

    pub fn deny(&mut self) {
        self.plug();
        self.denied = true;
    }

    pub fn unplug(&mut self) {
        self.sysfs.remove(COMMAND_NODE);
        self.sysfs.remove(EVENTS_NODE);
        self.plugged = false;
        self.command = CommandNode::Closed;
        self.events_peer = None;
    }

    pub fn flap(&mut self, flap: Flap) {
        self.headset.claimed_by_other();
        self.command = CommandNode::Missing {
            back_at: Some(self.now.after(flap.node_back_after)),
            readable_at: self.now.after(flap.node_back_after + flap.unreadable_for),
            stale_frame: flap.stale_frame,
        };
        self.sysfs.remove(COMMAND_NODE);
    }

    pub fn remove_command_node(&mut self) {
        self.command = CommandNode::Missing {
            back_at: None,
            readable_at: self.now,
            stale_frame: false,
        };
        self.sysfs.remove(COMMAND_NODE);
    }

    pub fn fail_next_write(&mut self) {
        if let CommandNode::Open(peer) = std::mem::replace(&mut self.command, CommandNode::Closed) {
            rustix::net::shutdown(&peer, Shutdown::Read).expect("shutdown");
            self.severed.push(peer);
        }
        self.flap(Flap {
            node_back_after: FAILED_WRITE_FLAP,
            ..Flap::default()
        });
    }

    pub fn emit(&mut self, emissions: Vec<Emission>) {
        self.inbox.extend(emissions);
    }

    pub fn open(&mut self, path: &Path) -> Result<Port, OpenError> {
        if self.denied {
            return Err(OpenError::Denied);
        }
        if !self.plugged {
            return Err(OpenError::Gone);
        }
        if path == self.sysfs.dev_node(EVENTS_NODE) {
            let (ours, theirs) = pair();
            self.events_peer = Some(theirs);
            return Ok(Port::from_fd(ours));
        }
        if path == self.sysfs.dev_node(COMMAND_NODE) {
            return self.open_command();
        }
        Err(OpenError::Gone)
    }

    fn open_command(&mut self) -> Result<Port, OpenError> {
        let stale_frame = match self.command {
            CommandNode::Missing { back_at, .. } if back_at.is_none_or(|at| self.now < at) => {
                return Err(OpenError::Gone);
            }
            CommandNode::Missing { readable_at, .. } if self.now < readable_at => {
                return Err(OpenError::Denied);
            }
            CommandNode::Missing { stale_frame, .. } => stale_frame,
            CommandNode::Closed | CommandNode::Open(_) => false,
        };
        let (ours, theirs) = pair();
        self.command = CommandNode::Open(theirs);
        if stale_frame {
            self.inbox.push(Emission {
                at: self.now.after(1),
                interface: Interface::Command,
                bytes: FakeHeadset::other_claimant_reply(),
            });
        }
        Ok(Port::from_fd(ours))
    }

    pub fn advance(&mut self, deadline: Option<Millis>) -> Advance {
        match self.next_stop(deadline) {
            Stop::Emission(at) => {
                self.advance_to(at);
                self.deliver_due();
                Advance::Delivered
            }
            Stop::Deadline(at) => {
                self.advance_to(at);
                Advance::TimedOut
            }
            Stop::Horizon => {
                self.advance_to(self.horizon);
                self.reached = true;
                Advance::TimedOut
            }
        }
    }

    fn advance_to(&mut self, at: Millis) {
        self.now = self.now.max(at);
        if self.command_node_is_back() {
            self.sysfs.add(COMMAND_NODE, &NodeSpec::nova_5(3));
        }
    }

    fn command_node_is_back(&self) -> bool {
        let CommandNode::Missing {
            back_at: Some(back),
            ..
        } = self.command
        else {
            return false;
        };
        back <= self.now && self.plugged
    }

    pub fn collect_writes(&mut self) {
        let CommandNode::Open(peer) = &self.command else {
            return;
        };
        while let Some(report) = receive(peer) {
            let frame = frame_from_report(&report).expect("the worker writes whole reports");
            let emissions = self.headset.receive(&frame, self.now);
            self.inbox.extend(emissions);
            self.writes.push((self.now, frame));
        }
    }

    fn next_stop(&self, deadline: Option<Millis>) -> Stop {
        let emission = self.inbox.iter().map(|emission| emission.at).min();
        let emission = emission.filter(|at| *at <= self.horizon);
        let deadline = deadline.filter(|at| *at <= self.horizon);
        match (emission, deadline) {
            (Some(emission), Some(deadline)) if emission <= deadline => Stop::Emission(emission),
            (_, Some(deadline)) => Stop::Deadline(deadline),
            (Some(emission), None) => Stop::Emission(emission),
            (None, None) => Stop::Horizon,
        }
    }

    fn deliver_due(&mut self) {
        let due = self
            .inbox
            .iter()
            .enumerate()
            .filter(|(_, emission)| emission.at <= self.now)
            .min_by_key(|(_, emission)| emission.at)
            .map(|(index, _)| index);
        if let Some(index) = due {
            let emission = self.inbox.remove(index);
            self.deliver(&emission);
        }
    }

    fn deliver(&self, emission: &Emission) {
        let peer = match (emission.interface, &self.command) {
            (Interface::Command, CommandNode::Open(peer)) => Some(peer),
            (Interface::Events, _) => self.events_peer.as_ref(),
            (Interface::Command, _) => None,
        };
        if let Some(peer) = peer {
            send(peer, &emission.bytes);
        }
    }
}
