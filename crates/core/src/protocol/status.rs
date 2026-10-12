use crate::settings::values::{OutOfRange, Percent};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Link {
    NotPaired,
    Searching,
    HeadsetOff,
    Connected,
}

impl Link {
    pub const ALL: [Self; 4] = [
        Self::NotPaired,
        Self::Searching,
        Self::HeadsetOff,
        Self::Connected,
    ];

    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::NotPaired => 0,
            Self::Searching => 1,
            Self::HeadsetOff => 2,
            Self::Connected => 3,
        }
    }

    #[must_use]
    pub const fn is_connected(self) -> bool {
        matches!(self, Self::Connected)
    }
}

impl TryFrom<u8> for Link {
    type Error = OutOfRange;

    fn try_from(raw: u8) -> Result<Self, OutOfRange> {
        Self::ALL
            .into_iter()
            .find(|link| link.raw() == raw)
            .ok_or(OutOfRange)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Charging {
    Unknown,
    Charging,
    PluggedIn,
    Discharging,
}

impl Charging {
    pub const ALL: [Self; 4] = [
        Self::Unknown,
        Self::Charging,
        Self::PluggedIn,
        Self::Discharging,
    ];

    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::Charging => 1,
            Self::PluggedIn => 2,
            Self::Discharging => 3,
        }
    }
}

impl TryFrom<u8> for Charging {
    type Error = OutOfRange;

    fn try_from(raw: u8) -> Result<Self, OutOfRange> {
        Self::ALL
            .into_iter()
            .find(|state| state.raw() == raw)
            .ok_or(OutOfRange)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Status {
    pub link: Link,
    pub battery: Percent,
    pub charging: Charging,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChatMix {
    pub game: Percent,
    pub chat: Percent,
}
