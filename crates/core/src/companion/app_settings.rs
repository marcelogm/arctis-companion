use std::collections::BTreeMap;

use super::adoption::DongleIdentity;
use super::per_path::PerPath;
use super::seen_apps::SeenApp;
use crate::battery::sample::Sample;
use crate::eq::curve::Curve;
use crate::eq::preset::{Preset, PresetId};
use crate::messages::audio::{AppIdentifiers, SoundPath};
use crate::settings::headset_settings::{HeadsetSettings, SettingChange};
use crate::settings::values::{
    AutoPowerOff, MicVolume, MuteLight, Percent, Sidetone, WirelessMode,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HeadsetValues {
    pub mic_volume: MicVolume,
    pub sidetone: Sidetone,
    pub mute_light: MuteLight,
    pub volume_limiter: bool,
    pub wireless_mode: WirelessMode,
    pub auto_power_off: AutoPowerOff,
}

impl HeadsetValues {
    #[must_use]
    pub const fn from_settings(settings: &HeadsetSettings) -> Self {
        Self {
            mic_volume: settings.mic_volume,
            sidetone: settings.sidetone,
            mute_light: settings.mute_light,
            volume_limiter: settings.volume_limiter,
            wireless_mode: settings.wireless_mode,
            auto_power_off: settings.auto_power_off,
        }
    }

    #[must_use]
    pub const fn with_curve(self, equalizer: Curve) -> HeadsetSettings {
        HeadsetSettings {
            mic_volume: self.mic_volume,
            sidetone: self.sidetone,
            mute_light: self.mute_light,
            volume_limiter: self.volume_limiter,
            wireless_mode: self.wireless_mode,
            auto_power_off: self.auto_power_off,
            equalizer,
        }
    }

    pub fn apply(&mut self, change: &SettingChange) {
        match *change {
            SettingChange::MicVolume(value) => self.mic_volume = value,
            SettingChange::Sidetone(value) => self.sidetone = value,
            SettingChange::MuteLight(value) => self.mute_light = value,
            SettingChange::VolumeLimiter(value) => self.volume_limiter = value,
            SettingChange::WirelessMode(value) => self.wireless_mode = value,
            SettingChange::AutoPowerOff(value) => self.auto_power_off = value,
            SettingChange::Equalizer(_) => {}
        }
    }
}

pub type RoutePresets = PerPath<PresetId>;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AppRules(BTreeMap<String, PresetId>);

impl AppRules {
    pub fn set(&mut self, identifier: String, preset: PresetId) -> Vec<String> {
        let replaced: Vec<String> = self
            .0
            .keys()
            .filter(|stored| **stored != identifier && stored.eq_ignore_ascii_case(&identifier))
            .cloned()
            .collect();
        for stored in &replaced {
            self.0.remove(stored);
        }
        self.0.insert(identifier, preset);
        replaced
    }

    pub fn remove(&mut self, identifier: &str) {
        self.0.remove(identifier);
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn entries(&self) -> impl Iterator<Item = (&str, &PresetId)> {
        self.0
            .iter()
            .map(|(identifier, preset)| (identifier.as_str(), preset))
    }

    #[must_use]
    pub fn preset_for(&self, identifier: &str) -> Option<&PresetId> {
        self.0
            .iter()
            .find(|(stored, _)| stored.eq_ignore_ascii_case(identifier))
            .map(|(_, preset)| preset)
    }

    #[must_use]
    pub fn matching(&self, ids: &AppIdentifiers) -> Option<&PresetId> {
        ids.as_slice().iter().find_map(|id| self.preset_for(id))
    }
}

impl FromIterator<(String, PresetId)> for AppRules {
    fn from_iter<I: IntoIterator<Item = (String, PresetId)>>(entries: I) -> Self {
        Self(entries.into_iter().collect())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppSettings {
    pub headset: HeadsetValues,
    pub route_presets: RoutePresets,
    pub app_rules: AppRules,
    pub automatic_presets: bool,
    pub adopted_identity: Option<DongleIdentity>,
    pub bluetooth_sink_seen: bool,
    pub low_battery_threshold: Percent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PersistedChange {
    Headset(HeadsetValues),
    RoutePreset(SoundPath, PresetId),
    AppRule {
        identifier: String,
        preset: Option<PresetId>,
    },
    AutomaticPresets(bool),
    AdoptedIdentity(DongleIdentity),
    BluetoothSinkSeen,
    LowBatteryThreshold(Percent),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Persisted {
    pub settings: AppSettings,
    pub custom_presets: Vec<Preset>,
    pub seen_apps: Vec<SeenApp>,
    pub battery: Vec<Sample>,
}
