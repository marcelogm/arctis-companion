use super::values::{AutoPowerOff, MicVolume, MuteLight, Sidetone, WirelessMode};
use crate::eq::Curve;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadsetSettings {
    pub mic_volume: MicVolume,
    pub sidetone: Sidetone,
    pub mute_light: MuteLight,
    pub volume_limiter: bool,
    pub wireless_mode: WirelessMode,
    pub auto_power_off: AutoPowerOff,
    pub equalizer: Curve,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Setting {
    MicVolume,
    Sidetone,
    MuteLight,
    VolumeLimiter,
    WirelessMode,
    AutoPowerOff,
    Equalizer,
}

impl Setting {
    pub const ALL: [Self; 7] = [
        Self::MicVolume,
        Self::Sidetone,
        Self::MuteLight,
        Self::VolumeLimiter,
        Self::WirelessMode,
        Self::AutoPowerOff,
        Self::Equalizer,
    ];
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingChange {
    MicVolume(MicVolume),
    Sidetone(Sidetone),
    MuteLight(MuteLight),
    VolumeLimiter(bool),
    WirelessMode(WirelessMode),
    AutoPowerOff(AutoPowerOff),
    Equalizer(Curve),
}

impl SettingChange {
    #[must_use]
    pub const fn setting(&self) -> Setting {
        match self {
            Self::MicVolume(_) => Setting::MicVolume,
            Self::Sidetone(_) => Setting::Sidetone,
            Self::MuteLight(_) => Setting::MuteLight,
            Self::VolumeLimiter(_) => Setting::VolumeLimiter,
            Self::WirelessMode(_) => Setting::WirelessMode,
            Self::AutoPowerOff(_) => Setting::AutoPowerOff,
            Self::Equalizer(_) => Setting::Equalizer,
        }
    }
}

impl HeadsetSettings {
    pub fn apply(&mut self, change: SettingChange) {
        match change {
            SettingChange::MicVolume(value) => self.mic_volume = value,
            SettingChange::Sidetone(value) => self.sidetone = value,
            SettingChange::MuteLight(value) => self.mute_light = value,
            SettingChange::VolumeLimiter(value) => self.volume_limiter = value,
            SettingChange::WirelessMode(value) => self.wireless_mode = value,
            SettingChange::AutoPowerOff(value) => self.auto_power_off = value,
            SettingChange::Equalizer(value) => self.equalizer = value,
        }
    }

    #[must_use]
    pub fn value_of(&self, setting: Setting) -> SettingChange {
        match setting {
            Setting::MicVolume => SettingChange::MicVolume(self.mic_volume),
            Setting::Sidetone => SettingChange::Sidetone(self.sidetone),
            Setting::MuteLight => SettingChange::MuteLight(self.mute_light),
            Setting::VolumeLimiter => SettingChange::VolumeLimiter(self.volume_limiter),
            Setting::WirelessMode => SettingChange::WirelessMode(self.wireless_mode),
            Setting::AutoPowerOff => SettingChange::AutoPowerOff(self.auto_power_off),
            Setting::Equalizer => SettingChange::Equalizer(self.equalizer.clone()),
        }
    }
}
