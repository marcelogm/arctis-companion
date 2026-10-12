use std::collections::VecDeque;
use std::os::fd::{AsFd, BorrowedFd};

use arctis_core::messages::device::{DeviceUpdate, Phase};
use arctis_core::protocol::Frame;
use arctis_core::session::{Cadence, Effect, Input, Interface, Session};
use arctis_core::time::Millis;

use crate::discovery::{self, Dongle, Node, SysfsRoot};
use crate::open_dongle::{OpenDongle, PortKind};
use crate::port::{OpenError, Opener, Port, PortGone, Received};
use crate::rescan::Rescan;
use crate::waiter::Waiter;
use crate::wake::Mailbox;

pub struct Wiring<O, W> {
    pub root: SysfsRoot,
    pub opener: O,
    pub waiter: W,
    pub mailbox: Mailbox,
    pub updates: async_channel::Sender<DeviceUpdate>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Mailbox,
    Port(PortKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    Continue,
    Quit,
}

pub struct Worker<O, W> {
    session: Session,
    root: SysfsRoot,
    opener: O,
    waiter: W,
    mailbox: Mailbox,
    updates: async_channel::Sender<DeviceUpdate>,
    dongle: Option<OpenDongle>,
    rescan: Rescan,
    wake_at: Option<Millis>,
}

impl<O: Opener, W: Waiter> Worker<O, W> {
    pub fn new(wiring: Wiring<O, W>) -> Self {
        let now = wiring.waiter.now();
        Self {
            session: Session::new(),
            root: wiring.root,
            opener: wiring.opener,
            waiter: wiring.waiter,
            mailbox: wiring.mailbox,
            updates: wiring.updates,
            dongle: None,
            rescan: Rescan::starting_at(now),
            wake_at: None,
        }
    }

    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.session.phase()
    }

    pub fn run(mut self) {
        while self.turn() == Flow::Continue {}
    }

    pub fn turn(&mut self) -> Flow {
        for source in self.wait() {
            if self.serve(source) == Flow::Quit {
                return Flow::Quit;
            }
        }
        self.run_timers();
        Flow::Continue
    }

    fn wait(&mut self) -> Vec<Source> {
        let deadline = [self.wake_at, self.rescan.next()]
            .into_iter()
            .flatten()
            .min();
        let mut sources = vec![(Source::Mailbox, self.mailbox.as_fd())];
        let ports = self.dongle.iter().flat_map(OpenDongle::fds);
        sources.extend(ports.map(|(kind, fd)| (Source::Port(kind), fd)));
        let fds: Vec<BorrowedFd<'_>> = sources.iter().map(|(_, fd)| *fd).collect();
        let ready = self.waiter.wait(&fds, deadline);
        sources
            .iter()
            .zip(ready)
            .filter(|(_, ready)| *ready)
            .map(|((source, _), _)| *source)
            .collect()
    }

    fn serve(&mut self, source: Source) -> Flow {
        match source {
            Source::Mailbox => self.read_mail(),
            Source::Port(kind) => {
                self.drain(kind);
                Flow::Continue
            }
        }
    }

    fn read_mail(&mut self) -> Flow {
        let mail = self.mailbox.take();
        for change in mail.changes {
            self.feed(Input::from(change));
        }
        if mail.quit {
            Flow::Quit
        } else {
            Flow::Continue
        }
    }

    fn drain(&mut self, kind: PortKind) {
        while let Some(received) = self.port(kind).map(Port::read) {
            match received {
                Ok(Received::Report(bytes)) => self.feed(report_input(kind, bytes)),
                Ok(Received::Empty) => break,
                Err(PortGone) => {
                    self.port_gone(kind);
                    break;
                }
            }
        }
    }

    fn port(&self, kind: PortKind) -> Option<&Port> {
        let dongle = self.dongle.as_ref()?;
        match kind {
            PortKind::Events => Some(&dongle.events),
            PortKind::Command => dongle.command.as_ref(),
        }
    }

    fn port_gone(&mut self, kind: PortKind) {
        let lost = match kind {
            PortKind::Events => Input::Lost,
            PortKind::Command => self.drop_command_port(),
        };
        self.feed(lost);
    }

    fn drop_command_port(&mut self) -> Input {
        if let Some(dongle) = self.dongle.as_mut() {
            dongle.command = None;
        }
        Input::CommandPortLost
    }

    fn run_timers(&mut self) {
        let now = self.waiter.now();
        if let Some(cadence) = self.rescan.take_due(now) {
            self.rescan_for(cadence);
        }
        if self.wake_at.is_some_and(|at| at <= now) {
            self.wake_at = None;
            self.feed(Input::Woke);
        }
    }

    fn rescan_for(&mut self, cadence: Cadence) {
        let nodes = discovery::scan(&self.root);
        match cadence {
            Cadence::Absent => self.discover(&nodes),
            Cadence::Rebinding => self.reopen_command(&nodes),
            Cadence::Off => {}
        }
    }

    fn discover(&mut self, nodes: &[Node]) {
        match discovery::first_dongle(nodes) {
            Some(found) => self.open_dongle(found),
            None => self.forget_denied_dongle(),
        }
    }

    fn open_dongle(&mut self, found: Dongle) {
        match OpenDongle::open(&mut self.opener, &found) {
            Ok(open) => {
                self.dongle = Some(open);
                self.feed(Input::Found {
                    pid: found.pid,
                    uniq: found.uniq,
                });
            }
            Err(OpenError::Denied) => self.feed(Input::Denied),
            Err(OpenError::Gone) => {}
        }
    }

    fn forget_denied_dongle(&mut self) {
        if self.session.phase() == Phase::PermissionDenied {
            self.feed(Input::Lost);
        }
    }

    fn reopen_command(&mut self, nodes: &[Node]) {
        let Some(dongle) = self.dongle.as_mut() else {
            return;
        };
        let Some(path) = discovery::command_node(nodes, &dongle.physical) else {
            return;
        };
        match self.opener.open(&path) {
            Ok(port) => {
                dongle.command = Some(port);
                self.feed(Input::CommandPortBack);
            }
            Err(OpenError::Denied) => self.feed(Input::CommandPortDenied),
            Err(OpenError::Gone) => {}
        }
    }

    fn feed(&mut self, input: Input) {
        let mut queue = VecDeque::from([input]);
        while let Some(input) = queue.pop_front() {
            let now = self.waiter.now();
            let effects = self.session.handle(input, now);
            queue.extend(
                effects
                    .into_iter()
                    .filter_map(|effect| self.execute(effect, now)),
            );
        }
        if !self.session.phase().is_present() {
            self.dongle = None;
        }
    }

    fn execute(&mut self, effect: Effect, now: Millis) -> Option<Input> {
        match effect {
            Effect::Write(frame) => return self.write(&frame),
            Effect::WakeAt(at) => self.wake_at = Some(at),
            Effect::Rescan(cadence) => self.rescan.set(cadence, now),
            Effect::Publish(update) => {
                let _ = self.updates.try_send(update);
            }
        }
        None
    }

    fn write(&mut self, frame: &Frame) -> Option<Input> {
        let command = self.dongle.as_ref()?.command.as_ref()?;
        match command.write(frame) {
            Ok(()) => None,
            Err(PortGone) => Some(self.drop_command_port()),
        }
    }
}

fn report_input(kind: PortKind, bytes: Vec<u8>) -> Input {
    match kind {
        PortKind::Events => Input::Unsolicited {
            interface: Interface::Events,
            bytes,
        },
        PortKind::Command => Input::Reply(bytes),
    }
}
