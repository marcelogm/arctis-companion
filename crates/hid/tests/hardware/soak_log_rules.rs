use std::time::Duration;

use arctis_core::messages::device::{DeviceUpdate, Phase};
use arctis_core::settings::headset_settings::HeadsetSettings;
use arctis_core::settings::values::Sidetone;
use arctis_core::testkit::FakeHeadset;

use crate::soak_log::SoakLog;

fn with_sidetone(level: u8) -> HeadsetSettings {
    HeadsetSettings {
        sidetone: Sidetone::try_from(level).expect("a sidetone level in range"),
        ..FakeHeadset::nova_5().settings().clone()
    }
}

fn original() -> HeadsetSettings {
    with_sidetone(3)
}

fn changed() -> HeadsetSettings {
    with_sidetone(2)
}

fn log_after(updates: Vec<DeviceUpdate>) -> SoakLog {
    let mut log = SoakLog::default();
    for update in updates {
        log.record(Duration::ZERO, update);
    }
    log
}

fn changed_then(phase: Phase) -> SoakLog {
    log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(original()),
        DeviceUpdate::Observed(changed()),
        DeviceUpdate::Phase(phase),
    ])
}

#[test]
fn a_restore_while_ready_is_requested() {
    let mut log = changed_then(Phase::Ready);
    assert_eq!(log.restore_request(), Some(original()));
    assert_eq!(log.requested, 1);
    assert!(!log.unrestored);
    log.record(Duration::ZERO, DeviceUpdate::Observed(original()));
    assert_eq!(log.confirmed, 1);
}

#[test]
fn a_restore_with_the_headset_off_is_left_for_a_person() {
    let mut log = changed_then(Phase::HeadsetOff);
    assert_eq!(log.restore_request(), None);
    assert_eq!(log.requested, 0);
    assert!(log.unrestored);
}

#[test]
fn unchanged_settings_need_no_restore() {
    let mut log = log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(original()),
    ]);
    assert_eq!(log.restore_request(), None);
    assert!(!log.unrestored);
}

#[test]
fn a_toggle_waits_for_the_previous_write() {
    let mut log = log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(original()),
    ]);
    assert_eq!(log.toggle_request(), Some(changed()));
    assert_eq!(log.toggle_request(), None);
    assert_eq!(log.requested, 1);
}

#[test]
fn a_toggle_from_sidetone_off_asks_for_level_one() {
    let mut log = log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(with_sidetone(0)),
    ]);
    assert_eq!(log.toggle_request(), Some(with_sidetone(1)));
}

#[test]
fn a_confirmed_toggle_is_followed_by_one_back_to_the_original() {
    let mut log = log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(original()),
    ]);
    assert_eq!(log.toggle_request(), Some(changed()));
    log.record(Duration::ZERO, DeviceUpdate::Observed(changed()));
    assert_eq!(log.toggle_request(), Some(original()));
    assert_eq!(log.confirmed, 1);
}

#[test]
fn a_toggle_needs_ready() {
    let mut log = log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(original()),
        DeviceUpdate::Phase(Phase::HeadsetOff),
    ]);
    assert_eq!(log.toggle_request(), None);
}
