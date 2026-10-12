use arctis_core::messages::device::{DeviceUpdate, Phase, SyncPolicy};
use arctis_core::protocol::Query;
use arctis_core::session::timing::{
    ENUMERATION_SETTLE, HEADSET_OFF_POLL, REBIND_GIVE_UP, REBIND_NOTICE, REBIND_SETTLE,
    RESCAN_REBINDING,
};
use arctis_core::session::{Cadence, Interface};
use arctis_core::settings::headset_settings::Setting;
use arctis_core::testkit::{Emission, FakeHeadset, Flap, Simulation};

use crate::support::{
    Arrow, SETTLED, arrow, assert_arrows, headset_off, observed, percent, probing, probing_silent,
    published, ready, sent, setter, settling, syncing, syncing_silent, with_sidetone,
};

const QUICK_FLAP: Flap = Flap {
    node_back_after: 60,
    unreadable_for: 0,
    stale_frame: false,
};
const FLAP_DONE: u64 = 60 + RESCAN_REBINDING + REBIND_SETTLE + 50;
const FLAP_PERIOD: u64 = 20_000;
const TEN_MINUTES: u64 = 600_000;

const ARROWS: [Arrow; 13] = [
    arrow(
        "Settling -> Rebinding: iface 3 EIO",
        settling,
        lose_command_port,
        Phase::Rebinding,
    ),
    arrow(
        "Probing -> Rebinding: iface 3 EIO",
        probing_silent,
        lose_command_port,
        Phase::Rebinding,
    ),
    arrow(
        "Syncing -> Rebinding: iface 3 EIO",
        syncing,
        lose_command_port,
        Phase::Rebinding,
    ),
    arrow(
        "Ready -> Rebinding: iface 3 EIO",
        ready,
        lose_command_port,
        Phase::Rebinding,
    ),
    arrow(
        "HeadsetOff -> Rebinding: iface 3 EIO",
        headset_off,
        lose_command_port,
        Phase::Rebinding,
    ),
    arrow(
        "Rebinding -> Rebinding: rescan finds no node",
        ready,
        node_stays_away,
        Phase::Rebinding,
    ),
    arrow(
        "Rebinding -> Rebinding: open EACCES",
        ready,
        node_stays_unreadable,
        Phase::Rebinding,
    ),
    arrow(
        "Rebinding -> Settling: reopened + REBIND_SETTLE",
        settling,
        quick_flap,
        Phase::Settling,
    ),
    arrow(
        "Rebinding -> Probing: reopened + REBIND_SETTLE",
        probing_silent,
        quick_flap,
        Phase::Probing,
    ),
    arrow(
        "Rebinding -> Syncing: reopened + REBIND_SETTLE",
        syncing_silent,
        quick_flap,
        Phase::Syncing,
    ),
    arrow(
        "Rebinding -> Ready: reopened + REBIND_SETTLE",
        ready,
        quick_flap,
        Phase::Ready,
    ),
    arrow(
        "Rebinding -> HeadsetOff: reopened + REBIND_SETTLE",
        headset_off,
        quick_flap,
        Phase::HeadsetOff,
    ),
    arrow(
        "Present -> Absent: Rebinding longer than REBIND_GIVE_UP",
        ready,
        give_up,
        Phase::Absent,
    ),
];

fn lose_command_port(simulation: &mut Simulation) {
    simulation.flap(QUICK_FLAP);
}

fn quick_flap(simulation: &mut Simulation) {
    simulation.flap(QUICK_FLAP);
    simulation.run_for(FLAP_DONE);
}

fn node_stays_away(simulation: &mut Simulation) {
    simulation.remove_command_node();
    simulation.run_for(REBIND_GIVE_UP - 1_000);
}

fn node_stays_unreadable(simulation: &mut Simulation) {
    simulation.flap(Flap {
        unreadable_for: REBIND_GIVE_UP,
        ..QUICK_FLAP
    });
    simulation.run_for(REBIND_GIVE_UP - 1_000);
}

fn give_up(simulation: &mut Simulation) {
    simulation.remove_command_node();
    simulation.run_for(REBIND_GIVE_UP + 1);
}

#[test]
fn every_rebinding_arrow_holds() {
    assert_arrows(&ARROWS);
}

