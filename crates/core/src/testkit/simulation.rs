use super::fake_headset::{Emission, FakeHeadset};
use crate::messages::device::{DeviceUpdate, Phase, SyncPolicy};
use crate::protocol::Frame;
use crate::protocol::status::Link;
use crate::session::timing::RESCAN_REBINDING;
use crate::session::{Cadence, Effect, Input, Interface, Session};
use crate::settings::headset_settings::{HeadsetSettings, SettingChange};
use crate::settings::values::Percent;
use crate::time::Millis;

const STEP_LIMIT: usize = 2_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flap {
    pub node_back_after: u64,
    pub unreadable_for: u64,
    pub stale_frame: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommandNode {
    Open,
    Missing {
        back_at: Option<Millis>,
        readable_at: Millis,
        stale_frame: bool,
    },
}

#[derive(Debug)]
pub struct Simulation {
    session: Session,
    headset: FakeHeadset,
    now: Millis,
    wake: Option<Millis>,
    inbox: Vec<Emission>,
    node: CommandNode,
    next_rescan: Option<Millis>,
    fail_next_write: bool,
    steps: usize,
    writes: Vec<(Millis, Frame)>,
    updates: Vec<(Millis, DeviceUpdate)>,
    rescans: Vec<Cadence>,
}

impl Simulation {
    #[must_use]
    pub fn new(headset: FakeHeadset) -> Self {
        Self {
            session: Session::new(),
            headset,
            now: Millis(0),
            wake: None,
            inbox: Vec::new(),
            node: CommandNode::Open,
            next_rescan: None,
            fail_next_write: false,
            steps: 0,
            writes: Vec::new(),
            updates: Vec::new(),
            rescans: Vec::new(),
        }
    }

    #[must_use]
    pub const fn now(&self) -> Millis {
        self.now
    }

    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.session.phase()
    }

    #[must_use]
    pub fn headset(&self) -> &FakeHeadset {
        &self.headset
    }

    pub fn headset_mut(&mut self) -> &mut FakeHeadset {
        &mut self.headset
    }

    #[must_use]
    pub fn writes(&self) -> &[(Millis, Frame)] {
        &self.writes
    }

    #[must_use]
    pub fn updates(&self) -> &[(Millis, DeviceUpdate)] {
        &self.updates
    }

    #[must_use]
    pub fn rescans(&self) -> &[Cadence] {
        &self.rescans
    }

    #[must_use]
    pub fn published_phases(&self) -> Vec<Phase> {
        self.updates
            .iter()
            .filter_map(|(_, update)| match update {
                DeviceUpdate::Phase(phase) => Some(*phase),
                _ => None,
            })
            .collect()
    }

    #[must_use]
    pub fn sent_ids(&self) -> Vec<u8> {
        self.writes
            .iter()
            .map(|(_, frame)| frame.payload()[0])
            .collect()
    }

    pub fn clear_logs(&mut self) {
        self.writes.clear();
        self.updates.clear();
        self.rescans.clear();
    }

    pub fn plug(&mut self) {
        let input = Input::Found {
            pid: self.headset.pid,
            uniq: self.headset.uniq.clone(),
        };
        self.feed(input);
    }

    pub fn deny(&mut self) {
        self.feed(Input::Denied);
    }

    pub fn unplug(&mut self) {
        self.node = CommandNode::Open;
        self.feed(Input::Lost);
    }

    pub fn desire(&mut self, settings: HeadsetSettings, policy: SyncPolicy) {
        self.feed(Input::Desired { settings, policy });
    }

    pub fn flap(&mut self, flap: Flap) {
        self.headset.claimed_by_other();
        self.node = CommandNode::Missing {
            back_at: Some(self.now.after(flap.node_back_after)),
            readable_at: self.now.after(flap.node_back_after + flap.unreadable_for),
            stale_frame: flap.stale_frame,
        };
        self.feed(Input::CommandPortLost);
    }

    pub fn remove_command_node(&mut self) {
        self.node = CommandNode::Missing {
            back_at: None,
            readable_at: self.now,
            stale_frame: false,
        };
        self.feed(Input::CommandPortLost);
    }

    pub fn fail_next_write(&mut self) {
        self.fail_next_write = true;
    }

    pub fn press(&mut self, change: SettingChange) {
        let emissions = self.headset.press(change, self.now);
        self.inbox.extend(emissions);
    }

    pub fn emit(&mut self, emissions: Vec<Emission>) {
        self.inbox.extend(emissions);
    }

    pub fn power_on(&mut self) {
        self.link(Link::Connected);
    }

    pub fn power_off(&mut self) {
        self.link(Link::HeadsetOff);
    }

    pub fn link(&mut self, link: Link) {
        let emissions = self.headset.link_event(link, self.now);
        self.emit(emissions);
    }

    pub fn battery(&mut self, battery: Percent) {
        let emissions = self.headset.battery_event(battery, self.now);
        self.emit(emissions);
    }

    pub fn run_for(&mut self, duration: u64) {
        let end = self.now.after(duration);
        while let Some(at) = self.next_moment().filter(|at| *at <= end) {
            self.steps += 1;
            assert!(
                self.steps < STEP_LIMIT,
                "the simulation took more than {STEP_LIMIT} steps"
            );
            self.now = self.now.max(at);
            self.step();
        }
        self.now = end;
    }

    pub fn run_until(&mut self, phase: Phase, limit: u64) -> u64 {
        let start = self.now;
        while self.phase() != phase {
            assert!(
                self.now.since(start) < limit,
                "{phase:?} not reached within {limit} ms"
            );
            self.run_for(1);
        }
        self.now.since(start)
    }

    pub fn run_until_sent(&mut self, id: u8, limit: u64) {
        let start = self.now;
        let sent = |simulation: &Self| simulation.sent_ids().contains(&id);
        while !sent(self) {
            assert!(
                self.now.since(start) < limit,
                "{id:#04x} not sent within {limit} ms"
            );
            self.run_for(1);
        }
    }

    fn next_moment(&self) -> Option<Millis> {
        let delivery = self.inbox.iter().map(|emission| emission.at).min();
        [self.wake, delivery, self.next_rescan]
            .into_iter()
            .flatten()
            .min()
    }

    fn step(&mut self) {
        if let Some(index) = self.due_emission() {
            let emission = self.inbox.remove(index);
            self.deliver(emission);
        } else if self.next_rescan.is_some_and(|at| at <= self.now) {
            self.rescan();
        } else if self.wake.is_some_and(|at| at <= self.now) {
            self.wake = None;
            self.feed(Input::Woke);
        }
    }

    fn due_emission(&self) -> Option<usize> {
        self.inbox
            .iter()
            .enumerate()
            .filter(|(_, emission)| emission.at <= self.now)
            .min_by_key(|(_, emission)| emission.at)
            .map(|(index, _)| index)
    }

    fn deliver(&mut self, emission: Emission) {
        match emission.interface {
            Interface::Command if self.node != CommandNode::Open => {}
            Interface::Command => self.feed(Input::Reply(emission.bytes)),
            Interface::Events => self.feed(Input::Unsolicited {
                interface: Interface::Events,
                bytes: emission.bytes,
            }),
        }
    }

    fn rescan(&mut self) {
        self.next_rescan = self.next_rescan.map(|at| at.after(RESCAN_REBINDING));
        let CommandNode::Missing {
            back_at,
            readable_at,
            stale_frame,
        } = self.node
        else {
            return;
        };
        if back_at.is_none_or(|back| self.now < back) {
            return;
        }
        if self.now < readable_at {
            self.feed(Input::CommandPortDenied);
            return;
        }
        self.reopen(stale_frame);
    }

    fn reopen(&mut self, stale_frame: bool) {
        self.node = CommandNode::Open;
        if stale_frame {
            self.inbox.push(Emission {
                at: self.now.after(1),
                interface: Interface::Command,
                bytes: FakeHeadset::other_claimant_reply(),
            });
        }
        self.feed(Input::CommandPortBack);
    }

    fn feed(&mut self, input: Input) {
        let effects = self.session.handle(input, self.now);
        for effect in effects {
            self.execute(effect);
        }
    }

    fn execute(&mut self, effect: Effect) {
        match effect {
            Effect::Write(frame) => self.write(frame),
            Effect::WakeAt(at) => self.wake = Some(at),
            Effect::Rescan(cadence) => self.set_cadence(cadence),
            Effect::Publish(update) => self.updates.push((self.now, update)),
        }
    }

    fn write(&mut self, frame: Frame) {
        assert_eq!(
            self.node,
            CommandNode::Open,
            "the session wrote while the command node was gone"
        );
        let now = self.now;
        if std::mem::take(&mut self.fail_next_write) {
            self.writes.push((now, frame));
            self.flap(Flap {
                node_back_after: 60,
                ..Flap::default()
            });
            return;
        }
        let emissions = self.headset.receive(&frame, now);
        self.writes.push((now, frame));
        self.inbox.extend(emissions);
    }

    fn set_cadence(&mut self, cadence: Cadence) {
        self.rescans.push(cadence);
        self.next_rescan = match cadence {
            Cadence::Rebinding => Some(self.now.after(RESCAN_REBINDING)),
            Cadence::Absent | Cadence::Off => None,
        };
    }
}
