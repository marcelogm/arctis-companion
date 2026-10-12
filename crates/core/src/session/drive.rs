use super::effect::Effect;
use super::exchange::Expiry;
use super::phase::Stage;
use super::reducer::Session;
use super::timing::{HEADSET_OFF_POLL, STATUS_SAFETY_POLL};
use crate::protocol::Query;
use crate::time::Millis;

impl Session {
    pub(super) fn expire(&mut self) {
        match self.exchange.expire(self.now) {
            Some(Expiry::Resend(frame)) => self.outbox.push(Effect::Write(frame)),
            Some(Expiry::GaveUp) => self.unanswered(),
            None => {}
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
        self.reconcile();
        let free = self.exchange.free_at().is_some_and(|free| self.now >= free);
        if free && let Some(query) = self.next_query() {
            let frame = self.exchange.send(query, self.now);
            self.outbox.push(Effect::Write(frame));
        }
    }

    fn reconcile(&mut self) {
        if matches!(&self.stage, Stage::Syncing(readings) if readings.is_complete()) {
            self.enter(Stage::Ready);
        }
    }

    fn next_query(&mut self) -> Option<Query> {
        let now = self.now;
        match &mut self.stage {
            Stage::Probing => Some(Query::Status),
            Stage::HeadsetOff { next_poll } if now >= *next_poll => {
                *next_poll = now.after(HEADSET_OFF_POLL);
                Some(Query::Status)
            }
            Stage::Syncing(readings) if readings.wants_query() => readings.next_query(),
            Stage::Ready => (now >= self.safety_poll_at()).then_some(Query::Status),
            _ => None,
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
        ]
        .into_iter()
        .flatten()
        .min()
    }

    fn timed_command(&self) -> Option<Millis> {
        match &self.stage {
            Stage::HeadsetOff { next_poll } => Some(*next_poll),
            Stage::Ready => Some(self.safety_poll_at()),
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
