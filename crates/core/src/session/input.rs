use crate::messages::device::{Change, SyncPolicy};
use crate::settings::headset_settings::HeadsetSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Interface {
    Command,
    Events,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Found {
        pid: u16,
        uniq: Option<String>,
    },
    Denied,
    Lost,
    CommandPortLost,
    CommandPortDenied,
    CommandPortBack,
    Reply(Vec<u8>),
    Unsolicited {
        interface: Interface,
        bytes: Vec<u8>,
    },
    Woke,
    Desired {
        settings: HeadsetSettings,
        policy: SyncPolicy,
    },
}

impl From<Change> for Input {
    fn from(change: Change) -> Self {
        match change {
            Change::Desired { settings, policy } => Self::Desired { settings, policy },
        }
    }
}
