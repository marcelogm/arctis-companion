use arctis_core::messages::device::{Phase, SyncPolicy};
use arctis_core::session::timing::VERIFY_BACKOFF;
use arctis_core::settings::headset_settings::{HeadsetSettings, Setting, SettingChange};
use arctis_core::testkit::{Behaviour, FakeHeadset, Simulation};

use crate::support::{
    SETTLED, headset_off, mic, observed, plugged, ready, sent, setter, sidetone, with_sidetone,
};

fn setter_writes(simulation: &Simulation) -> usize {
    Setting::ALL
        .into_iter()
        .map(|setting| sent(simulation, setter(setting)))
        .sum()
}

#[test]
fn without_desired_settings_the_session_never_writes_a_setter() {
    let mut simulation = ready();
    simulation.run_for(900_000);
    assert_eq!(setter_writes(&simulation), 0);
}

#[test]
fn adopt_writes_nothing_and_publishes_observed() {
    let mut simulation = ready();
    let headset = simulation.headset().settings().clone();
    simulation.clear_logs();
    simulation.desire(with_sidetone(&headset, 6), SyncPolicy::Adopt);
    simulation.run_for(60_000);
    assert_eq!(setter_writes(&simulation), 0);
    assert_eq!(observed(&simulation), Some(headset));
}

#[test]
fn adopt_during_a_transaction_stops_its_remaining_writes() {
    let mut simulation = ready();
    let desired = HeadsetSettings {
        mic_volume: mic(7),
        ..with_sidetone(simulation.headset().settings(), 6)
    };
    simulation.clear_logs();
    simulation.desire(desired.clone(), SyncPolicy::Enforce);
    simulation.run_until_sent(setter(Setting::MicVolume), 1_000);
    simulation.desire(desired, SyncPolicy::Adopt);
    simulation.run_for(60_000);
    assert_eq!(setter_writes(&simulation), 1);
}

#[test]
fn enforce_writes_only_the_drifted_fields() {
    let mut simulation = ready();
    let desired = with_sidetone(simulation.headset().settings(), 6);
    simulation.clear_logs();
    simulation.desire(desired, SyncPolicy::Enforce);
    simulation.run_for(1_000);
    assert_eq!(setter_writes(&simulation), 1);
    assert_eq!(sent(&simulation, setter(Setting::Sidetone)), 1);
    assert_eq!(simulation.phase(), Phase::Ready);
}

#[test]
fn desired_settings_arriving_after_probing_are_applied_before_ready() {
    let mut simulation = plugged(FakeHeadset::nova_5());
    simulation.run_until(Phase::Syncing, SETTLED);
    let desired = with_sidetone(simulation.headset().settings(), 6);
    simulation.desire(desired.clone(), SyncPolicy::Enforce);
    simulation.run_until(Phase::Ready, SETTLED);
    let ready_at = simulation.now();
    let sidetone_at = simulation
        .writes()
        .iter()
        .find(|(_, frame)| frame.payload()[0] == setter(Setting::Sidetone))
        .map(|(at, _)| *at);
    assert!(sidetone_at.is_some_and(|at| at < ready_at));
    assert_eq!(simulation.headset().settings(), &desired);
}

#[test]
fn a_reconcile_while_syncing_waits_through_the_verify_backoff() {
    let slow = Behaviour {
        readback_latency: VERIFY_BACKOFF[2] - 10,
        ..Behaviour::default()
    };
    let mut simulation = plugged(FakeHeadset::nova_5().with_behaviour(slow));
    simulation.run_until(Phase::Syncing, SETTLED);
    let desired = with_sidetone(simulation.headset().settings(), 6);
    simulation.desire(desired.clone(), SyncPolicy::Enforce);
    simulation.run_until(Phase::Ready, SETTLED);
    assert_eq!(sent(&simulation, setter(Setting::Sidetone)), 1);
    assert_eq!(simulation.headset().settings(), &desired);
}

#[test]
fn desired_changes_while_the_headset_is_off_apply_on_the_next_syncing() {
    let mut simulation = headset_off();
    let desired = with_sidetone(simulation.headset().settings(), 6);
    simulation.desire(desired.clone(), SyncPolicy::Enforce);
    simulation.run_for(30_000);
    assert_eq!(setter_writes(&simulation), 0);
    simulation.power_on();
    simulation.run_until(Phase::Ready, SETTLED);
    assert_eq!(simulation.headset().settings(), &desired);
}

#[test]
fn a_reconnect_writes_only_the_fields_that_drifted() {
    let mut simulation = ready();
    let desired = with_sidetone(simulation.headset().settings(), 6);
    simulation.desire(desired, SyncPolicy::Enforce);
    simulation.run_for(1_000);
    simulation.power_off();
    simulation.run_for(100);
    simulation
        .headset_mut()
        .set(SettingChange::MicVolume(mic(3)));
    simulation.clear_logs();
    simulation.power_on();
    simulation.run_until(Phase::Ready, SETTLED);
    assert_eq!(setter_writes(&simulation), 1);
    assert_eq!(sent(&simulation, setter(Setting::MicVolume)), 1);
    assert_eq!(simulation.headset().settings().sidetone, sidetone(6));
}

#[test]
fn an_unplugged_dongle_forgets_the_desired_settings() {
    let mut simulation = ready();
    let desired = with_sidetone(simulation.headset().settings(), 6);
    simulation.desire(desired, SyncPolicy::Enforce);
    simulation.run_for(1_000);
    simulation.unplug();
    simulation
        .headset_mut()
        .set(SettingChange::Sidetone(sidetone(1)));
    simulation.clear_logs();
    simulation.plug();
    simulation.run_until(Phase::Ready, SETTLED);
    assert_eq!(setter_writes(&simulation), 0);
}

#[test]
fn a_setting_changed_back_before_its_readback_is_written_back() {
    let mut simulation = ready();
    let headset = simulation.headset().settings().clone();
    simulation.desire(with_sidetone(&headset, 6), SyncPolicy::Enforce);
    simulation.run_until_sent(setter(Setting::Sidetone), 1_000);
    simulation.desire(headset.clone(), SyncPolicy::Enforce);
    simulation.run_for(1_000);
    assert_eq!(simulation.headset().settings(), &headset);
    assert_eq!(observed(&simulation), Some(headset));
    assert_eq!(simulation.phase(), Phase::Ready);
}
