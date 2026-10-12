use super::phase::Resume;
use super::timing::{REBIND_GIVE_UP, REBIND_NOTICE, REBIND_SETTLE};
use crate::time::Millis;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RebindStep {
    Wait,
    Notice,
    Resume(Resume),
    GiveUp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rebinding {
    resume: Resume,
    since: Millis,
    settle_until: Option<Millis>,
    noticed: bool,
}

impl Rebinding {
    pub(crate) fn new(resume: Resume, now: Millis) -> Self {
        Self {
            resume,
            since: now,
            settle_until: None,
            noticed: false,
        }
    }

    pub(crate) fn reopened(&mut self, now: Millis) {
        self.settle_until = Some(now.after(REBIND_SETTLE));
    }

    pub(crate) fn vanished(&mut self) {
        self.settle_until = None;
    }

    pub(crate) fn deadline(&self) -> Millis {
        let end = self
            .settle_until
            .unwrap_or_else(|| self.since.after(REBIND_GIVE_UP));
        if self.noticed {
            return end;
        }
        end.min(self.since.after(REBIND_NOTICE))
    }

    pub(crate) fn step(&mut self, now: Millis) -> RebindStep {
        match self.settle_until {
            Some(until) if now >= until => return RebindStep::Resume(self.resume),
            None if now >= self.since.after(REBIND_GIVE_UP) => return RebindStep::GiveUp,
            _ => {}
        }
        if self.noticed || now < self.since.after(REBIND_NOTICE) {
            return RebindStep::Wait;
        }
        self.noticed = true;
        RebindStep::Notice
    }
}
