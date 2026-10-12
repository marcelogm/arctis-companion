use arctis_core::companion::timing::QUIT_GRACE;
use arctis_core::companion::{Action, Input};
use arctis_core::messages::audio::{AudioCommand, SinkKind};
use arctis_core::messages::device::{DeviceUpdate, Phase};
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::settings::headset_settings::Setting;

use crate::support::{
    Harness, START, audio_commands, curve_of, default_sink, device_changes, device_curves,
    firefox_playing, percent, persisted, ready, wakes,
};

#[test]
fn quitting_closes_the_battery_line() {
    let mut h = ready();
    let status = DeviceUpdate::Status {
        battery: percent(50),
        charging: Charging::Discharging,
        link: Link::Connected,
    };
    let _ = h.device(status);
    let quitting = h.at(1_000).send(Input::Quitting);
    let closing = |action: &Action| matches!(action, Action::RecordBattery(sample) if !sample.reading.is_connected());
    assert!(quitting.iter().any(closing));
}

#[test]
fn quitting_with_nothing_to_restore_is_immediate() {
    let mut h = ready();
    let actions = h.send(Input::Quitting);
    assert!(actions.contains(&Action::ReadyToQuit));
    assert!(device_changes(&actions).is_empty());
}

#[test]
fn quitting_restores_the_route_preset_and_waits_for_its_verify() {
    let mut h = firefox_playing();
    let quitting = h.at(40_000).send(Input::Quitting);
    assert_eq!(device_curves(&quitting), [curve_of("flat")]);
    assert_eq!(wakes(&quitting), [START.after(40_000 + QUIT_GRACE)]);
    assert!(!quitting.contains(&Action::ReadyToQuit));
    assert!(
        h.at(40_200)
            .headset_holds("flat")
            .contains(&Action::ReadyToQuit)
    );
}

#[test]
fn quitting_gives_up_after_the_grace() {
    let mut h = firefox_playing();
    let _ = h.at(40_000).send(Input::Quitting);
    assert!(
        !h.at(40_999)
            .send(Input::Woke)
            .contains(&Action::ReadyToQuit)
    );
    assert!(
        h.at(40_000 + QUIT_GRACE)
            .send(Input::Woke)
            .contains(&Action::ReadyToQuit)
    );
}

#[test]
fn a_failed_restore_does_not_hold_the_quit() {
    let mut h = firefox_playing();
    let _ = h.at(40_000).send(Input::Quitting);
    let failed = h
        .at(40_500)
        .device(DeviceUpdate::WriteFailed(Setting::Equalizer));
    assert!(failed.contains(&Action::ReadyToQuit));
}

#[test]
fn quitting_while_the_headset_is_off_does_not_wait() {
    let mut h = firefox_playing();
    let _ = h.device(DeviceUpdate::Phase(Phase::HeadsetOff));
    assert!(
        h.at(40_000)
            .send(Input::Quitting)
            .contains(&Action::ReadyToQuit)
    );
}

#[test]
fn ready_to_quit_is_sent_once() {
    let mut h = ready();
    let _ = h.send(Input::Quitting);
    assert!(!h.at(5_000).send(Input::Woke).contains(&Action::ReadyToQuit));
}

#[test]
fn quitting_removes_the_system_equalizer_for_good() {
    let mut h = Harness::new(persisted());
    let _ = h.audio(default_sink(SinkKind::NovaBluetooth));
    let quitting = h.send(Input::Quitting);
    assert_eq!(audio_commands(&quitting), [AudioCommand::RemoveEqualizer]);
    assert!(quitting.contains(&Action::ReadyToQuit));
    assert!(audio_commands(&h.audio(default_sink(SinkKind::NovaBluetooth))).is_empty());
}
