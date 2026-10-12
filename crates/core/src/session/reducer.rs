use super::echo::EchoFilter;
use super::effect::{Cadence, Effect};
use super::exchange::Exchange;
use super::input::Input;
use super::phase::Stage;
use super::rebind::Rebinding;
use super::timing::{ENUMERATION_SETTLE, HEADSET_OFF_POLL};
use super::verify::Transaction;
use crate::messages::device::{DeviceUpdate, Phase, SyncPolicy};
use crate::protocol::model::{Model, ModelLookup};
use crate::protocol::status::Status;
use crate::settings::headset_settings::{HeadsetSettings, Setting};
use crate::time::Millis;

#[derive(Clone, Debug)]
pub(super) struct Dongle {
    pub(super) model: Model,
    pub(super) uniq: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub(super) now: Millis,
    pub(super) outbox: Vec<Effect>,
    pub(super) stage: Stage,
    pub(super) dongle: Option<Dongle>,
    pub(super) desired: Option<HeadsetSettings>,
    pub(super) observed: Option<HeadsetSettings>,
    pub(super) status: Option<Status>,
    pub(super) last_status_at: Millis,
    pub(super) transaction: Option<Transaction>,
    pub(super) failed: Vec<Setting>,
    pub(super) echoes: EchoFilter,
    pub(super) exchange: Exchange,
    pub(super) published: Phase,
    pub(super) scheduled: Option<Millis>,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    #[must_use]
    pub fn new() -> Self {
        Self {
            now: Millis(0),
            outbox: Vec::new(),
            stage: Stage::Absent,
            dongle: None,
            desired: None,
            observed: None,
            status: None,
            last_status_at: Millis(0),
            transaction: None,
            failed: Vec::new(),
            echoes: EchoFilter::default(),
            exchange: Exchange::default(),
            published: Phase::Absent,
            scheduled: None,
        }
    }

    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.stage.phase()
    }

    pub fn handle(&mut self, input: Input, now: Millis) -> Vec<Effect> {
        self.now = now;
        self.apply(input);
        self.expire();
        self.drive();
        self.schedule();
        std::mem::take(&mut self.outbox)
    }

    fn apply(&mut self, input: Input) {
        match input {
            Input::Found { pid, uniq } => self.found(pid, uniq),
            Input::Denied => self.denied(),
            Input::Lost => self.lost(),
            Input::CommandPortLost => self.command_port_lost(),
            Input::CommandPortBack => self.command_port_back(),
            Input::CommandPortDenied => {}
            Input::Woke => self.scheduled = None,
            Input::Reply(bytes) => self.command_frame(&bytes),
            Input::Unsolicited { interface, bytes } => self.unsolicited(interface, &bytes),
            Input::Desired { settings, policy } => self.desire(settings, policy),
        }
    }

    fn found(&mut self, pid: u16, uniq: Option<String>) {
        if self.stage.is_present() {
            return;
        }
        let ModelLookup::Supported(model) = Model::lookup(pid) else {
            return;
        };
        self.dongle = Some(Dongle { model, uniq });
        self.outbox.push(Effect::Rescan(Cadence::Off));
        let until = self.now.after(ENUMERATION_SETTLE);
        self.enter(Stage::Settling { until });
    }

    fn denied(&mut self) {
        if matches!(self.stage, Stage::Absent) {
            self.enter(Stage::PermissionDenied);
        }
    }

    pub(super) fn lost(&mut self) {
        if matches!(self.stage, Stage::Absent) {
            return;
        }
        let outbox = std::mem::take(&mut self.outbox);
        *self = Self {
            outbox,
            published: self.published,
            ..Self::new()
        };
        self.outbox.push(Effect::Rescan(Cadence::Absent));
        self.announce();
    }

    fn command_port_lost(&mut self) {
        if let Stage::Rebinding(rebinding) = &mut self.stage {
            rebinding.vanished();
            self.outbox.push(Effect::Rescan(Cadence::Rebinding));
            return;
        }
        let Some(resume) = self.stage.resume_point() else {
            return;
        };
        self.interrupt();
        self.stage = Stage::Rebinding(Rebinding::new(resume, self.now));
        self.outbox.push(Effect::Rescan(Cadence::Rebinding));
    }

    fn command_port_back(&mut self) {
        if let Stage::Rebinding(rebinding) = &mut self.stage {
            rebinding.reopened(self.now);
            self.outbox.push(Effect::Rescan(Cadence::Off));
        }
    }

    fn desire(&mut self, settings: HeadsetSettings, policy: SyncPolicy) {
        self.failed.clear();
        match policy {
            SyncPolicy::Enforce => self.desired = Some(settings),
            SyncPolicy::Adopt => {
                self.desired = None;
                self.publish_observed();
            }
        }
    }

    pub(super) fn enter(&mut self, stage: Stage) {
        if matches!(stage, Stage::Syncing(_)) {
            self.failed.clear();
        }
        self.stage = stage;
        self.announce();
    }

    pub(super) fn announce(&mut self) {
        let phase = self.stage.phase();
        if phase != Phase::Rebinding && phase != self.published {
            self.publish_phase(phase);
        }
    }

    pub(super) fn publish_phase(&mut self, phase: Phase) {
        self.published = phase;
        self.publish(DeviceUpdate::Phase(phase));
    }

    pub(super) fn publish_observed(&mut self) {
        if let Some(observed) = self.observed.clone() {
            self.publish(DeviceUpdate::Observed(observed));
        }
    }

    pub(super) fn publish(&mut self, update: DeviceUpdate) {
        self.outbox.push(Effect::Publish(update));
    }

    pub(super) fn headset_off(&mut self) {
        self.interrupt();
        let next_poll = self.now.after(HEADSET_OFF_POLL);
        self.enter(Stage::HeadsetOff { next_poll });
    }

    pub(super) fn interrupt(&mut self) {
        self.transaction = None;
        self.exchange.abandon();
        self.echoes.close(self.now);
    }
}
