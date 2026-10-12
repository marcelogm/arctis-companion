use crate::protocol::status::{Charging, Link};
use crate::settings::values::Percent;
use crate::time::Millis;

const MILLIS_PER_SECOND: u64 = 1_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnixSeconds(pub u64);

impl UnixSeconds {
    #[must_use]
    pub const fn from_millis(now: Millis) -> Self {
        Self(now.0 / MILLIS_PER_SECOND)
    }

    #[must_use]
    pub const fn minus(self, seconds: u64) -> Self {
        Self(self.0.saturating_sub(seconds))
    }

    #[must_use]
    pub const fn since(self, earlier: Self) -> u64 {
        self.0.saturating_sub(earlier.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Reading {
    pub percent: Percent,
    pub charging: Charging,
    pub link: Link,
}

impl Reading {
    #[must_use]
    pub const fn is_plugged(self) -> bool {
        matches!(self.charging, Charging::Charging | Charging::PluggedIn)
    }

    #[must_use]
    pub const fn is_connected(self) -> bool {
        self.link.is_connected()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Sample {
    pub at: UnixSeconds,
    pub reading: Reading,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sampler {
    last: Option<Reading>,
}

impl Sampler {
    #[must_use]
    pub fn starting_from(history: &[Sample]) -> Self {
        Self {
            last: history.last().map(|sample| sample.reading),
        }
    }

    pub fn observe(&mut self, reading: Reading, at: UnixSeconds) -> Option<Sample> {
        if self.last == Some(reading) {
            return None;
        }
        self.last = Some(reading);
        Some(Sample { at, reading })
    }
}
