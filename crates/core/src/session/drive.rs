use super::effect::Effect;
use super::exchange::{Expiry, Outgoing, Unanswered};
use super::phase::Stage;
use super::rebind::RebindStep;
use super::reducer::Session;
use super::timing::{HEADSET_OFF_POLL, STATUS_SAFETY_POLL};
use crate::messages::device::Phase;
use crate::protocol::Query;
use crate::time::Millis;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Next {
    Status,
    HeadsetOffPoll,
    SyncRead,
    Reconcile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Due {
    at: Millis,
    next: Next,
}

impl Due {
    const fn at(at: Millis, next: Next) -> Self {
        Self { at, next }
    }
}

impl Session {
    pub(super) fn expire(&mut self) {
        match self.exchange.expire(self.now) {
            Some(Expiry::Resend(frame)) => self.outbox.push(Effect::Write(frame)),
            Some(Expiry::GaveUp(unanswered)) => self.unanswered(unanswered),
            None => {}
        }
        if self.link_lost_at().is_some_and(|at| self.now >= at) {
            self.headset_off();
        }
        self.expire_stage();
    }

    fn unanswered(&mut self, unanswered: Unanswered) {
        if !self.reconciler.requeue_within_grace(unanswered) {
            self.lost_contact();
        }
    }

    pub(super) fn lost_contact(&mut self) {
        if matches!(
            self.stage,
            Stage::Probing | Stage::Syncing(_) | Stage::Ready
        ) {
            self.headset_off();
        }
    }

    fn expire_stage(&mut self) {
        let now = self.now;
        match &mut self.stage {
            Stage::Settling { until } if now >= *until => {
                self.enter(Stage::Probing);
            }
            Stage::Rebinding(rebinding) => {
                let step = rebinding.step(now);
                self.rebinding_step(step);
            }
            _ => {}
        }
    }

    fn rebinding_step(&mut self, step: RebindStep) {
        match step {
            RebindStep::Wait => {}
            RebindStep::Notice => self.publish_phase(Phase::Rebinding),
            RebindStep::GiveUp => self.lost(),
            RebindStep::Resume(resume) if resume.needs_link() && !self.link_up_or_awaited() => {
                self.headset_off();
            }
            RebindStep::Resume(resume) => self.enter(resume.restart(self.now)),
        }
    }

    fn link_up_or_awaited(&self) -> bool {
        self.status.is_none_or(|status| status.link.is_connected())
            || self.reconciler.waits_for_link(self.now)
    }

    fn link_lost_at(&self) -> Option<Millis> {
        match self.stage {
            Stage::Syncing(_) | Stage::Ready => self.reconciler.link_lost_at(),
            _ => None,
        }
    }

    pub(super) fn drive(&mut self) {
        self.finish_syncing_without_drift();
        let Some(due) = self.due().filter(|due| self.can_send_at(due.at)) else {
            return;
        };
        if let Some(outgoing) = self.take(due.next) {
            self.send(outgoing);
        }
    }

    fn finish_syncing_without_drift(&mut self) {
        if let Stage::Syncing(readings) = &self.stage
            && readings.is_complete()
            && self.reconciler.is_in_sync()
        {
            self.enter(Stage::Ready);
        }
    }

    fn can_send_at(&self, at: Millis) -> bool {
        self.exchange
            .free_at()
            .is_some_and(|free| self.now >= free.max(at))
    }

    fn due(&self) -> Option<Due> {
        let now = self.now;
        match &self.stage {
            Stage::Probing => Some(Due::at(now, Next::Status)),
            Stage::HeadsetOff { next_poll } => Some(Due::at(*next_poll, Next::HeadsetOffPoll)),
            Stage::Syncing(readings) if readings.wants_query() => {
                Some(Due::at(now, Next::SyncRead))
            }
            Stage::Ready if self.reconciler.is_in_sync() => {
                Some(Due::at(self.safety_poll_at(), Next::Status))
            }
            Stage::Syncing(_) | Stage::Ready => self.reconcile_due(),
            _ => None,
        }
    }

    fn reconcile_due(&self) -> Option<Due> {
        let at = self.reconciler.due_at(self.now)?;
        Some(Due::at(at, Next::Reconcile))
    }

    fn take(&mut self, next: Next) -> Option<Outgoing> {
        match next {
            Next::Status => Some(Outgoing::Ask(Query::Status)),
            Next::HeadsetOffPoll => {
                self.postpone_headset_off_poll();
                Some(Outgoing::Ask(Query::Status))
            }
            Next::SyncRead => self.next_sync_read(),
            Next::Reconcile => self.reconciler.next(self.now),
        }
    }

    fn postpone_headset_off_poll(&mut self) {
        let now = self.now;
        if let Stage::HeadsetOff { next_poll } = &mut self.stage {
            *next_poll = now.after(HEADSET_OFF_POLL);
        }
    }

    fn next_sync_read(&mut self) -> Option<Outgoing> {
        match &mut self.stage {
            Stage::Syncing(readings) => readings.next_query().map(Outgoing::Ask),
            _ => None,
        }
    }

    fn send(&mut self, outgoing: Outgoing) {
        let frame = self.exchange.send(outgoing, self.now);
        self.outbox.push(Effect::Write(frame));
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
        let command = self
            .due()
            .zip(self.exchange.free_at())
            .map(|(due, free)| due.at.max(free));
        [
            command,
            self.exchange.deadline(),
            self.stage_deadline(),
            self.link_lost_at(),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    fn stage_deadline(&self) -> Option<Millis> {
        match &self.stage {
            Stage::Settling { until } => Some(*until),
            Stage::Rebinding(rebinding) => Some(rebinding.deadline()),
            _ => None,
        }
    }
}
