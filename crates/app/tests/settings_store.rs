#[path = "common/schema.rs"]
mod schema;

use arctis_app::platform::settings_store::GSettingsStore;
use arctis_core::companion::adoption::DongleIdentity;
use arctis_core::companion::app_settings::{AppRules, AppSettings, HeadsetValues, RoutePresets};
use arctis_core::eq::preset::PresetId;
use arctis_core::settings::values::{
    AutoPowerOff, MicVolume, MuteLight, Percent, Sidetone, WirelessMode,
};
use gtk::gio::prelude::*;

use schema::{compiled_dir, memory_pair};

const KEYS: [&str; 15] = [
    "mic-volume",
    "sidetone",
    "mute-light",
    "volume-limiter",
    "extended-range",
    "auto-power-off",
    "preset-dongle",
    "preset-bluetooth",
    "app-rules",
    "automatic-presets",
    "adopted-identity",
    "bluetooth-sink-seen",
    "low-battery-threshold",
    "run-in-background",
    "background-requested",
];

#[test]
fn the_schema_declares_every_key() {
    let dir = compiled_dir();
    let (settings, _) = memory_pair(dir.path());
    let schema = settings.settings_schema().expect("schema");
    for key in KEYS {
        assert!(schema.has_key(key), "{key}");
    }
    assert_eq!(schema.list_keys().len(), KEYS.len());
}

fn defaults() -> AppSettings {
    AppSettings {
        headset: HeadsetValues {
            mic_volume: MicVolume::try_from(10).expect("mic volume"),
            sidetone: Sidetone::try_from(0).expect("sidetone"),
            mute_light: MuteLight::Medium,
            volume_limiter: true,
            wireless_mode: WirelessMode::Standard,
            auto_power_off: AutoPowerOff::try_from(30).expect("auto power off"),
        },
        route_presets: RoutePresets {
            dongle: PresetId::new("flat"),
            bluetooth: PresetId::new("flat"),
        },
        app_rules: AppRules::default(),
        automatic_presets: true,
        adopted_identity: None,
        bluetooth_sink_seen: false,
        low_battery_threshold: Percent::try_from(15).expect("percent"),
    }
}

#[test]
fn a_fresh_schema_loads_the_defaults() {
    let dir = compiled_dir();
    let (settings, _) = memory_pair(dir.path());
    assert_eq!(GSettingsStore::new(settings).load(), defaults());
}

fn store_maximums(settings: &gtk::gio::Settings) {
    settings.set_int("mic-volume", 15).expect("mic-volume");
    settings.set_int("sidetone", 10).expect("sidetone");
    settings.set_enum("mute-light", 10).expect("mute-light");
    settings
        .set_boolean("extended-range", true)
        .expect("extended-range");
    settings
        .set_int("auto-power-off", 90)
        .expect("auto-power-off");
    settings
        .set_string("preset-bluetooth", "podcast")
        .expect("preset");
    settings
        .set_string("adopted-identity", "2232")
        .expect("identity");
    settings
        .set_int("low-battery-threshold", 50)
        .expect("threshold");
}

fn assert_maximum_headset(headset: HeadsetValues) {
    assert_eq!(headset.mic_volume.get(), 15);
    assert_eq!(headset.sidetone.get(), 10);
    assert_eq!(headset.mute_light, MuteLight::High);
    assert_eq!(headset.wireless_mode, WirelessMode::ExtendedRange);
    assert_eq!(headset.auto_power_off.minutes(), 90);
}

#[test]
fn stored_values_load_into_app_settings() {
    let dir = compiled_dir();
    let (settings, _) = memory_pair(dir.path());
    store_maximums(&settings);
    let loaded = GSettingsStore::new(settings).load();
    assert_maximum_headset(loaded.headset);
    assert_eq!(loaded.route_presets.bluetooth, PresetId::new("podcast"));
    assert_eq!(
        loaded.adopted_identity,
        Some(DongleIdentity::new(0x2232, None))
    );
    assert_eq!(loaded.low_battery_threshold.get(), 50);
}

#[test]
fn app_rules_load_as_identifier_to_preset() {
    let dir = compiled_dir();
    let (settings, _) = memory_pair(dir.path());
    let rules: std::collections::HashMap<String, String> =
        [("firefox".to_owned(), "footsteps".to_owned())].into();
    settings
        .set("app-rules", rules.to_variant())
        .expect("rules");
    let loaded = GSettingsStore::new(settings).load();
    assert_eq!(
        loaded.app_rules.preset_for("Firefox"),
        Some(&PresetId::new("footsteps"))
    );
    assert_eq!(loaded.app_rules.len(), 1);
}

#[test]
fn an_unreadable_identity_means_never_adopted() {
    let dir = compiled_dir();
    let (settings, _) = memory_pair(dir.path());
    settings
        .set_string("adopted-identity", "not-hex")
        .expect("identity");
    assert_eq!(GSettingsStore::new(settings).load().adopted_identity, None);
}
