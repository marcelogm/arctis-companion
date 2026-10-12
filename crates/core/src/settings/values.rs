use crate::bounded::bounded;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutOfRange;

bounded!(MicVolume, u8, 0, 15);
bounded!(Sidetone, u8, 0, 10);
bounded!(Percent, u8, 0, 100);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MuteLight {
    Off,
    Low,
    Medium,
    High,
}

impl MuteLight {
    pub const ALL: [Self; 4] = [Self::Off, Self::Low, Self::Medium, Self::High];

    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::Low => 1,
            Self::Medium => 4,
            Self::High => 10,
        }
    }

    #[must_use]
    pub const fn from_raw(raw: u8) -> Self {
        match raw {
            0 => Self::Off,
            1 => Self::Low,
            10 => Self::High,
            _ => Self::Medium,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WirelessMode {
    ExtendedRange,
    Standard,
}

impl WirelessMode {
    pub const ALL: [Self; 2] = [Self::ExtendedRange, Self::Standard];

    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::ExtendedRange => 0,
            Self::Standard => 1,
        }
    }
}

impl TryFrom<u8> for WirelessMode {
    type Error = OutOfRange;

    fn try_from(raw: u8) -> Result<Self, OutOfRange> {
        Self::ALL
            .into_iter()
            .find(|mode| mode.raw() == raw)
            .ok_or(OutOfRange)
    }
}

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
