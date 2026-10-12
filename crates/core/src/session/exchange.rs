use super::timing::{COMMAND_SPACING, REPLY_TIMEOUT};
use crate::protocol::{Command, Frame, Query, Reply, decode_reply, encode};
use crate::settings::headset_settings::SettingChange;
use crate::time::Millis;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Outgoing {
    Ask(Query),
    Write(SettingChange),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Unanswered {
    pub(crate) query: Query,
    pub(crate) asked_at: Millis,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Expiry {
    Resend(Frame),
    GaveUp(Unanswered),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Awaiting {
    query: Query,
    asked_at: Millis,
    deadline: Millis,
    resent: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Exchange {
    awaiting: Option<Awaiting>,
    last_write: Option<Millis>,
}

impl Exchange {
    pub(crate) fn free_at(&self) -> Option<Millis> {
        if self.awaiting.is_some() {
            return None;
        }
        Some(
            self.last_write
                .map_or(Millis(0), |at| at.after(COMMAND_SPACING)),
        )
    }

    pub(crate) fn deadline(&self) -> Option<Millis> {
        self.awaiting.map(|awaiting| awaiting.deadline)
    }

    pub(crate) fn send(&mut self, outgoing: Outgoing, now: Millis) -> Frame {
        match outgoing {
            Outgoing::Ask(query) => self.ask(query, now),
            Outgoing::Write(change) => self.write(&Command::from(change), now),
        }
    }

    pub(crate) fn answer(&mut self, bytes: &[u8]) -> Option<Reply> {
        let awaiting = self.awaiting?;
        if bytes.first() != Some(&awaiting.query.id()) {
            return None;
        }
        let reply = decode_reply(awaiting.query, bytes).ok()?;
        self.awaiting = None;
        Some(reply)
    }

    pub(crate) fn expire(&mut self, now: Millis) -> Option<Expiry> {
        let awaiting = self.awaiting.filter(|awaiting| now >= awaiting.deadline)?;
        if !awaiting.resent {
            return Some(Expiry::Resend(self.resend(awaiting, now)));
        }
        self.awaiting = None;
        Some(Expiry::GaveUp(Unanswered {
            query: awaiting.query,
            asked_at: awaiting.asked_at,
        }))
    }

    pub(crate) fn abandon(&mut self) {
        self.awaiting = None;
    }

    fn ask(&mut self, query: Query, now: Millis) -> Frame {
        self.awaiting = Some(Awaiting {
            query,
            asked_at: now,
            deadline: now.after(REPLY_TIMEOUT),
            resent: false,
        });
        self.write(&Command::from(query), now)
    }

    fn resend(&mut self, awaiting: Awaiting, now: Millis) -> Frame {
        self.awaiting = Some(Awaiting {
            deadline: now.after(REPLY_TIMEOUT),
            resent: true,
            ..awaiting
        });
        self.write(&Command::from(awaiting.query), now)
    }

    fn write(&mut self, command: &Command, now: Millis) -> Frame {
        self.last_write = Some(now);
        encode(command)
    }
}