#[test]
fn a_flap_every_20_s_for_10_minutes_sends_no_query_and_publishes_no_phase() {
    let mut simulation = ready();
    simulation.clear_logs();
    for flap in 0..TEN_MINUTES / FLAP_PERIOD {
        let stale_frame = flap % 2 == 0;
        simulation.flap(Flap {
            stale_frame,
            ..QUICK_FLAP
        });
        if flap % 3 == 0 {
            simulation.battery(percent(50));
        }
        simulation.run_for(FLAP_PERIOD);
    }
    assert!(simulation.writes().is_empty());
    assert!(simulation.published_phases().is_empty());
    assert_eq!(simulation.phase(), Phase::Ready);
}

#[test]
fn a_flap_while_settling_restarts_the_settle() {
    let mut simulation = settling();
    simulation.run_for(ENUMERATION_SETTLE - 100);
    simulation.flap(QUICK_FLAP);
    simulation.run_for(FLAP_DONE);
    let resumed = simulation.now();
    simulation.run_until(Phase::Probing, SETTLED);
    assert!(simulation.now().since(resumed) >= ENUMERATION_SETTLE - FLAP_DONE);
}

#[test]
fn a_flap_while_probing_resends_the_status_query() {
    let mut simulation = probing();
    simulation.flap(QUICK_FLAP);
    simulation.run_for(FLAP_DONE);
    assert_eq!(simulation.phase(), Phase::Syncing);
    assert_eq!(sent(&simulation, Query::Status.id()), 2);
}

#[test]
fn lost_while_rebinding_reaches_absent() {
    let mut simulation = ready();
    simulation.remove_command_node();
    simulation.unplug();
    assert_eq!(simulation.phase(), Phase::Absent);
    assert_eq!(simulation.rescans().last(), Some(&Cadence::Absent));
}

#[test]
fn rebinding_gives_up_into_absent_after_rebind_give_up() {
    let mut simulation = ready();
    simulation.clear_logs();
    simulation.remove_command_node();
    simulation.run_for(REBIND_GIVE_UP - 1);
    assert_eq!(simulation.phase(), Phase::Rebinding);
    simulation.run_for(1);
    assert_eq!(simulation.phase(), Phase::Absent);
    assert_eq!(
        simulation.published_phases(),
        [Phase::Rebinding, Phase::Absent]
    );
    assert_eq!(simulation.rescans(), [Cadence::Rebinding, Cadence::Absent]);
}

#[test]
fn an_event_handled_as_rebinding_gives_up_is_still_published() {
    let mut simulation = ready();
    simulation.remove_command_node();
    simulation.run_for(REBIND_GIVE_UP - 1);
    simulation.clear_logs();
    simulation.emit(vec![Emission {
        at: simulation.now().after(1),
        interface: Interface::Events,
        bytes: vec![0xB7, 0x30],
    }]);
    simulation.run_for(1);
    assert_eq!(simulation.phase(), Phase::Absent);
    let statuses = published(&simulation, |update| {
        matches!(update, DeviceUpdate::Status { .. })
    });
    assert_eq!(statuses, 1);
}

#[test]
fn eacces_twice_then_success_publishes_no_permission_denied() {
    let mut simulation = ready();
    simulation.clear_logs();
    simulation.flap(Flap {
        unreadable_for: 2 * RESCAN_REBINDING,
        ..QUICK_FLAP
    });
    simulation.run_for(4 * RESCAN_REBINDING);
    assert_eq!(simulation.phase(), Phase::Ready);
    assert!(simulation.published_phases().is_empty());
}

#[test]
fn a_node_unreadable_for_10_ms_is_retried_on_the_next_rescan() {
    let mut simulation = ready();
    simulation.clear_logs();
    simulation.flap(Flap {
        node_back_after: RESCAN_REBINDING,
        unreadable_for: 10,
        stale_frame: false,
    });
    simulation.run_for(2 * RESCAN_REBINDING);
    assert_eq!(simulation.phase(), Phase::Rebinding);
    simulation.run_for(RESCAN_REBINDING);
    assert_eq!(simulation.phase(), Phase::Ready);
    assert!(simulation.published_phases().is_empty());
}

#[test]
fn a_node_that_vanishes_again_during_the_rebind_settle_keeps_rebinding() {
    let mut simulation = ready();
    simulation.flap(QUICK_FLAP);
    simulation.run_for(RESCAN_REBINDING + 10);
    simulation.flap(Flap {
        node_back_after: 1_000,
        ..QUICK_FLAP
    });
    simulation.run_for(REBIND_SETTLE);
    assert_eq!(simulation.phase(), Phase::Rebinding);
    simulation.run_for(1_000 + FLAP_DONE);
    assert_eq!(simulation.phase(), Phase::Ready);
}

