use arctis_core::messages::device::Phase;
use arctis_core::protocol::Query;
use arctis_core::protocol::status::Link;
use arctis_core::session::Interface;
use arctis_core::session::timing::{REPLY_TIMEOUT, WIRELESS_MODE_GRACE};
use arctis_core::settings::headset_settings::{HeadsetSettings, Setting};
use arctis_core::settings::values::WirelessMode;
use arctis_core::testkit::{Behaviour, Emission, Flap, Simulation};

use crate::support::{desire, observed, ready_behaving, setter};

fn extended_range_written() -> (Simulation, HeadsetSettings) {
    let mut simulation = ready_behaving(Behaviour::default());
    let desired = HeadsetSettings {
        wireless_mode: WirelessMode::ExtendedRange,
        ..simulation.headset().settings().clone()
    };
    desire(&mut simulation, &desired);
    simulation.run_until_sent(setter(Setting::WirelessMode), 1_000);
    (simulation, desired)
}

#[test]
fn a_link_drop_within_the_grace_waits_for_the_link_to_return() {
    let (mut simulation, desired) = extended_range_written();
    simulation.link(Link::HeadsetOff);
    simulation.run_for(2_000);
    assert_eq!(simulation.sent_ids(), [setter(Setting::WirelessMode)]);
    simulation.link(Link::Connected);
    simulation.run_for(1_000);
    assert_eq!(simulation.phase(), Phase::Ready);
    assert_eq!(
        simulation.sent_ids(),
        [setter(Setting::WirelessMode), Query::Wireless.id()]
    );
    assert_eq!(observed(&simulation), Some(desired));
}

#[test]
fn a_link_still_down_when_the_grace_ends_means_headset_off() {
    let (mut simulation, _) = extended_range_written();
    simulation.link(Link::HeadsetOff);
    simulation.run_for(WIRELESS_MODE_GRACE - 20);
    assert_eq!(simulation.phase(), Phase::Ready);
    simulation.run_for(20);
    assert_eq!(simulation.phase(), Phase::HeadsetOff);
}

#[test]
fn silence_within_the_grace_is_tolerated() {
    let (mut simulation, desired) = extended_range_written();
    simulation.headset_mut().behaviour.reply_latency = None;
    simulation.run_for(WIRELESS_MODE_GRACE - 100);
    assert_eq!(simulation.phase(), Phase::Ready);
    simulation.headset_mut().behaviour.reply_latency = Some(1);
    simulation.run_for(2_000);
    assert_eq!(simulation.phase(), Phase::Ready);
    assert_eq!(observed(&simulation), Some(desired));
}

#[test]
fn a_link_drop_at_the_end_of_the_grace_means_headset_off() {
    let (mut simulation, _) = extended_range_written();
    simulation.headset_mut().behaviour.reply_latency = None;
    simulation.run_for(WIRELESS_MODE_GRACE);
    assert_eq!(simulation.phase(), Phase::Ready);
    simulation.link(Link::HeadsetOff);
    simulation.run_for(1);
    assert_eq!(simulation.phase(), Phase::HeadsetOff);
}

#[test]
fn a_readback_asked_as_the_grace_ends_is_no_longer_tolerated() {
    let (mut simulation, _) = extended_range_written();
    simulation.link(Link::HeadsetOff);
    simulation.headset_mut().behaviour.reply_latency = None;
    simulation.run_for(WIRELESS_MODE_GRACE - 1);
    simulation.emit(vec![Emission {
        at: simulation.now().after(1),
        interface: Interface::Events,
        bytes: vec![0xB9, Link::Connected.raw()],
    }]);
    simulation.run_for(1 + 2 * REPLY_TIMEOUT);
    assert_eq!(simulation.phase(), Phase::HeadsetOff);
}

fn grace_ends_at(simulation: &Simulation) -> u64 {
    let (written_at, _) = simulation.writes()[0];
    written_at
        .after(WIRELESS_MODE_GRACE)
        .since(simulation.now())
}

#[test]
fn a_link_drop_after_the_confirming_readback_waits_for_the_link() {
    let (mut simulation, desired) = extended_range_written();
    simulation.run_for(300);
    simulation.link(Link::HeadsetOff);
    simulation.run_for(1_000);
    assert_eq!(simulation.phase(), Phase::Ready);
    simulation.link(Link::Connected);
    simulation.run_for(2 * WIRELESS_MODE_GRACE);
    assert_eq!(simulation.phase(), Phase::Ready);
    assert_eq!(observed(&simulation), Some(desired));
}

#[test]
fn a_link_down_after_the_confirming_readback_means_headset_off_when_the_grace_ends() {
    let (mut simulation, _) = extended_range_written();
    simulation.run_for(300);
    simulation.link(Link::HeadsetOff);
    let remaining = grace_ends_at(&simulation);
    simulation.run_for(remaining - 1);
    assert_eq!(simulation.phase(), Phase::Ready);
    simulation.run_for(1);
    assert_eq!(simulation.phase(), Phase::HeadsetOff);
}

#[test]
fn a_rebind_within_the_grace_keeps_the_grace() {
    let (mut simulation, desired) = extended_range_written();
    simulation.flap(Flap {
        node_back_after: 60,
        ..Flap::default()
    });
    simulation.link(Link::HeadsetOff);
    simulation.run_for(1_000);
    assert_eq!(simulation.phase(), Phase::Ready);
    simulation.link(Link::Connected);
    simulation.run_for(2 * WIRELESS_MODE_GRACE);
    assert_eq!(simulation.phase(), Phase::Ready);
    assert_eq!(observed(&simulation), Some(desired));
    assert!(!simulation.published_phases().contains(&Phase::HeadsetOff));
}
