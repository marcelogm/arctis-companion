use arctis_core::messages::device::{DeviceUpdate, SyncPolicy};
use arctis_core::session::Interface;
use arctis_core::session::timing::ECHO_WINDOW;
use arctis_core::settings::headset_settings::{HeadsetSettings, SettingChange};
use arctis_core::testkit::{Behaviour, FakeHeadset, Simulation};

use crate::support::{mic, ready_with, sent, sidetone, with_sidetone};

fn echoing(delay: u64, event_interface: Interface) -> Simulation {
    let behaviour = Behaviour {
        echo_delay: Some(delay),
        event_interface,
        ..Behaviour::default()
    };
    let mut simulation = ready_with(FakeHeadset::nova_5().with_behaviour(behaviour));
    simulation.clear_logs();
    simulation
}

fn headset_changes(simulation: &Simulation) -> Vec<SettingChange> {
    simulation
        .updates()
        .iter()
        .filter_map(|(_, update)| match update {
            DeviceUpdate::HeadsetChanged(change) => Some(change.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn an_echo_of_an_older_value_during_a_drag_is_consumed() {
    let mut simulation = echoing(300, Interface::Events);
    let base = simulation.headset().settings().clone();
    simulation.desire(with_sidetone(&base, 3), SyncPolicy::Enforce);
    simulation.run_until_sent(0x39, 1_000);
    simulation.run_for(100);
    let target = with_sidetone(&base, 6);
    simulation.desire(target.clone(), SyncPolicy::Enforce);
    simulation.run_for(2_000);
    assert!(headset_changes(&simulation).is_empty());
    assert_eq!(simulation.headset().settings(), &target);
}

#[test]
fn an_echo_on_the_command_interface_is_consumed() {
    let mut simulation = echoing(20, Interface::Command);
    let desired = with_sidetone(simulation.headset().settings(), 6);
    simulation.desire(desired.clone(), SyncPolicy::Enforce);
    simulation.run_for(2_000);
    assert!(headset_changes(&simulation).is_empty());
    assert_eq!(simulation.headset().settings(), &desired);
}

#[test]
fn an_echo_after_the_echo_window_is_taken_as_intent() {
    let mut simulation = echoing(ECHO_WINDOW + 500, Interface::Events);
    let desired = with_sidetone(simulation.headset().settings(), 6);
    simulation.desire(desired, SyncPolicy::Enforce);
    simulation.run_for(3_000);
    assert_eq!(
        headset_changes(&simulation),
        [SettingChange::Sidetone(sidetone(6))]
    );
}

#[test]
fn a_real_press_during_an_unrelated_write_becomes_headset_changed() {
    let mut simulation = echoing(20, Interface::Events);
    let desired = with_sidetone(simulation.headset().settings(), 6);
    simulation.desire(desired, SyncPolicy::Enforce);
    simulation.run_until_sent(0x39, 1_000);
    simulation.press(SettingChange::MicVolume(mic(7)));
    simulation.run_for(5_000);
    assert_eq!(
        headset_changes(&simulation),
        [SettingChange::MicVolume(mic(7))]
    );
    assert_eq!(sent(&simulation, 0x37), 0);
    assert_eq!(sent(&simulation, 0x39), 1);
    assert_eq!(simulation.headset().settings().mic_volume, mic(7));
}

#[test]
fn a_press_on_the_setting_being_written_wins() {
    let mut simulation = echoing(20, Interface::Events);
    let desired = with_sidetone(simulation.headset().settings(), 6);
    simulation.desire(desired, SyncPolicy::Enforce);
    simulation.run_until_sent(0x39, 1_000);
    simulation.press(SettingChange::Sidetone(sidetone(2)));
    simulation.run_for(5_000);
    assert_eq!(
        headset_changes(&simulation),
        [SettingChange::Sidetone(sidetone(2))]
    );
    assert_eq!(sent(&simulation, 0x39), 1);
    assert_eq!(simulation.headset().settings().sidetone, sidetone(2));
}

#[test]
fn a_press_on_a_setting_not_yet_written_cancels_its_write() {
    let mut simulation = echoing(20, Interface::Events);
    let desired = HeadsetSettings {
        mic_volume: mic(7),
        ..with_sidetone(simulation.headset().settings(), 6)
    };
    simulation.desire(desired, SyncPolicy::Enforce);
    simulation.run_until_sent(0x37, 1_000);
    simulation.press(SettingChange::Sidetone(sidetone(2)));
    simulation.run_for(2_000);
    assert_eq!(sent(&simulation, 0x39), 0);
    assert_eq!(simulation.headset().settings().sidetone, sidetone(2));
}
