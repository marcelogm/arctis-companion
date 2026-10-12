use super::effect::Effect;
use super::exchange::{Expiry, Outgoing};
use super::phase::Stage;
use super::reconcile::drift;
use super::reducer::Session;
use super::timing::{HEADSET_OFF_POLL, STATUS_SAFETY_POLL};
use super::verify::Transaction;
use crate::protocol::Query;
use crate::settings::headset_settings::SettingChange;
use crate::time::Millis;

impl Session {
    pub(super) fn expire(&mut self) {
        match self.exchange.expire(self.now) {
            Some(Expiry::Resend(frame)) => self.outbox.push(Effect::Write(frame)),
            Some(Expiry::GaveUp) => self.unanswered(),
            None => {}
        }
        if self
            .transaction
            .as_ref()
            .is_some_and(|transaction| transaction.lost_link_for_good(self.now))
        {
            self.headset_off();
        }
        self.expire_stage();
    }

    fn unanswered(&mut self) {
        if matches!(
            self.stage,
            Stage::Probing | Stage::Syncing(_) | Stage::Ready
        ) {
            self.headset_off();
        }
    }

    fn expire_stage(&mut self) {
        if let Stage::Settling { until } = self.stage
            && self.now >= until
        {
            self.enter(Stage::Probing);
        }
    }

    pub(super) fn drive(&mut self) {
        let free = self.exchange.free_at().is_some_and(|free| self.now >= free);
        self.reconcile(free);
        if free && let Some(outgoing) = self.next_outgoing() {
            self.send(outgoing);
        }
    }

    fn send(&mut self, outgoing: Outgoing) {
        if let Outgoing::Write(change) = &outgoing {
            self.echoes.record(change.clone());
        }
        let frame = self.exchange.send(outgoing, self.now);
        self.outbox.push(Effect::Write(frame));
    }

    fn reconcile(&mut self, free: bool) {
        if self.transaction.is_some() || !self.reconciles() {
            return;
        }
        let changes = self.drift();
        if changes.is_empty() && matches!(self.stage, Stage::Syncing(_)) {
            self.enter(Stage::Ready);
        } else if !changes.is_empty() && free {
            self.transaction = Some(Transaction::new(changes));
        }
    }

    fn reconciles(&self) -> bool {
        match &self.stage {
            Stage::Syncing(readings) => readings.is_complete(),
            Stage::Ready => true,
            _ => false,
        }
    }

    fn drift(&self) -> Vec<SettingChange> {
        match (&self.desired, &self.observed) {
            (Some(desired), Some(observed)) => drift(desired, observed, &self.failed),
            _ => Vec::new(),
        }
    }

    fn next_outgoing(&mut self) -> Option<Outgoing> {
        let now = self.now;
        match &mut self.stage {
            Stage::Probing => Some(Outgoing::Ask(Query::Status)),
            Stage::HeadsetOff { next_poll } if now >= *next_poll => {
                *next_poll = now.after(HEADSET_OFF_POLL);
                Some(Outgoing::Ask(Query::Status))
            }
            Stage::Syncing(readings) if readings.wants_query() => {
                readings.next_query().map(Outgoing::Ask)
            }
            Stage::Syncing(_) => self.transaction.as_mut()?.next(now),
            Stage::Ready => self.ready_outgoing(),
            _ => None,
        }
    }

    fn ready_outgoing(&mut self) -> Option<Outgoing> {
        match &mut self.transaction {
            Some(transaction) => transaction.next(self.now),
            None => (self.now >= self.safety_poll_at()).then_some(Outgoing::Ask(Query::Status)),
        }
    }

    fn safety_poll_at(&self) -> Millis {
        self.last_status_at.after(STATUS_SAFETY_POLL)
    }

    pub(super) fn schedule(&mut self) {
        let wake = self.next_wake();
        if wake != self.scheduled {
            self.outbox.extend(wake.map(Effect::WakeAt));
        }
        self.scheduled = wake;
    }

    fn next_wake(&self) -> Option<Millis> {
        let free = self.exchange.free_at();
        let spacing = free.filter(|free| *free > self.now);
        let timed = self
            .timed_command()
            .zip(free)
            .map(|(due, free)| due.max(free));
        [
            spacing,
            timed,
            self.exchange.deadline(),
            self.stage_deadline(),
            self.transaction
                .as_ref()
                .and_then(Transaction::link_lost_at),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    fn timed_command(&self) -> Option<Millis> {
        match &self.stage {
            Stage::HeadsetOff { next_poll } => Some(*next_poll),
            Stage::Syncing(_) => self.transaction.as_ref().and_then(Transaction::due_at),
            Stage::Ready => self
                .transaction
                .as_ref()
                .map_or(Some(self.safety_poll_at()), Transaction::due_at),
            _ => None,
        }
    }

    fn stage_deadline(&self) -> Option<Millis> {
        match &self.stage {
            Stage::Settling { until } => Some(*until),
            _ => None,
        }
    }
}
