use crate::bounded::raw_enum;
use crate::settings::values::Percent;

raw_enum!(Link {
    NotPaired = 0,
    Searching = 1,
    HeadsetOff = 2,
    Connected = 3,
});

impl Link {
    #[must_use]
    pub const fn is_connected(self) -> bool {
        matches!(self, Self::Connected)
    }
}

raw_enum!(Charging {
    Unknown = 0,
    Charging = 1,
    PluggedIn = 2,
    Discharging = 3,
});

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
