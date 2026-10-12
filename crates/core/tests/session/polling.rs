use arctis_core::messages::device::{DeviceUpdate, Phase, SyncPolicy};
use arctis_core::protocol::Query;
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::session::timing::{
    COMMAND_SPACING, ENUMERATION_SETTLE, HEADSET_OFF_POLL, REPLY_TIMEOUT, STATUS_SAFETY_POLL,
};
use arctis_core::settings::headset_settings::Setting;
use arctis_core::testkit::{Behaviour, FakeHeadset};

use crate::support::{
    SETTLED, headset_off, percent, plugged, quiet, ready, ready_with, sent, setter, silent,
    with_sidetone,
};

#[test]
fn an_unanswered_query_is_resent_once_before_headset_off() {
    let mut simulation = plugged(FakeHeadset::nova_5().with_behaviour(silent()));
    simulation.run_until(Phase::HeadsetOff, SETTLED);
    let times: Vec<_> = simulation.writes().iter().map(|(at, _)| at.0).collect();
    assert_eq!(
        times,
        [ENUMERATION_SETTLE, ENUMERATION_SETTLE + REPLY_TIMEOUT]
    );
    assert_eq!(simulation.now().0, ENUMERATION_SETTLE + 2 * REPLY_TIMEOUT);
}

#[test]
fn a_headset_without_events_is_synced_within_headset_off_poll_of_powering_on() {
    let mut simulation = plugged(FakeHeadset::nova_5().powered_off().with_behaviour(quiet()));
    simulation.run_until(Phase::HeadsetOff, SETTLED);
    simulation.power_on();
    let waited = simulation.run_until(Phase::Syncing, HEADSET_OFF_POLL + COMMAND_SPACING);
    assert!(waited <= HEADSET_OFF_POLL + 1);
}

#[test]
fn headset_off_polls_the_status_every_headset_off_poll() {
    let mut simulation = headset_off();
    simulation.clear_logs();
    simulation.run_for(6 * HEADSET_OFF_POLL);
    assert_eq!(simulation.sent_ids(), [Query::Status.id(); 6]);
}

#[test]
fn ready_polls_the_status_only_after_status_safety_poll() {
    let mut simulation = ready();
    simulation.clear_logs();
    simulation.run_for(STATUS_SAFETY_POLL - 1_000);
    assert_eq!(sent(&simulation, Query::Status.id()), 0);
    simulation.run_for(STATUS_SAFETY_POLL);
    assert_eq!(simulation.sent_ids(), [Query::Status.id()]);
}

#[test]
fn battery_events_postpone_the_safety_poll() {
    let mut simulation = ready();
    simulation.clear_logs();
    for _ in 0..4 {
        simulation.run_for(STATUS_SAFETY_POLL - 60_000);
        simulation.battery(percent(40));
    }
    assert_eq!(sent(&simulation, Query::Status.id()), 0);
}

#[test]
fn status_events_are_published_with_the_last_known_status() {
    let mut simulation = ready();
    simulation.clear_logs();
    simulation.battery(percent(50));
    simulation.run_for(1);
    let expected = DeviceUpdate::Status {
        battery: percent(50),
        charging: Charging::Discharging,
        link: Link::Connected,
    };
    assert_eq!(
        simulation.updates().last().map(|(_, update)| update),
        Some(&expected)
    );
}

#[test]
fn the_safety_poll_waits_for_the_write_transaction() {
    let lossy = Behaviour {
        drops_writes: true,
        ..Behaviour::default()
    };
    let mut simulation = ready_with(FakeHeadset::nova_5().with_behaviour(lossy));
    simulation.run_for(STATUS_SAFETY_POLL - 1_000);
    simulation.clear_logs();
    let desired = with_sidetone(simulation.headset().settings(), 6);
    simulation.desire(desired, SyncPolicy::Enforce);
    simulation.run_for(5_000);
    let ids = simulation.sent_ids();
    assert_eq!(ids.last(), Some(&Query::Status.id()));
    assert_eq!(sent(&simulation, Query::Status.id()), 1);
    assert_eq!(sent(&simulation, setter(Setting::Sidetone)), 2);
}
