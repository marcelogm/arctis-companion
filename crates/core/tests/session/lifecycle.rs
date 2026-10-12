use arctis_core::messages::device::{DeviceUpdate, Phase};
use arctis_core::protocol::firmware::{Firmware, Version};
use arctis_core::protocol::model::{Model, ModelLookup, Support};
use arctis_core::session::timing::{
    COMMAND_SPACING, ENUMERATION_SETTLE, HEADSET_OFF_POLL, REPLY_TIMEOUT, STATUS_SAFETY_POLL,
};
use arctis_core::session::{Cadence, Interface, Session};
use arctis_core::testkit::{Behaviour, Emission, FakeHeadset, Simulation};
use arctis_core::time::Millis;

use crate::support::{
    Arrow, SETTLED, absent, arrow, assert_arrows, headset_off, percent, plugged, probing,
    probing_silent, ready, ready_with, settling, silent, syncing, syncing_silent,
};

const UNREAD_FIRMWARE: Firmware = Firmware {
    receiver: None,
    headset: None,
};

const ARROWS: [Arrow; 16] = [
    arrow(
        "Absent -> PermissionDenied: open EACCES",
        absent,
        Simulation::deny,
        Phase::PermissionDenied,
    ),
    arrow(
        "Absent -> Settling: both interfaces opened",
        absent,
        Simulation::plug,
        Phase::Settling,
    ),
    arrow(
        "PermissionDenied -> Settling: open succeeds on rescan",
        permission_denied,
        Simulation::plug,
        Phase::Settling,
    ),
    arrow(
        "PermissionDenied -> Absent: dongle gone",
        permission_denied,
        Simulation::unplug,
        Phase::Absent,
    ),
    arrow(
        "Settling -> Probing: ENUMERATION_SETTLE",
        settling,
        run_settle,
        Phase::Probing,
    ),
    arrow(
        "Probing -> HeadsetOff: B0 link != 3",
        probing_headset_off,
        run_reply,
        Phase::HeadsetOff,
    ),
    arrow(
        "Probing -> HeadsetOff: B0 unanswered twice",
        probing_silent,
        run_two_timeouts,
        Phase::HeadsetOff,
    ),
    arrow(
        "Probing -> Syncing: B0 link == 3",
        probing,
        run_reply,
        Phase::Syncing,
    ),
    arrow(
        "HeadsetOff -> Syncing: event B9 link 3",
        headset_off,
        power_on_with_event,
        Phase::Syncing,
    ),
    arrow(
        "HeadsetOff -> Syncing: HEADSET_OFF_POLL B0 link 3",
        headset_off_without_events,
        power_on_silently,
        Phase::Syncing,
    ),
    arrow(
        "Syncing -> Ready: replies decoded, reconcile verified",
        syncing,
        run_sync,
        Phase::Ready,
    ),
    arrow(
        "Syncing -> HeadsetOff: query unanswered twice",
        syncing_silent,
        run_two_timeouts,
        Phase::HeadsetOff,
    ),
    arrow(
        "Ready -> HeadsetOff: event B9 link != 3",
        ready,
        power_off_with_event,
        Phase::HeadsetOff,
    ),
    arrow(
        "Ready -> HeadsetOff: query unanswered twice",
        ready,
        silence_then_safety_poll,
        Phase::HeadsetOff,
    ),
    arrow("Ready -> Ready: events", ready, battery_event, Phase::Ready),
    arrow(
        "Present -> Absent: Lost",
        ready,
        Simulation::unplug,
        Phase::Absent,
    ),
];

#[test]
fn a_new_session_is_absent() {
    assert_eq!(Session::new().phase(), Phase::Absent);
}

#[test]
fn every_arrow_without_rebinding_holds() {
    assert_arrows(&ARROWS);
}

#[test]
fn the_identity_is_published_once_the_firmware_is_read() {
    let simulation = ready();
    let ModelLookup::Supported(model) = Model::lookup(0x2232) else {
        panic!("the Nova 5 is supported");
    };
    let identity = DeviceUpdate::Identity {
        model,
        uniq: None,
        firmware: Firmware {
            receiver: Some(Version {
                major: 1,
                minor: 16,
                patch: 0,
            }),
            headset: Some(Version {
                major: 1,
                minor: 7,
                patch: 0,
            }),
        },
    };
    assert!(
        simulation
            .updates()
            .iter()
            .any(|(_, update)| *update == identity)
    );
}

