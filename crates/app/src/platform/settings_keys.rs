use std::collections::HashMap;
use std::fmt::Debug;

use arctis_core::companion::adoption::DongleIdentity;
use arctis_core::companion::app_settings::{AppRules, HeadsetValues, RoutePresets};
use arctis_core::eq::preset::PresetId;
use arctis_core::eq::reference::Overlays;
use arctis_core::messages::audio::SoundPath;
use arctis_core::settings::values::{
    AutoPowerOff, MicVolume, MuteLight, Percent, Sidetone, WirelessMode,
};
use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib;

use crate::logging::warn;

pub const MIC_VOLUME: &str = "mic-volume";
pub const SIDETONE: &str = "sidetone";
pub const MUTE_LIGHT: &str = "mute-light";
pub const VOLUME_LIMITER: &str = "volume-limiter";
pub const EXTENDED_RANGE: &str = "extended-range";
pub const AUTO_POWER_OFF: &str = "auto-power-off";
pub const PRESET_DONGLE: &str = "preset-dongle";
pub const PRESET_BLUETOOTH: &str = "preset-bluetooth";
pub const APP_RULES: &str = "app-rules";
pub const AUTOMATIC_PRESETS: &str = "automatic-presets";
pub const ADOPTED_IDENTITY: &str = "adopted-identity";
pub const BLUETOOTH_SINK_SEEN: &str = "bluetooth-sink-seen";
pub const LOW_BATTERY_THRESHOLD: &str = "low-battery-threshold";
pub const RUN_IN_BACKGROUND: &str = "run-in-background";
pub const BACKGROUND_REQUESTED: &str = "background-requested";
pub const GRAPH_HEADSET: &str = "graph-headset";
pub const GRAPH_TARGET: &str = "graph-target";

#[must_use]
pub fn accepted<T: TryFrom<u8> + Debug>(key: &str, stored: i32, fallback: fn(u8) -> T) -> T {
    let raw = small(stored);
    T::try_from(raw).unwrap_or_else(|_| {
        let replacement = fallback(raw);
        warn(&format!(
            "{key} holds {stored}, which the headset does not take; reading it as {replacement:?}"
        ));
        replacement
    })
}

#[must_use]
pub fn headset(settings: &gio::Settings) -> HeadsetValues {
    HeadsetValues {
        mic_volume: accepted(MIC_VOLUME, settings.int(MIC_VOLUME), MicVolume::nearest),
        sidetone: accepted(SIDETONE, settings.int(SIDETONE), Sidetone::nearest),
        mute_light: accepted(MUTE_LIGHT, settings.enum_(MUTE_LIGHT), MuteLight::from_raw),
        volume_limiter: settings.boolean(VOLUME_LIMITER),
        wireless_mode: wireless_mode(settings.boolean(EXTENDED_RANGE)),
        auto_power_off: accepted(
            AUTO_POWER_OFF,
            settings.enum_(AUTO_POWER_OFF),
            AutoPowerOff::nearest,
        ),
    }
}

#[must_use]
pub fn low_battery_threshold(settings: &gio::Settings) -> Percent {
    let stored = settings.int(LOW_BATTERY_THRESHOLD);
    accepted(LOW_BATTERY_THRESHOLD, stored, Percent::nearest)
}

pub fn write_headset(
    settings: &gio::Settings,
    values: HeadsetValues,
) -> Result<(), glib::BoolError> {
    settings.set_int(MIC_VOLUME, i32::from(values.mic_volume.get()))?;
    settings.set_int(SIDETONE, i32::from(values.sidetone.get()))?;
    settings.set_enum(MUTE_LIGHT, i32::from(values.mute_light.raw()))?;
    settings.set_boolean(VOLUME_LIMITER, values.volume_limiter)?;
    let extended = values.wireless_mode == WirelessMode::ExtendedRange;
    settings.set_boolean(EXTENDED_RANGE, extended)?;
    settings.set_enum(AUTO_POWER_OFF, i32::from(values.auto_power_off.minutes()))
}

#[must_use]
pub const fn route_key(path: SoundPath) -> &'static str {
    match path {
        SoundPath::Dongle => PRESET_DONGLE,
        SoundPath::Bluetooth => PRESET_BLUETOOTH,
    }
}

#[must_use]
pub fn route_presets(settings: &gio::Settings) -> RoutePresets {
    RoutePresets {
        dongle: PresetId::new(settings.string(PRESET_DONGLE).to_string()),
        bluetooth: PresetId::new(settings.string(PRESET_BLUETOOTH).to_string()),
    }
}

#[must_use]
pub fn app_rules(settings: &gio::Settings) -> AppRules {
    stored_rules(settings)
        .into_iter()
        .map(|(identifier, preset)| (identifier, PresetId::new(preset)))
        .collect()
}

pub fn write_app_rule(
    settings: &gio::Settings,
    identifier: &str,
    preset: Option<&PresetId>,
) -> Result<(), glib::BoolError> {
    let mut rules = stored_rules(settings);
    match preset {
        Some(preset) => rules.insert(identifier.to_owned(), preset.as_str().to_owned()),
        None => rules.remove(identifier),
    };
    settings.set(APP_RULES, rules.to_variant())
}

#[must_use]
pub fn overlays(settings: &gio::Settings) -> Overlays {
    Overlays {
        headset: settings.boolean(GRAPH_HEADSET),
        target: None,
    }
    .with_target_setting(&settings.string(GRAPH_TARGET))
}

pub fn write_overlays(settings: &gio::Settings, overlays: Overlays) -> Result<(), glib::BoolError> {
    settings.set_boolean(GRAPH_HEADSET, overlays.headset)?;
    settings.set_string(GRAPH_TARGET, overlays.target_setting())
}

#[must_use]
pub fn adopted_identity(settings: &gio::Settings) -> Option<DongleIdentity> {
    DongleIdentity::from_setting(&settings.string(ADOPTED_IDENTITY))
}

fn stored_rules(settings: &gio::Settings) -> HashMap<String, String> {
    settings.get(APP_RULES)
}

fn small(value: i32) -> u8 {
    u8::try_from(value.max(0)).unwrap_or(u8::MAX)
}

const fn wireless_mode(extended_range: bool) -> WirelessMode {
    if extended_range {
        WirelessMode::ExtendedRange
    } else {
        WirelessMode::Standard
    }
}
