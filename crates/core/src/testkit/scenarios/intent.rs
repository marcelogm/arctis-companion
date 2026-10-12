use super::{headset_changes, headset_off, observed, ready, setter, sidetone, with_sidetone};
use crate::messages::device::{Phase, SyncPolicy};
use crate::protocol::status::Link;
use crate::session::Interface;
use crate::settings::headset_settings::{Setting, SettingChange};
use crate::testkit::fake_headset::Behaviour;
use crate::testkit::rig::Rig;

pub fn an_echo_on_the_events_interface_is_consumed<R: Rig>() {
    assert_echo_consumed::<R>(Interface::Events);
}

pub fn an_echo_on_the_command_interface_is_consumed<R: Rig>() {
    assert_echo_consumed::<R>(Interface::Command);
}

fn assert_echo_consumed<R: Rig>(event_interface: Interface) {
    let mut rig = ready::<R>(Behaviour {
        echo_delay: Some(20),
        event_interface,
        ..Behaviour::default()
    });
    let desired = with_sidetone(rig.headset().settings(), 6);
    rig.desire(desired.clone(), SyncPolicy::Enforce);
    rig.run_for(2_000);
    assert!(headset_changes(&rig).is_empty());
    assert_eq!(rig.headset().settings(), &desired);
}

pub fn a_press_becomes_headset_changed<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    rig.press(SettingChange::Sidetone(sidetone(3)));
    rig.run_for(100);
    assert_eq!(
        headset_changes(&rig),
        [SettingChange::Sidetone(sidetone(3))]
    );
}

pub fn without_desired_settings_nothing_is_written<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    rig.run_for(60_000);
    assert!(rig.writes().is_empty());
}

pub fn adopt_writes_nothing_and_publishes_observed<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    let desired = with_sidetone(rig.headset().settings(), 6);
    rig.desire(desired, SyncPolicy::Adopt);
    rig.run_for(1_000);
    assert!(rig.writes().is_empty());
    assert_eq!(observed(&rig).as_ref(), Some(rig.headset().settings()));
}

pub fn changes_made_while_the_headset_is_off_apply_when_it_turns_on<R: Rig>() {
    let mut rig = headset_off::<R>(Behaviour::default());
    let desired = with_sidetone(rig.headset().settings(), 6);
    rig.desire(desired.clone(), SyncPolicy::Enforce);
    rig.run_for(1_000);
    assert!(!rig.sent_ids().contains(&setter(Setting::Sidetone)));
    rig.link(Link::Connected);
    rig.run_until(Phase::Ready, 3_000);
    assert_eq!(rig.headset().settings(), &desired);
}
