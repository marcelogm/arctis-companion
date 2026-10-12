use std::collections::VecDeque;

use super::exchange::Outgoing;
use super::timing::{VERIFY_BACKOFF, WIRELESS_MODE_GRACE};
use crate::eq::curve::equivalent;
use crate::protocol::status::Link;
use crate::protocol::{Command, Query};
use crate::settings::headset_settings::{HeadsetSettings, Setting, SettingChange};
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
    grace_until: Option<Millis>,
    link_down: bool,
    sent: Vec<Setting>,
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
            grace_until: None,
            link_down: false,
            sent: Vec::new(),
        }
    }

    pub(crate) fn next(&mut self, now: Millis) -> Option<Outgoing> {
        if let Some(change) = self.unwritten.pop_front() {
            self.wrote(&change, now);
            return Some(Outgoing::Write(change));
        }
        if self.readback_due_at().is_none_or(|due| now < due) {
            return None;
        }
        self.unread.pop_front().map(Outgoing::Ask)
    }

    pub(crate) fn due_at(&self, now: Millis) -> Option<Millis> {
        if self.unwritten.is_empty() {
            self.readback_due_at()
        } else {
            Some(now)
        }
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

    pub(crate) fn keep_wanted(&mut self, desired: &HeadsetSettings) -> Vec<Setting> {
        let wanted = |change: &SettingChange| same(change, &desired.value_of(change.setting()));
        self.unwritten.retain(wanted);
        let (kept, dropped): (Vec<_>, Vec<_>) = self.targets.drain(..).partition(wanted);
        self.targets = kept;
        dropped
            .iter()
            .map(SettingChange::setting)
            .filter(|setting| self.sent.contains(setting))
            .collect()
    }

    pub(crate) const fn is_done(&self) -> bool {
        self.targets.is_empty()
    }

    pub(crate) fn requeue(&mut self, query: Query) {
        self.unread.push_front(query);
    }

    pub(crate) fn tolerates(&self, asked_at: Millis) -> bool {
        self.grace_until.is_some_and(|until| asked_at < until)
    }

    pub(crate) fn link(&mut self, link: Link) {
        self.link_down = !link.is_connected();
    }

    pub(crate) fn link_lost_at(&self) -> Option<Millis> {
        self.link_down
            .then(|| self.grace_until.unwrap_or(self.written_at))
    }

    pub(crate) fn lost_link_for_good(&self, now: Millis) -> bool {
        self.link_lost_at().is_some_and(|at| now >= at)
    }

    fn readback_due_at(&self) -> Option<Millis> {
        if self.link_down || self.unread.is_empty() {
            return None;
        }
        Some(self.written_at.after(backoff(self.step)))
    }

    fn wrote(&mut self, change: &SettingChange, now: Millis) {
        self.written_at = now;
        self.sent.push(change.setting());
        if change.setting() == Setting::WirelessMode {
            self.grace_until = Some(now.after(WIRELESS_MODE_GRACE));
        }
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

#[must_use]
pub(crate) fn same(a: &SettingChange, b: &SettingChange) -> bool {
    match (a, b) {
        (SettingChange::Equalizer(left), SettingChange::Equalizer(right)) => {
            equivalent(left, right)
        }
        _ => a == b,
    }
}

fn readbacks(changes: &[SettingChange]) -> Vec<Query> {
    let mut queries = Vec::new();
    for query in changes
        .iter()
        .filter_map(|change| Command::from(change.clone()).readback())
    {
        if !queries.contains(&query) {
            queries.push(query);
        }
    }
    queries
}

fn backoff(step: usize) -> u64 {
    VERIFY_BACKOFF.get(step).copied().unwrap_or(0)
}
