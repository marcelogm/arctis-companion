use std::collections::VecDeque;

use super::exchange::Outgoing;
use super::reconcile::{readbacks, same};
use super::timing::VERIFY_BACKOFF;
use crate::protocol::Query;
use crate::settings::headset_settings::{Setting, SettingChange};
use crate::time::Millis;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Progress {
    Pending,
    Done,
    Failed(Vec<Setting>),
}

#[derive(Clone, Debug)]
pub(crate) struct Transaction {
    targets: Vec<SettingChange>,
    unwritten: VecDeque<SettingChange>,
    unread: VecDeque<Query>,
    written_at: Millis,
    step: usize,
    retried: bool,
    link_down: bool,
}

impl Transaction {
    pub(crate) fn new(changes: Vec<SettingChange>) -> Self {
        Self {
            unwritten: changes.iter().cloned().collect(),
            targets: changes,
            unread: VecDeque::new(),
            written_at: Millis(0),
            step: 0,
            retried: false,
            link_down: false,
        }
    }

    pub(crate) fn next(&mut self, now: Millis) -> Option<Outgoing> {
        if let Some(change) = self.unwritten.pop_front() {
            self.wrote(now);
            return Some(Outgoing::Write(change));
        }
        if self.due_at().is_none_or(|due| now < due) {
            return None;
        }
        self.unread.pop_front().map(Outgoing::Ask)
    }

    pub(crate) fn due_at(&self) -> Option<Millis> {
        if !self.unwritten.is_empty() {
            return Some(self.written_at);
        }
        if self.link_down || self.unread.is_empty() {
            return None;
        }
        Some(self.written_at.after(backoff(self.step)))
    }

    pub(crate) fn check(&mut self, readings: &[SettingChange]) -> Progress {
        self.targets
            .retain(|target| !readings.iter().any(|reading| same(reading, target)));
        if !self.unread.is_empty() {
            return Progress::Pending;
        }
        if self.targets.is_empty() {
            return Progress::Done;
        }
        self.next_step()
    }

    pub(crate) fn forget(&mut self, setting: Setting) -> bool {
        self.targets.retain(|target| target.setting() != setting);
        self.unwritten.retain(|change| change.setting() != setting);
        self.targets.is_empty()
    }

    pub(crate) fn link(&mut self, up: bool) {
        self.link_down = !up;
    }

    pub(crate) fn link_lost_at(&self) -> Option<Millis> {
        self.link_down.then_some(self.written_at)
    }

    pub(crate) fn lost_link_for_good(&self, now: Millis) -> bool {
        self.link_lost_at().is_some_and(|at| now >= at)
    }

    fn wrote(&mut self, now: Millis) {
        self.written_at = now;
        if self.unwritten.is_empty() {
            self.step = 0;
            self.unread = readbacks(&self.targets).into();
        }
    }

    fn next_step(&mut self) -> Progress {
        if self.step + 1 < VERIFY_BACKOFF.len() {
            self.step += 1;
            self.unread = readbacks(&self.targets).into();
            return Progress::Pending;
        }
        if !self.retried {
            self.retried = true;
            self.unwritten = self.targets.iter().cloned().collect();
            return Progress::Pending;
        }
        Progress::Failed(self.targets.iter().map(SettingChange::setting).collect())
    }
}

fn backoff(step: usize) -> u64 {
    VERIFY_BACKOFF.get(step).copied().unwrap_or(0)
}
