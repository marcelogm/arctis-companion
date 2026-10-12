use arctis_app::dbus::properties::{changed, property_values};
use arctis_app::dbus::state::{
    battery_level, charging_name, error_name, reason_pair, route_name, state_name, warning_name,
};
use arctis_core::battery::sample::Reading;
use arctis_core::companion::adoption::AdoptionStatus;
use arctis_core::companion::alerts::BatteryWarning;
use arctis_core::companion::routing::Route;
use arctis_core::companion::selection::ActivePreset;
use arctis_core::companion::selection::{ActivateError, PresetReason};
use arctis_core::messages::device::Phase;
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::testkit::snapshots::percent;
use arctis_core::testkit::snapshots::{id, rules, snapshot, with};

const ADOPTED: AdoptionStatus = AdoptionStatus::Adopted;
const NEVER: AdoptionStatus = AdoptionStatus::NeverAdopted;
const UNKNOWN: AdoptionStatus = AdoptionStatus::Unknown;

#[test]
fn state_follows_phase_adoption_and_route() {
    let table = [
        (Phase::Absent, NEVER, Route::None, "absent"),
        (Phase::Absent, NEVER, Route::Dongle, "absent"),
        (
            Phase::PermissionDenied,
            NEVER,
            Route::None,
            "permission-denied",
        ),
        (Phase::Absent, NEVER, Route::Bluetooth, "bluetooth-only"),
        (
            Phase::PermissionDenied,
            ADOPTED,
            Route::Bluetooth,
            "bluetooth-only",
        ),
        (Phase::Settling, ADOPTED, Route::Dongle, "connecting"),
        (Phase::Probing, UNKNOWN, Route::None, "connecting"),
        (Phase::Syncing, NEVER, Route::Dongle, "connecting"),
        (Phase::HeadsetOff, NEVER, Route::Dongle, "awaiting-headset"),
        (
            Phase::HeadsetOff,
            UNKNOWN,
            Route::Dongle,
            "awaiting-headset",
        ),
        (Phase::HeadsetOff, ADOPTED, Route::Dongle, "headset-off"),
        (Phase::Ready, ADOPTED, Route::Dongle, "connected"),
        (Phase::Rebinding, ADOPTED, Route::Bluetooth, "connected"),
    ];
    for (phase, adoption, route, expected) in table {
        assert_eq!(
            state_name(phase, adoption, route),
            expected,
            "{phase:?} {adoption:?} {route:?}"
        );
    }
}

fn reading(level: u8, charging: Charging, link: Link) -> Reading {
    Reading {
        percent: percent(level),
        charging,
        link,
    }
}

#[test]
fn battery_level_needs_a_connected_reading() {
    assert_eq!(battery_level(None), -1);
    assert_eq!(
        battery_level(Some(reading(73, Charging::Discharging, Link::HeadsetOff))),
        -1
    );
    assert_eq!(
        battery_level(Some(reading(73, Charging::Discharging, Link::Connected))),
        73
    );
}

#[test]
fn charging_words() {
    assert_eq!(charging_name(None), "unknown");
    let table = [
        (Charging::Unknown, "unknown"),
        (Charging::Charging, "charging"),
        (Charging::PluggedIn, "charged"),
        (Charging::Discharging, "discharging"),
    ];
    for (charging, expected) in table {
        assert_eq!(
            charging_name(Some(reading(50, charging, Link::Connected))),
            expected
        );
    }
}

#[test]
fn warning_words() {
    assert_eq!(warning_name(BatteryWarning::None), "none");
    assert_eq!(warning_name(BatteryWarning::Low), "low");
    assert_eq!(warning_name(BatteryWarning::Critical), "critical");
}

#[test]
fn route_words() {
    assert_eq!(route_name(Route::Dongle), "dongle");
    assert_eq!(route_name(Route::Bluetooth), "bluetooth");
    assert_eq!(route_name(Route::None), "none");
}

#[test]
fn error_names() {
    assert_eq!(
        error_name(ActivateError::NoRoute),
        "io.github.marcelogm.ArctisCompanion.Error.NoRoute"
    );
    assert_eq!(
        error_name(ActivateError::NotReady),
        "io.github.marcelogm.ArctisCompanion.Error.NotReady"
    );
}

