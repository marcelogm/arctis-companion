use super::timing::WIRELESS_MODE_GRACE;
use crate::protocol::status::Link;
use crate::time::Millis;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Grace {
    until: Option<Millis>,
    link_down: bool,
}

impl Grace {
    pub(crate) fn open(&mut self, now: Millis) {
        self.until = Some(now.after(WIRELESS_MODE_GRACE));
    }

    pub(crate) fn close(&mut self) {
        self.until = None;
    }

    pub(crate) fn link(&mut self, link: Link) {
        self.link_down = !link.is_connected();
    }

    pub(crate) fn covers(&self, at: Millis) -> bool {
        self.until.is_some_and(|until| at < until)
    }

    pub(crate) fn link_lost_at(&self) -> Option<Millis> {
        self.until.filter(|_| self.link_down)
    }
}
