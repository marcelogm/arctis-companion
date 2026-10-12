use arctis_core::session::Cadence;
use arctis_core::session::timing::RESCAN_REBINDING;
use arctis_core::time::Millis;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rescan {
    cadence: Cadence,
    next: Option<Millis>,
}

impl Rescan {
    #[must_use]
    pub const fn starting_at(now: Millis) -> Self {
        Self {
            cadence: Cadence::Absent,
            next: Some(now),
        }
    }

    #[must_use]
    pub const fn next(self) -> Option<Millis> {
        self.next
    }

    pub fn set(&mut self, cadence: Cadence, now: Millis) {
        self.cadence = cadence;
        self.next = cadence.interval().map(|interval| now.after(interval));
    }

    pub const fn retry_soon(&mut self, now: Millis) {
        self.next = Some(now.after(RESCAN_REBINDING));
    }

    pub fn take_due(&mut self, now: Millis) -> Option<Cadence> {
        self.next.filter(|at| *at <= now)?;
        self.next = self.cadence.interval().map(|interval| now.after(interval));
        Some(self.cadence)
    }
}
