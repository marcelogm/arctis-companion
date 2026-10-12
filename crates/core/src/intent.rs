use crate::eq::preset::{Preset, PresetId};
use crate::messages::audio::SoundPath;
use crate::settings::headset_settings::{Setting, SettingChange};
use crate::settings::values::{
    AutoPowerOff, MicVolume, MuteLight, Percent, Sidetone, WirelessMode,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Intent {
    SetMicVolume(MicVolume),
    SetSidetone(Sidetone),
    SetMuteLight(MuteLight),
    SetVolumeLimiter(bool),
    SetExtendedRange(bool),
    SetAutoPowerOff(AutoPowerOff),
    RetryWrite,
    SetRoutePreset(SoundPath, PresetId),
    ActivatePreset(PresetId),
    UseRoutePreset(SoundPath),
    SetAutomaticPresets(bool),
    SetAppRule {
        identifier: String,
        preset: PresetId,
    },
    RemoveAppRule(String),
    SavePreset(Preset),
    DeletePreset(PresetId),
    StartPreview(PresetId),
    EndPreview,
    SetLowBatteryThreshold(Percent),
    SetRunInBackground(bool),
}

impl Intent {
    #[must_use]
    pub const fn setting_change(&self) -> Option<SettingChange> {
        match self {
            Self::SetMicVolume(level) => Some(SettingChange::MicVolume(*level)),
            Self::SetSidetone(level) => Some(SettingChange::Sidetone(*level)),
            Self::SetMuteLight(light) => Some(SettingChange::MuteLight(*light)),
            Self::SetVolumeLimiter(on) => Some(SettingChange::VolumeLimiter(*on)),
            Self::SetExtendedRange(on) => Some(SettingChange::WirelessMode(wireless_mode(*on))),
            Self::SetAutoPowerOff(choice) => Some(SettingChange::AutoPowerOff(*choice)),
            _ => None,
        }
    }

    #[must_use]
    pub fn from_scale(setting: Setting, step: u8) -> Option<Self> {
        match setting {
            Setting::MicVolume => MicVolume::try_from(step).ok().map(Self::SetMicVolume),
            Setting::Sidetone => Sidetone::try_from(step).ok().map(Self::SetSidetone),
            Setting::MuteLight => MuteLight::from_step(step).map(Self::SetMuteLight),
            _ => None,
        }
    }

    #[must_use]
    pub const fn from_switch(setting: Setting, active: bool) -> Option<Self> {
        match setting {
            Setting::VolumeLimiter => Some(Self::SetVolumeLimiter(active)),
            Setting::WirelessMode => Some(Self::SetExtendedRange(active)),
            _ => None,
        }
    }

    #[must_use]
    pub fn from_auto_power_off_choice(index: usize) -> Option<Self> {
        AutoPowerOff::CHOICES
            .get(index)
            .copied()
            .map(Self::SetAutoPowerOff)
    }
}

const fn wireless_mode(extended_range: bool) -> WirelessMode {
    if extended_range {
        WirelessMode::ExtendedRange
    } else {
        WirelessMode::Standard
    }
}
