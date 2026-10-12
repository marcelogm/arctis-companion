use super::rebind::Rebinding;
use super::syncing::Readings;
use super::timing::ENUMERATION_SETTLE;
use crate::messages::device::Phase;
use crate::time::Millis;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Resume {
    Settling,
    Probing,
    HeadsetOff(Millis),
    Syncing,
    Ready,
}

#[derive(Clone, Debug)]
pub(crate) enum Stage {
    Absent,
    PermissionDenied,
    Settling { until: Millis },
    Probing,
    HeadsetOff { next_poll: Millis },
    Syncing(Readings),
    Ready,
    Rebinding(Rebinding),
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
            Self::Rebinding(_) => Phase::Rebinding,
        }
    }

    pub(crate) const fn resume_point(&self) -> Option<Resume> {
        match self {
            Self::Settling { .. } => Some(Resume::Settling),
            Self::Probing => Some(Resume::Probing),
            Self::HeadsetOff { next_poll } => Some(Resume::HeadsetOff(*next_poll)),
            Self::Syncing(_) => Some(Resume::Syncing),
            Self::Ready => Some(Resume::Ready),
            Self::Absent | Self::PermissionDenied | Self::Rebinding(_) => None,
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

impl Resume {
    pub(crate) fn restart(self, now: Millis) -> Stage {
        match self {
            Self::Settling => Stage::Settling {
                until: now.after(ENUMERATION_SETTLE),
            },
            Self::Probing => Stage::Probing,
            Self::HeadsetOff(next_poll) => Stage::HeadsetOff { next_poll },
            Self::Syncing => Stage::Syncing(Readings::default()),
            Self::Ready => Stage::Ready,
        }
    }

    pub(crate) const fn needs_link(self) -> bool {
        matches!(self, Self::Syncing | Self::Ready)
    }
}