#[test]
fn a_stale_frame_from_another_claimant_is_dropped() {
    let mut simulation = ready();
    simulation.clear_logs();
    simulation.flap(Flap {
        stale_frame: true,
        ..QUICK_FLAP
    });
    simulation.run_for(FLAP_DONE);
    assert_eq!(simulation.phase(), Phase::Ready);
    let statuses = published(&simulation, |update| {
        matches!(update, DeviceUpdate::Status { .. })
    });
    assert_eq!(statuses, 0);
}

#[test]
fn the_first_reply_after_a_rebind_ignores_the_tail_of_another_claimant() {
    let mut simulation = syncing();
    simulation.flap(QUICK_FLAP);
    simulation.run_until(Phase::Ready, SETTLED);
    let settings = simulation.headset().settings().clone();
    assert_eq!(observed(&simulation), Some(settings));
}

#[test]
fn a_write_lost_to_eio_is_redone_and_verified_after_reopen() {
    let mut simulation = ready();
    let desired = with_sidetone(simulation.headset().settings(), 6);
    simulation.clear_logs();
    simulation.fail_next_write();
    simulation.desire(desired.clone(), SyncPolicy::Enforce);
    simulation.run_for(FLAP_DONE + 1_000);
    assert_eq!(sent(&simulation, setter(Setting::Sidetone)), 2);
    assert_eq!(simulation.headset().settings(), &desired);
    assert_eq!(observed(&simulation), Some(desired));
    let failures = published(&simulation, |update| {
        matches!(update, DeviceUpdate::WriteFailed(_))
    });
    assert_eq!(failures, 0);
}

#[test]
fn rebinding_is_published_only_after_rebind_notice() {
    let mut simulation = ready();
    simulation.clear_logs();
    simulation.flap(Flap {
        node_back_after: REBIND_NOTICE + 1_000,
        ..QUICK_FLAP
    });
    simulation.run_for(REBIND_NOTICE - 1);
    assert!(simulation.published_phases().is_empty());
    simulation.run_for(1);
    assert_eq!(simulation.published_phases(), [Phase::Rebinding]);
    simulation.run_for(1_000 + FLAP_DONE);
    assert_eq!(
        simulation.published_phases(),
        [Phase::Rebinding, Phase::Ready]
    );
}

#[test]
fn desired_changes_while_rebinding_are_written_after_resume() {
    let mut simulation = ready();
    let desired = with_sidetone(simulation.headset().settings(), 6);
    simulation.clear_logs();
    simulation.flap(Flap {
        node_back_after: 1_000,
        ..QUICK_FLAP
    });
    simulation.desire(desired.clone(), SyncPolicy::Enforce);
    simulation.run_for(1_000);
    assert!(simulation.writes().is_empty());
    simulation.run_for(FLAP_DONE + 1_000);
    assert_eq!(simulation.headset().settings(), &desired);
    assert_eq!(observed(&simulation), Some(desired));
}

#[test]
fn a_link_lost_while_rebinding_resumes_into_headset_off() {
    let mut simulation = ready();
    simulation.flap(Flap {
        node_back_after: 1_000,
        ..QUICK_FLAP
    });
    simulation.power_off();
    simulation.run_for(1_000 + FLAP_DONE);
    assert_eq!(simulation.phase(), Phase::HeadsetOff);
}

#[test]
fn a_headset_off_poll_keeps_its_schedule_across_a_flap() {
    let mut simulation = headset_off();
    simulation.clear_logs();
    simulation.run_for(HEADSET_OFF_POLL - 1_000);
    simulation.flap(QUICK_FLAP);
    simulation.run_for(1_000 + FLAP_DONE);
    assert_eq!(sent(&simulation, Query::Status.id()), 1);
}

#[test]
fn a_bootloader_node_never_reaches_rebinding() {
    let mut simulation = Simulation::new(FakeHeadset::nova_5().with_pid(0x2254));
    simulation.plug();
    simulation.remove_command_node();
    assert_eq!(simulation.phase(), Phase::Absent);
    assert!(simulation.rescans().is_empty());
}
