use super::selection::ActivePreset;
use super::timing::{EDITOR_THROTTLE, auto_switch_min_interval};
use crate::messages::audio::SoundPath;
use crate::time::Millis;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cause {
    User,
    Automatic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Trigger {
    pub now: Millis,
    pub cause: Cause,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Decision {
    Keep,
    Relabel,
    Apply,
    Hold,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct PathEngine {
    applied: Option<ActivePreset>,
    last_automatic: Option<Millis>,
    held_until: Option<Millis>,
}

impl PathEngine {
    fn allowed_at(&self, interval: u64) -> Option<Millis> {
        self.last_automatic.map(|at| at.after(interval))
    }

    fn decision(&self, target: &ActivePreset, trigger: Trigger, interval: u64) -> Decision {
        let Some(applied) = &self.applied else {
            return Decision::Apply;
        };
        if applied == target {
            return Decision::Keep;
        }
        if applied.preset == target.preset {
            return Decision::Relabel;
        }
        match (trigger.cause, self.allowed_at(interval)) {
            (Cause::Automatic, Some(at)) if trigger.now < at => Decision::Hold,
            _ => Decision::Apply,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditorThrottle {
    last_sent: Option<Millis>,
    pending: bool,
}

impl EditorThrottle {
    pub fn request(&mut self) {
        self.pending = true;
    }

    pub fn cancel(&mut self) {
        self.pending = false;
    }

    pub fn take_due(&mut self, now: Millis) -> bool {
        let allowed = self
            .last_sent
            .is_none_or(|sent| now >= sent.after(EDITOR_THROTTLE));
        if !(self.pending && allowed) {
            return false;
        }
        self.pending = false;
        self.last_sent = Some(now);
        true
    }

    #[must_use]
    pub fn next_deadline(&self) -> Option<Millis> {
        self.last_sent
            .filter(|_| self.pending)
            .map(|sent| sent.after(EDITOR_THROTTLE))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Engines {
    dongle: PathEngine,
    bluetooth: PathEngine,
    pub editor: EditorThrottle,
}

impl Engines {
    #[must_use]
    pub fn applied(&self, path: SoundPath) -> Option<&ActivePreset> {
        self.engine(path).applied.as_ref()
    }

    pub fn decide(&mut self, path: SoundPath, target: &ActivePreset, trigger: Trigger) -> Decision {
        let interval = auto_switch_min_interval(path);
        let engine = self.engine_mut(path);
        let decision = engine.decision(target, trigger, interval);
        engine.held_until = if decision == Decision::Hold {
            engine.allowed_at(interval)
        } else {
            None
        };
        decision
    }

    pub fn record(
        &mut self,
        path: SoundPath,
        active: ActivePreset,
        budget_spent_at: Option<Millis>,
    ) {
        let engine = self.engine_mut(path);
        engine.applied = Some(active);
        if budget_spent_at.is_some() {
            engine.last_automatic = budget_spent_at;
        }
    }

    #[must_use]
    pub fn next_deadline(&self) -> Option<Millis> {
        [
            self.dongle.held_until,
            self.bluetooth.held_until,
            self.editor.next_deadline(),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    const fn engine(&self, path: SoundPath) -> &PathEngine {
        match path {
            SoundPath::Dongle => &self.dongle,
            SoundPath::Bluetooth => &self.bluetooth,
        }
    }

    fn engine_mut(&mut self, path: SoundPath) -> &mut PathEngine {
        match path {
            SoundPath::Dongle => &mut self.dongle,
            SoundPath::Bluetooth => &mut self.bluetooth,
        }
    }
}
