use crate::protocol::firmware::Firmware;
use crate::protocol::model::Model;
use crate::protocol::status::{Charging, Link};
use crate::settings::headset_settings::{HeadsetSettings, Setting, SettingChange};
use crate::settings::values::Percent;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SyncPolicy {
    Adopt,
    Enforce,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Desired {
        settings: HeadsetSettings,
        policy: SyncPolicy,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Phase {
    Absent,
    PermissionDenied,
    Settling,
    Probing,
    HeadsetOff,
    Syncing,
    Ready,
    Rebinding,
}

impl Phase {
    #[must_use]
    pub const fn is_present(self) -> bool {
        !matches!(self, Self::Absent | Self::PermissionDenied)
    }

    #[must_use]
    pub const fn is_connecting(self) -> bool {
        matches!(self, Self::Settling | Self::Probing | Self::Syncing)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceUpdate {
    Phase(Phase),
    Identity {
        model: Model,
        uniq: Option<String>,
        firmware: Firmware,
    },
    Observed(HeadsetSettings),
    Status {
        battery: Percent,
        charging: Charging,
        link: Link,
    },
    HeadsetChanged(SettingChange),
    WriteFailed(Setting),
}
