#[path = "common/main_context.rs"]
mod main_context;
#[path = "common/memory_settings.rs"]
mod memory_settings;
#[path = "common/schema.rs"]
mod schema;

use std::cell::RefCell;
use std::rc::Rc;

use arctis_app::platform::settings_keys::accepted;
use arctis_app::platform::settings_store::GSettingsStore;
use arctis_app::ports::SettingsStore;
use arctis_core::companion::adoption::DongleIdentity;
use arctis_core::companion::app_settings::PersistedChange;
use arctis_core::companion::app_settings::{AppRules, AppSettings, HeadsetValues, RoutePresets};
use arctis_core::eq::preset::PresetId;
use arctis_core::messages::audio::SoundPath;
use arctis_core::settings::values::{
    AutoPowerOff, MicVolume, MuteLight, Percent, Sidetone, WirelessMode,
};
use gtk::gio::prelude::*;

use main_context::{drain, on_own_context};
use memory_settings::memory_pair;
use schema::compiled_dir;

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
        .set_enum("auto-power-off", 90)
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
fn auto_power_off_takes_only_the_headset_choices() {
    let dir = compiled_dir();
    let (settings, _) = memory_pair(dir.path());
    assert!(settings.set_enum("auto-power-off", 7).is_err());
    assert_eq!(settings.string("auto-power-off"), "30m");
    let store = GSettingsStore::new(settings);
    for choice in AutoPowerOff::CHOICES {
        let mut values = headset_values();
        values.auto_power_off = choice;
        store.persist(&PersistedChange::Headset(values));
        assert_eq!(store.load().headset.auto_power_off, choice);
    }
}

#[test]
fn a_value_the_headset_does_not_take_reads_as_its_fallback() {
    assert_eq!(
        accepted("mic-volume", 40, MicVolume::nearest),
        MicVolume::nearest(15)
    );
    assert_eq!(
        accepted("mic-volume", -3, MicVolume::nearest),
        MicVolume::nearest(0)
    );
    assert_eq!(
        accepted("mute-light", 7, MuteLight::from_raw),
        MuteLight::Medium
    );
    assert_eq!(
        accepted("low-battery-threshold", 50, Percent::nearest).get(),
        50
    );
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

fn persisted_then_loaded(change: &PersistedChange) -> AppSettings {
    let dir = compiled_dir();
    let (settings, _) = memory_pair(dir.path());
    let store = GSettingsStore::new(settings);
    store.persist(change);
    store.load()
}

fn headset_values() -> HeadsetValues {
    HeadsetValues {
        mic_volume: MicVolume::try_from(7).expect("mic volume"),
        sidetone: Sidetone::try_from(3).expect("sidetone"),
        mute_light: MuteLight::Low,
        volume_limiter: false,
        wireless_mode: WirelessMode::ExtendedRange,
        auto_power_off: AutoPowerOff::try_from(15).expect("auto power off"),
    }
}

fn assert_round_trip(change: &PersistedChange, apply: impl FnOnce(&mut AppSettings)) {
    let mut expected = defaults();
    apply(&mut expected);
    assert_eq!(persisted_then_loaded(change), expected, "{change:?}");
}

#[test]
fn headset_and_route_changes_round_trip() {
    assert_round_trip(&PersistedChange::Headset(headset_values()), |s| {
        s.headset = headset_values();
    });
    let bass = PresetId::new("bass-boost");
    assert_round_trip(
        &PersistedChange::RoutePreset(SoundPath::Dongle, bass.clone()),
        |s| {
            s.route_presets.dongle = bass;
        },
    );
    let podcast = PresetId::new("podcast");
    assert_round_trip(
        &PersistedChange::RoutePreset(SoundPath::Bluetooth, podcast.clone()),
        |s| {
            s.route_presets.bluetooth = podcast;
        },
    );
}

#[test]
fn app_and_dongle_changes_round_trip() {
    let rule = PersistedChange::AppRule {
        identifier: "firefox".to_owned(),
        preset: Some(PresetId::new("footsteps")),
    };
    assert_round_trip(&rule, |s| {
        s.app_rules
            .set("firefox".to_owned(), PresetId::new("footsteps"));
    });
    assert_round_trip(&PersistedChange::AutomaticPresets(false), |s| {
        s.automatic_presets = false;
    });
    let identity = DongleIdentity::new(0x2232, Some("ABC"));
    assert_round_trip(&PersistedChange::AdoptedIdentity(identity.clone()), |s| {
        s.adopted_identity = Some(identity);
    });
    assert_round_trip(&PersistedChange::BluetoothSinkSeen, |s| {
        s.bluetooth_sink_seen = true;
    });
    let threshold = Percent::try_from(20).expect("percent");
    assert_round_trip(&PersistedChange::LowBatteryThreshold(threshold), |s| {
        s.low_battery_threshold = threshold;
    });
}

#[test]
fn removing_an_app_rule_deletes_only_that_rule() {
    let dir = compiled_dir();
    let (settings, _) = memory_pair(dir.path());
    let store = GSettingsStore::new(settings);
    for identifier in ["firefox", "spotify"] {
        store.persist(&PersistedChange::AppRule {
            identifier: identifier.to_owned(),
            preset: Some(PresetId::new("footsteps")),
        });
    }
    store.persist(&PersistedChange::AppRule {
        identifier: "firefox".to_owned(),
        preset: None,
    });
    let rules = store.load().app_rules;
    assert_eq!(rules.len(), 1);
    assert!(rules.preset_for("spotify").is_some());
}

type Seen = Rc<RefCell<Vec<AppSettings>>>;

fn watched(store: &GSettingsStore) -> (arctis_app::platform::settings_store::SettingsWatch, Seen) {
    let seen = Seen::default();
    let log = Rc::clone(&seen);
    let watch = store.watch(move |settings| log.borrow_mut().push(settings));
    (watch, seen)
}

#[test]
fn an_outside_change_reaches_the_watcher_once_per_turn() {
    on_own_context(|context| {
        let dir = compiled_dir();
        let (ours, theirs) = memory_pair(dir.path());
        let (_watch, seen) = watched(&GSettingsStore::new(ours));
        theirs.set_int("sidetone", 6).expect("sidetone");
        theirs
            .set_boolean("automatic-presets", false)
            .expect("automatic presets");
        drain(context);
        let seen = seen.borrow();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].headset.sidetone.get(), 6);
        assert!(!seen[0].automatic_presets);
    });
}

#[test]
fn a_multi_key_write_is_seen_whole() {
    on_own_context(|context| {
        let dir = compiled_dir();
        let (ours, _) = memory_pair(dir.path());
        let store = GSettingsStore::new(ours);
        let (_watch, seen) = watched(&store);
        store.persist(&PersistedChange::Headset(headset_values()));
        drain(context);
        assert_eq!(seen.borrow().len(), 1);
        assert_eq!(seen.borrow()[0].headset, headset_values());
    });
}

#[test]
fn dropping_the_watch_stops_it() {
    on_own_context(|context| {
        let dir = compiled_dir();
        let (ours, theirs) = memory_pair(dir.path());
        let (watch, seen) = watched(&GSettingsStore::new(ours));
        drop(watch);
        theirs.set_int("sidetone", 6).expect("sidetone");
        drain(context);
        assert!(seen.borrow().is_empty());
    });
}
