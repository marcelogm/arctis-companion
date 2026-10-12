use arctis_app::dbus::state::{
    battery_level, charging_name, error_name, reason_pair, route_name, state_name, warning_name,
};
use arctis_core::battery::sample::Reading;
use arctis_core::companion::adoption::AdoptionStatus;
use arctis_core::companion::alerts::BatteryWarning;
use arctis_core::companion::routing::Route;
use arctis_core::companion::selection::{ActivateError, PresetReason};
use arctis_core::messages::device::Phase;
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::testkit::snapshots::percent;

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
