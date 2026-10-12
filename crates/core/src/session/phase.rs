use super::syncing::Readings;
use crate::messages::device::Phase;
use crate::time::Millis;

#[derive(Clone, Debug)]
pub(crate) enum Stage {
    Absent,
    PermissionDenied,
    Settling { until: Millis },
    Probing,
    HeadsetOff { next_poll: Millis },
    Syncing(Readings),
    Ready,
}

impl Stage {
    pub(crate) const fn phase(&self) -> Phase {
        match self {
            Self::Absent => Phase::Absent,
            Self::PermissionDenied => Phase::PermissionDenied,
            Self::Settling { .. } => Phase::Settling,
            Self::Probing => Phase::Probing,
            Self::HeadsetOff { .. } => Phase::HeadsetOff,
            Self::Syncing(_) => Phase::Syncing,
            Self::Ready => Phase::Ready,
        }
    }

    pub(crate) const fn drains_command_frames(&self) -> bool {
        !matches!(
            self,
            Self::Probing | Self::HeadsetOff { .. } | Self::Syncing(_) | Self::Ready
        )
    }

    pub(crate) const fn is_present(&self) -> bool {
        !matches!(self, Self::Absent | Self::PermissionDenied)
    }
}