#[test]
fn a_bootloader_dongle_is_never_opened() {
    let mut simulation = plugged(FakeHeadset::nova_5().with_pid(0x2233));
    simulation.run_for(2 * ENUMERATION_SETTLE);
    assert_eq!(simulation.phase(), Phase::Absent);
    assert!(simulation.writes().is_empty());
    assert!(simulation.updates().is_empty());
    assert!(simulation.rescans().is_empty());
}

#[test]
fn a_found_dongle_stops_the_rescan_and_a_lost_one_restarts_it() {
    let mut simulation = settling();
    simulation.unplug();
    assert_eq!(simulation.rescans(), [Cadence::Off, Cadence::Absent]);
}

#[test]
fn lost_from_every_present_phase_reaches_absent() {
    let setups: [fn() -> Simulation; 5] = [settling, probing, syncing, headset_off, ready];
    for setup in setups {
        let mut simulation = setup();
        let before = simulation.phase();
        simulation.unplug();
        assert_eq!(simulation.phase(), Phase::Absent, "from {before:?}");
        assert_eq!(
            simulation.rescans().last(),
            Some(&Cadence::Absent),
            "from {before:?}"
        );
        assert_eq!(
            simulation.published_phases().last(),
            Some(&Phase::Absent),
            "from {before:?}"
        );
    }
}

#[test]
fn a_command_frame_during_the_enumeration_settle_is_dropped() {
    let mut simulation = settling();
    simulation.clear_logs();
    simulation.emit(vec![Emission {
        at: Millis(0),
        interface: Interface::Command,
        bytes: vec![0x39, 0x05],
    }]);
    simulation.run_for(1);
    assert!(simulation.updates().is_empty());
}

#[test]
fn a_dongle_is_identified_while_the_headset_is_off() {
    let identities = identities(&headset_off());
    assert_eq!(identities, [(0x2232, Support::Verified, UNREAD_FIRMWARE)]);
}

#[test]
fn an_untested_model_is_opened_and_identified_as_untested() {
    let identities = identities(&ready_with(FakeHeadset::nova_5().with_pid(0x2255)));
    assert!(!identities.is_empty());
    assert!(
        identities
            .iter()
            .all(|&(pid, support, _)| pid == 0x2255 && support == Support::Untested)
    );
}

fn identities(simulation: &Simulation) -> Vec<(u16, Support, Firmware)> {
    simulation
        .updates()
        .iter()
        .filter_map(|(_, update)| match update {
            DeviceUpdate::Identity {
                model, firmware, ..
            } => Some((model.pid, model.support, *firmware)),
            _ => None,
        })
        .collect()
}

fn permission_denied() -> Simulation {
    let mut simulation = absent();
    simulation.deny();
    simulation
}

fn probing_headset_off() -> Simulation {
    let mut simulation = plugged(FakeHeadset::nova_5().powered_off());
    simulation.run_for(ENUMERATION_SETTLE);
    simulation
}

fn headset_off_without_events() -> Simulation {
    let quiet = Behaviour {
        sends_events: false,
        ..Behaviour::default()
    };
    let mut simulation = plugged(FakeHeadset::nova_5().powered_off().with_behaviour(quiet));
    simulation.run_until(Phase::HeadsetOff, SETTLED);
    simulation
}

fn run_settle(simulation: &mut Simulation) {
    simulation.run_for(ENUMERATION_SETTLE);
}

fn run_reply(simulation: &mut Simulation) {
    simulation.run_for(1);
}

fn run_two_timeouts(simulation: &mut Simulation) {
    simulation.run_for(2 * REPLY_TIMEOUT + COMMAND_SPACING);
}

fn run_sync(simulation: &mut Simulation) {
    simulation.run_for(1_000);
}

fn power_on_with_event(simulation: &mut Simulation) {
    simulation.power_on();
    simulation.run_for(100);
}

fn power_on_silently(simulation: &mut Simulation) {
    simulation.power_on();
    simulation.run_for(HEADSET_OFF_POLL + 10);
}

fn power_off_with_event(simulation: &mut Simulation) {
    simulation.power_off();
    simulation.run_for(10);
}

fn silence_then_safety_poll(simulation: &mut Simulation) {
    simulation.headset_mut().behaviour = silent();
    simulation.run_for(STATUS_SAFETY_POLL + 2 * REPLY_TIMEOUT);
}

fn battery_event(simulation: &mut Simulation) {
    simulation.battery(percent(50));
    simulation.run_for(10);
}
