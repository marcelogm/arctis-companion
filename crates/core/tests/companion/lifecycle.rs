use arctis_core::battery::sample::{Sample, UnixSeconds};
use arctis_core::companion::adoption::AdoptionStatus;
use arctis_core::companion::alerts::{AlertId, BatteryWarning};
use arctis_core::companion::dongle::WriteFailure;
use arctis_core::companion::selection::{ActivateError, ActivePreset};
use arctis_core::companion::{Action, Input};
use arctis_core::messages::audio::SoundPath;
use arctis_core::messages::device::{DeviceUpdate, Phase};
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::settings::headset_settings::Setting;

use crate::support::{
    Harness, START, adopted, discharging, firmware, id, nova_5, observed, percent, persisted,
    published, wakes,
};

fn status(value: u8) -> DeviceUpdate {
    DeviceUpdate::Status {
        battery: percent(value),
        charging: Charging::Discharging,
        link: Link::Connected,
    }
}

fn identity() -> DeviceUpdate {
    DeviceUpdate::Identity {
        model: nova_5(),
        uniq: None,
        firmware: firmware(16),
    }
}

#[test]
fn the_first_handle_publishes_and_an_idle_one_does_not() {
    let mut h = Harness::new(persisted());
    assert!(published(&h.send(Input::Woke)).is_some());
    assert!(published(&h.send(Input::Woke)).is_none());
}

#[test]
fn a_phase_change_is_published() {
    let mut h = Harness::new(persisted());
    let snapshot = published(&h.device(DeviceUpdate::Phase(Phase::Settling))).expect("published");
    assert_eq!(snapshot.dongle.phase, Phase::Settling);
}

#[test]
fn the_identity_fills_model_firmware_and_observed() {
    let mut h = Harness::new(adopted());
    let _ = h.connect(observed());
    let snapshot = h.snapshot();
    assert_eq!(snapshot.dongle.model, Some(nova_5()));
    assert_eq!(snapshot.dongle.firmware, Some(firmware(16)));
    assert_eq!(snapshot.dongle.observed, Some(observed()));
}

#[test]
fn an_absent_dongle_is_forgotten() {
    let mut h = Harness::new(adopted());
    let _ = h.connect(observed());
    let _ = h.device(status(73));
    let gone = h.at(60_000).device(DeviceUpdate::Phase(Phase::Absent));
    let closing = |action: &Action| matches!(action, Action::RecordBattery(sample) if sample.reading.link == Link::HeadsetOff);
    assert!(gone.iter().any(closing));
    let snapshot = h.snapshot();
    assert_eq!(snapshot.dongle.model, None);
    assert_eq!(snapshot.dongle.adoption, AdoptionStatus::Unknown);
    assert_eq!(snapshot.battery.reading, None);
}

#[test]
fn rebinding_keeps_the_dongle_and_schedules_nothing() {
    let mut h = Harness::new(adopted());
    let _ = h.connect(observed());
    let rebinding = h.at(100).device(DeviceUpdate::Phase(Phase::Rebinding));
    assert!(wakes(&rebinding).is_empty());
    let snapshot = published(&rebinding).expect("published");
    assert_eq!(snapshot.dongle.phase, Phase::Rebinding);
    assert_eq!(snapshot.dongle.adoption, AdoptionStatus::Adopted);
}

#[test]
fn a_write_failure_is_published_until_observed_matches() {
    let mut h = Harness::new(adopted());
    let _ = h.connect(observed());
    let failed =
        published(&h.device(DeviceUpdate::WriteFailed(Setting::Sidetone))).expect("published");
    assert_eq!(failed.dongle.not_applied, [Setting::Sidetone]);
    let failure = WriteFailure {
        serial: 1,
        setting: Setting::Sidetone,
    };
    assert_eq!(failed.dongle.last_failure, Some(failure));
    let _ = h.device(DeviceUpdate::Observed(observed()));
    assert!(h.snapshot().dongle.not_applied.is_empty());
}

#[test]
fn battery_readings_are_recorded_once_and_published() {
    let mut h = Harness::new(persisted());
    let sample = Sample {
        at: UnixSeconds::from_millis(START),
        reading: discharging(73),
    };
    assert!(
        h.device(status(73))
            .contains(&Action::RecordBattery(sample))
    );
    let again = h.at(60_000).device(status(73));
    assert!(
        !again
            .iter()
            .any(|action| matches!(action, Action::RecordBattery(_)))
    );
    assert_eq!(h.snapshot().battery.reading, Some(discharging(73)));
}

#[test]
fn a_low_battery_reading_notifies() {
    let mut h = Harness::new(persisted());
    let actions = h.device(status(15));
    let low = |action: &Action| matches!(action, Action::Notify(alert) if alert.id == AlertId::LowBattery);
    assert!(actions.iter().any(low));
    assert_eq!(h.snapshot().battery.warning, BatteryWarning::Low);
}

#[test]
fn a_critical_reading_withdraws_the_low_notification() {
    let mut h = Harness::new(persisted());
    let _ = h.device(status(15));
    let actions = h.device(status(4));
    let critical = |action: &Action| matches!(action, Action::Notify(alert) if alert.id == AlertId::CriticalBattery);
    assert!(actions.iter().any(critical));
    assert!(actions.contains(&Action::Withdraw(AlertId::LowBattery)));
    assert_eq!(h.snapshot().battery.warning, BatteryWarning::Critical);
}

#[test]
fn a_dongle_that_leaves_withdraws_the_battery_warning() {
    let mut h = Harness::new(persisted());
    let _ = h.device(status(14));
    let actions = h.device(DeviceUpdate::Phase(Phase::Absent));
    assert!(actions.contains(&Action::Withdraw(AlertId::LowBattery)));
    assert_eq!(h.snapshot().battery.warning, BatteryWarning::None);
}

#[test]
fn restored_history_is_in_the_snapshot() {
    let mut saved = persisted();
    saved.battery = vec![Sample {
        at: UnixSeconds(1),
        reading: discharging(50),
    }];
    assert_eq!(
        Harness::new(saved.clone()).snapshot().battery.history,
        saved.battery
    );
}

#[test]
fn the_picked_path_needs_a_route_or_an_adopted_dongle() {
    let mut h = Harness::new(persisted());
    assert_eq!(h.snapshot().picked_path(), Err(ActivateError::NoRoute));
    let _ = h.device(DeviceUpdate::Phase(Phase::HeadsetOff));
    let _ = h.device(identity());
    assert_eq!(h.snapshot().picked_path(), Err(ActivateError::NotReady));
    let mut ready = Harness::new(adopted());
    let _ = ready.connect(observed());
    assert_eq!(ready.snapshot().picked_path(), Ok(SoundPath::Dongle));
}

#[test]
fn a_missing_route_preset_resolves_to_flat() {
    let mut saved = persisted();
    saved.settings.route_presets.dongle = id("gone");
    let snapshot = Harness::new(saved).snapshot();
    assert_eq!(snapshot.presets.route_preset(SoundPath::Dongle), id("flat"));
    assert_eq!(
        snapshot.presets.active.dongle,
        ActivePreset::route(id("flat"))
    );
    assert_eq!(snapshot.presets.resolve(&id("gone")).id(), &id("flat"));
}

#[test]
fn fresh_preset_ids_avoid_every_known_preset() {
    let snapshot = Harness::new(persisted()).snapshot();
    assert_eq!(snapshot.fresh_preset_id("Flat"), id("flat-2"));
}
