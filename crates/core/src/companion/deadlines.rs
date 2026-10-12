use super::action::Action;
use crate::time::Millis;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Deadlines {
    scheduled: Option<Millis>,
}

impl Deadlines {
    pub fn schedule(&mut self, earliest: Option<Millis>) -> Option<Action> {
        if earliest.is_none() || earliest == self.scheduled {
            return None;
        }
        self.scheduled = earliest;
        earliest.map(Action::WakeAt)
    }

    pub fn woke(&mut self) {
        self.scheduled = None;
    }
}

#[must_use]
pub fn earliest(deadlines: impl IntoIterator<Item = Option<Millis>>) -> Option<Millis> {
    deadlines.into_iter().flatten().min()
}
