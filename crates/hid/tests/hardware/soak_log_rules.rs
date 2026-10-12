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
    assert_eq!(log.restore_request(Duration::ZERO), Some(original()));
    assert_eq!(log.requested, 1);
    assert!(!log.unrestored);
    log.record(Duration::ZERO, DeviceUpdate::Observed(original()));
    assert_eq!(log.confirmed, 1);
}

#[test]
fn a_restore_with_the_headset_off_is_left_for_a_person() {
    let mut log = changed_then(Phase::HeadsetOff);
    assert_eq!(log.restore_request(Duration::ZERO), None);
    assert_eq!(log.requested, 0);
    assert!(log.unrestored);
}

#[test]
fn unchanged_settings_need_no_restore() {
    let mut log = log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(original()),
    ]);
    assert_eq!(log.restore_request(Duration::ZERO), None);
    assert!(!log.unrestored);
}

#[test]
fn a_toggle_waits_for_the_previous_write() {
    let mut log = log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(original()),
    ]);
    assert_eq!(log.toggle_request(Duration::ZERO), Some(changed()));
    assert_eq!(log.toggle_request(Duration::ZERO), None);
    assert_eq!(log.requested, 1);
}

#[test]
fn a_toggle_from_sidetone_off_asks_for_level_one() {
    let mut log = log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(with_sidetone(0)),
    ]);
    assert_eq!(log.toggle_request(Duration::ZERO), Some(with_sidetone(1)));
}

#[test]
fn a_confirmed_toggle_is_followed_by_one_back_to_the_original() {
    let mut log = log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(original()),
    ]);
    assert_eq!(log.toggle_request(Duration::ZERO), Some(changed()));
    log.record(Duration::ZERO, DeviceUpdate::Observed(changed()));
    assert_eq!(log.toggle_request(Duration::ZERO), Some(original()));
    assert_eq!(log.confirmed, 1);
}

#[test]
fn a_toggle_needs_ready() {
    let mut log = log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(original()),
        DeviceUpdate::Phase(Phase::HeadsetOff),
    ]);
    assert_eq!(log.toggle_request(Duration::ZERO), None);
}

#[test]
fn the_slowest_confirmation_is_measured_from_the_request() {
    let mut log = log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(original()),
    ]);
    let first = log
        .toggle_request(Duration::from_millis(1_000))
        .expect("a toggle");
    log.record(Duration::from_millis(1_120), DeviceUpdate::Observed(first));
    let second = log
        .toggle_request(Duration::from_millis(2_000))
        .expect("a toggle");
    log.record(Duration::from_millis(2_250), DeviceUpdate::Observed(second));
    assert_eq!(log.slowest_confirmation(), Some(Duration::from_millis(250)));
}

#[test]
fn an_interrupted_write_has_no_latency() {
    let mut log = log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(original()),
    ]);
    let _ = log.toggle_request(Duration::from_millis(1_000));
    log.record(
        Duration::from_millis(1_050),
        DeviceUpdate::Phase(Phase::Rebinding),
    );
    assert_eq!(log.slowest_confirmation(), None);
}
