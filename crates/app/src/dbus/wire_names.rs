use arctis_core::battery::sample::Reading;
use arctis_core::companion::adoption::AdoptionStatus;
use arctis_core::companion::alerts::BatteryWarning;
use arctis_core::companion::routing::Route;
use arctis_core::companion::selection::{ActivateError, PresetReason};
use arctis_core::messages::device::Phase;
use arctis_core::protocol::status::Charging;

pub const NO_ROUTE: &str = "io.github.marcelogm.ArctisCompanion.Error.NoRoute";
pub const NOT_READY: &str = "io.github.marcelogm.ArctisCompanion.Error.NotReady";

#[must_use]
pub const fn state_name(phase: Phase, adoption: AdoptionStatus, route: Route) -> &'static str {
    match phase {
        Phase::Absent | Phase::PermissionDenied if matches!(route, Route::Bluetooth) => {
            "bluetooth-only"
        }
        Phase::Absent => "absent",
        Phase::PermissionDenied => "permission-denied",
        Phase::Settling | Phase::Probing | Phase::Syncing => "connecting",
        Phase::HeadsetOff if matches!(adoption, AdoptionStatus::Adopted) => "headset-off",
        Phase::HeadsetOff => "awaiting-headset",
        Phase::Ready | Phase::Rebinding => "connected",
    }
}

#[must_use]
pub fn battery_level(reading: Option<Reading>) -> i32 {
    reading
        .filter(|reading| reading.is_connected())
        .map_or(-1, |reading| i32::from(reading.percent.get()))
}

#[must_use]
pub fn charging_name(reading: Option<Reading>) -> &'static str {
    match reading.map(|reading| reading.charging) {
        Some(Charging::Charging) => "charging",
        Some(Charging::PluggedIn) => "charged",
        Some(Charging::Discharging) => "discharging",
        Some(Charging::Unknown) | None => "unknown",
    }
}

#[must_use]
pub const fn warning_name(warning: BatteryWarning) -> &'static str {
    match warning {
        BatteryWarning::None => "none",
        BatteryWarning::Low => "low",
        BatteryWarning::Critical => "critical",
    }
}

#[must_use]
pub const fn route_name(route: Route) -> &'static str {
    match route {
        Route::Dongle => "dongle",
        Route::Bluetooth => "bluetooth",
        Route::None => "none",
    }
}

#[must_use]
pub fn reason_pair(reason: &PresetReason) -> (&'static str, String) {
    match reason {
        PresetReason::Route => ("route", String::new()),
        PresetReason::App(app) => ("app", app.clone()),
        PresetReason::Manual(app) => ("manual", app.clone().unwrap_or_default()),
    }
}

#[must_use]
pub const fn error_name(error: ActivateError) -> &'static str {
    match error {
        ActivateError::NoRoute => NO_ROUTE,
        ActivateError::NotReady => NOT_READY,
    }
}

#[must_use]
pub const fn error_message(error: ActivateError) -> &'static str {
    match error {
        ActivateError::NoRoute => "No headset route is active",
        ActivateError::NotReady => "The headset is not ready yet",
    }
}