#[test]
fn reasons_name_the_app() {
    assert_eq!(reason_pair(&PresetReason::Route), ("route", String::new()));
    assert_eq!(
        reason_pair(&PresetReason::App("Firefox".to_owned())),
        ("app", "Firefox".to_owned())
    );
    assert_eq!(
        reason_pair(&PresetReason::Manual(Some("Firefox".to_owned()))),
        ("manual", "Firefox".to_owned())
    );
    assert_eq!(
        reason_pair(&PresetReason::Manual(None)),
        ("manual", String::new())
    );
}

const SIGNATURES: [(&str, &str); 11] = [
    ("ActivePreset", "s"),
    ("ActivePresetReason", "(ss)"),
    ("AppRuleCount", "u"),
    ("AutomaticPresets", "b"),
    ("BatteryLevel", "i"),
    ("BatteryWarning", "s"),
    ("Charging", "s"),
    ("Model", "s"),
    ("Presets", "a(ss)"),
    ("Route", "s"),
    ("State", "s"),
];

#[test]
fn every_contract_property_has_a_value_of_its_type() {
    let values = property_values(&snapshot());
    let found: Vec<(&str, String)> = values
        .iter()
        .map(|(name, value)| (*name, value.type_().as_str().to_owned()))
        .collect();
    let expected: Vec<(&str, String)> = SIGNATURES
        .iter()
        .map(|(name, signature)| (*name, (*signature).to_owned()))
        .collect();
    assert_eq!(found, expected);
}

#[test]
fn a_ready_dongle_reads_as_connected_with_its_model_and_battery() {
    let values = property_values(&snapshot());
    assert_eq!(
        values["State"].get::<String>().as_deref(),
        Some("connected")
    );
    assert_eq!(
        values["Model"].get::<String>().as_deref(),
        Some("Arctis Nova 5")
    );
    assert_eq!(values["BatteryLevel"].get::<i32>(), Some(73));
    assert_eq!(
        values["Charging"].get::<String>().as_deref(),
        Some("discharging")
    );
    assert_eq!(values["Route"].get::<String>().as_deref(), Some("dongle"));
    assert_eq!(values["AutomaticPresets"].get::<bool>(), Some(true));
}

#[test]
fn presets_list_every_preset_with_its_name() {
    let values = property_values(&snapshot());
    let presets = values["Presets"]
        .get::<Vec<(String, String)>>()
        .expect("a(ss)");
    assert_eq!(presets.len(), snapshot().presets.all.len());
    assert!(presets.contains(&("bass-boost".to_owned(), "Bass Boost".to_owned())));
}

#[test]
fn the_active_preset_is_the_picked_paths() {
    let mut chosen = snapshot();
    chosen.presets.active.dongle = ActivePreset {
        preset: id("footsteps"),
        reason: PresetReason::App("Firefox".to_owned()),
    };
    chosen.presets.rules = rules(&[("firefox", "footsteps"), ("spotify", "bass-boost")]);
    let values = property_values(&chosen);
    assert_eq!(
        values["ActivePreset"].get::<String>().as_deref(),
        Some("footsteps")
    );
    assert_eq!(
        values["ActivePresetReason"].get::<(String, String)>(),
        Some(("app".to_owned(), "Firefox".to_owned()))
    );
    assert_eq!(values["AppRuleCount"].get::<u32>(), Some(2));
}

#[test]
fn no_picked_path_means_no_active_preset() {
    let values = property_values(&with(Phase::Absent, Route::None));
    assert_eq!(values["ActivePreset"].get::<String>().as_deref(), Some(""));
    assert_eq!(
        values["ActivePresetReason"].get::<(String, String)>(),
        Some(("route".to_owned(), String::new()))
    );
}

#[test]
fn only_changed_properties_are_reported() {
    let before = property_values(&snapshot());
    let mut lower = snapshot();
    if let Some(reading) = lower.battery.reading.as_mut() {
        reading.percent = percent(14);
    }
    lower.battery.warning = BatteryWarning::Low;
    let names: Vec<&str> = changed(&before, &property_values(&lower))
        .into_keys()
        .collect();
    assert_eq!(names, ["BatteryLevel", "BatteryWarning"]);
    assert!(changed(&before, &before).is_empty());
}
