use arctis_core::messages::device::{Phase, SyncPolicy};
use arctis_core::settings::headset_settings::Setting;
use arctis_core::testkit::FakeHeadset;

use crate::support::{SETTLED, observed, plugged, ready, sent, setter, with_sidetone};

const Q_TOO_NARROW: [u8; 6] = [0x20, 0x00, 0x01, 0x14, 0x64, 0x00];

fn holding_an_unreadable_curve() -> FakeHeadset {
    let mut headset = FakeHeadset::nova_5();
    headset.hold_unreadable_band(Q_TOO_NARROW);
    headset
}

#[test]
fn an_unreadable_curve_is_replaced_by_the_desired_one() {
    let mut simulation = plugged(holding_an_unreadable_curve());
    let desired = simulation.headset().settings().clone();
    simulation.desire(desired.clone(), SyncPolicy::Enforce);
    simulation.run_until(Phase::Ready, SETTLED);
    simulation.run_for(60_000);
    assert!(!simulation.published_phases().contains(&Phase::HeadsetOff));
    assert_eq!(simulation.phase(), Phase::Ready);
    assert_eq!(sent(&simulation, setter(Setting::Equalizer)), 1);
    assert_eq!(observed(&simulation), Some(desired));
}

#[test]
fn an_unreadable_curve_without_desired_settings_is_not_headset_off() {
    let mut simulation = plugged(holding_an_unreadable_curve());
    simulation.run_for(60_000);
    assert!(!simulation.published_phases().contains(&Phase::HeadsetOff));
    assert_eq!(simulation.phase(), Phase::Syncing);
}

#[test]
fn an_unreadable_readback_is_not_headset_off() {
    let mut simulation = ready();
    let desired = with_sidetone(simulation.headset().settings(), 6);
    simulation.headset_mut().hold_unreadable_band(Q_TOO_NARROW);
    simulation.clear_logs();
    simulation.desire(desired, SyncPolicy::Enforce);
    simulation.run_for(60_000);
    assert!(!simulation.published_phases().contains(&Phase::HeadsetOff));
    assert_eq!(simulation.phase(), Phase::Ready);
}
