use super::timing::{RESCAN_ABSENT, RESCAN_REBINDING};
use crate::messages::device::DeviceUpdate;
use crate::protocol::Frame;
use crate::time::Millis;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cadence {
    Absent,
    Rebinding,
    Off,
}

impl Cadence {
    #[must_use]
    pub const fn interval(self) -> Option<u64> {
        match self {
            Self::Absent => Some(RESCAN_ABSENT),
            Self::Rebinding => Some(RESCAN_REBINDING),
            Self::Off => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    Write(Frame),
    WakeAt(Millis),
    Rescan(Cadence),
    Publish(DeviceUpdate),
}
