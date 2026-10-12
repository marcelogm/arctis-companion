pub use crate::bounded::OutOfRange;
use crate::bounded::{bounded, raw_enum};

bounded!(MicVolume, u8, 0, 15);
bounded!(Sidetone, u8, 0, 10);
bounded!(Percent, u8, 0, 100);

raw_enum!(MuteLight {
    Off = 0,
    Low = 1,
    Medium = 4,
    High = 10,
});

impl MuteLight {
    #[must_use]
    pub fn from_raw(raw: u8) -> Self {
        Self::try_from(raw).unwrap_or(Self::Medium)
    }
}

raw_enum!(WirelessMode {
    ExtendedRange = 0,
    Standard = 1,
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AutoPowerOff(u8);

impl AutoPowerOff {
    pub const NEVER: Self = Self(0);
    pub const CHOICES: [Self; 10] = [
        Self(0),
        Self(1),
        Self(5),
        Self(10),
        Self(15),
        Self(30),
        Self(45),
        Self(60),
        Self(75),
        Self(90),
    ];

    #[must_use]
    pub const fn minutes(self) -> u8 {
        self.0
    }

    #[must_use]
    pub fn nearest(minutes: u8) -> Self {
        Self::CHOICES
            .into_iter()
            .min_by_key(|choice| choice.0.abs_diff(minutes))
            .unwrap_or(Self::NEVER)
    }
}

impl TryFrom<u8> for AutoPowerOff {
    type Error = OutOfRange;

    fn try_from(minutes: u8) -> Result<Self, OutOfRange> {
        Self::CHOICES
            .into_iter()
            .find(|choice| choice.0 == minutes)
            .ok_or(OutOfRange)
    }
}
