use super::effect::{Cadence, Effect};
use super::exchange::Exchange;
use super::input::Input;
use super::phase::Stage;
use super::rebind::Rebinding;
use super::reconcile::Reconciler;
use super::timing::{ENUMERATION_SETTLE, HEADSET_OFF_POLL};
use crate::messages::device::{DeviceUpdate, Phase, SyncPolicy};
use crate::protocol::firmware::Firmware;
use crate::protocol::model::{Model, ModelLookup};
use crate::protocol::status::Status;
use crate::settings::headset_settings::HeadsetSettings;
use crate::time::Millis;

const UNREAD_FIRMWARE: Firmware = Firmware {
    receiver: None,
    headset: None,
};

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
    pub(super) status: Option<Status>,
    pub(super) last_status_at: Millis,
    pub(super) reconciler: Reconciler,
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
            status: None,
            last_status_at: Millis(0),
            reconciler: Reconciler::default(),
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
        self.publish_identity(UNREAD_FIRMWARE);
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
        let updates = self.reconciler.desire(settings, policy, self.now);
        self.publish_all(updates);
    }

    pub(super) fn enter(&mut self, stage: Stage) {
        if matches!(stage, Stage::Syncing(_)) {
            self.reconciler.start_syncing();
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

    pub(super) fn publish_identity(&mut self, firmware: Firmware) {
        if let Some(dongle) = self.dongle.clone() {
            self.publish(DeviceUpdate::Identity {
                model: dongle.model,
                uniq: dongle.uniq,
                firmware,
            });
        }
    }

    pub(super) fn publish_all(&mut self, updates: Vec<DeviceUpdate>) {
        self.outbox.extend(updates.into_iter().map(Effect::Publish));
    }

    pub(super) fn publish(&mut self, update: DeviceUpdate) {
        self.outbox.push(Effect::Publish(update));
    }

    pub(super) fn headset_off(&mut self) {
        self.interrupt();
        self.reconciler.end_grace();
        let next_poll = self.now.after(HEADSET_OFF_POLL);
        self.enter(Stage::HeadsetOff { next_poll });
    }

    pub(super) fn interrupt(&mut self) {
        self.reconciler.interrupt(self.now);
        self.exchange.abandon();
    }
}
