use super::eq_wire::encode_curve;
use super::frame::Frame;
use crate::eq::Curve;
use crate::settings::headset_settings::SettingChange;
use crate::settings::values::{AutoPowerOff, MicVolume, MuteLight, Sidetone, WirelessMode};

pub(crate) const VOLUME_LIMITER: u8 = 0x27;
pub(crate) const EQUALIZER: u8 = 0x33;
pub(crate) const WIRELESS_MODE: u8 = 0x35;
pub(crate) const MIC_VOLUME: u8 = 0x37;
pub(crate) const SIDETONE: u8 = 0x39;
pub(crate) const AUTO_POWER_OFF: u8 = 0xA3;
pub(crate) const MUTE_LIGHT: u8 = 0xAE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Query {
    Status,
    Firmware,
    Audio,
    Ux,
    Wireless,
}

impl Query {
    pub const ALL: [Self; 5] = [
        Self::Status,
        Self::Firmware,
        Self::Audio,
        Self::Ux,
        Self::Wireless,
    ];

    #[must_use]
    pub const fn id(self) -> u8 {
        match self {
            Self::Status => 0xB0,
            Self::Firmware => 0x10,
            Self::Audio => 0x20,
            Self::Ux => 0xA0,
            Self::Wireless => 0x36,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    QueryStatus,
    QueryFirmware,
    QueryAudio,
    QueryUx,
    QueryWireless,
    SetMicVolume(MicVolume),
    SetSidetone(Sidetone),
    SetVolumeLimiter(bool),
    SetEqualizer(Curve),
    SetMuteLight(MuteLight),
    SetAutoPowerOff(AutoPowerOff),
    SetWirelessMode(WirelessMode),
}

impl Command {
    #[must_use]
    pub const fn readback(&self) -> Option<Query> {
        match self {
            Self::QueryStatus
            | Self::QueryFirmware
            | Self::QueryAudio
            | Self::QueryUx
            | Self::QueryWireless => None,
            Self::SetMicVolume(_)
            | Self::SetSidetone(_)
            | Self::SetVolumeLimiter(_)
            | Self::SetEqualizer(_) => Some(Query::Audio),
            Self::SetMuteLight(_) | Self::SetAutoPowerOff(_) => Some(Query::Ux),
            Self::SetWirelessMode(_) => Some(Query::Wireless),
        }
    }
}

impl From<Query> for Command {
    fn from(query: Query) -> Self {
        match query {
            Query::Status => Self::QueryStatus,
            Query::Firmware => Self::QueryFirmware,
            Query::Audio => Self::QueryAudio,
            Query::Ux => Self::QueryUx,
            Query::Wireless => Self::QueryWireless,
        }
    }
}

impl From<SettingChange> for Command {
    fn from(change: SettingChange) -> Self {
        match change {
            SettingChange::MicVolume(value) => Self::SetMicVolume(value),
            SettingChange::Sidetone(value) => Self::SetSidetone(value),
            SettingChange::MuteLight(value) => Self::SetMuteLight(value),
            SettingChange::VolumeLimiter(value) => Self::SetVolumeLimiter(value),
            SettingChange::WirelessMode(value) => Self::SetWirelessMode(value),
            SettingChange::AutoPowerOff(value) => Self::SetAutoPowerOff(value),
            SettingChange::Equalizer(value) => Self::SetEqualizer(value),
        }
    }
}

#[must_use]
pub fn encode(command: &Command) -> Frame {
    match command {
        Command::QueryStatus => query_frame(Query::Status),
        Command::QueryFirmware => query_frame(Query::Firmware),
        Command::QueryAudio => query_frame(Query::Audio),
        Command::QueryUx => query_frame(Query::Ux),
        Command::QueryWireless => query_frame(Query::Wireless),
        Command::SetMicVolume(level) => Frame::new(MIC_VOLUME, &[level.get()]),
        Command::SetSidetone(level) => Frame::new(SIDETONE, &[level.get()]),
        Command::SetVolumeLimiter(on) => Frame::new(VOLUME_LIMITER, &[u8::from(*on)]),
        Command::SetEqualizer(curve) => Frame::new(EQUALIZER, &encode_curve(curve)),
        Command::SetMuteLight(light) => Frame::new(MUTE_LIGHT, &[light.raw()]),
        Command::SetAutoPowerOff(delay) => Frame::new(AUTO_POWER_OFF, &[delay.minutes()]),
        Command::SetWirelessMode(mode) => Frame::new(WIRELESS_MODE, &[mode.raw()]),
    }
}

fn query_frame(query: Query) -> Frame {
    Frame::new(query.id(), &[])
}
