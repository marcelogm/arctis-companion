use super::{DISCOVERY, SETTLED, headset_off, plugged, ready};

const SLOW_REPLY: u64 = REPLY_TIMEOUT - 100;
use crate::messages::device::{DeviceUpdate, Phase};
use crate::protocol::Query;
use crate::protocol::status::Link;
use crate::session::Interface;
use crate::session::timing::{HEADSET_OFF_POLL, REPLY_TIMEOUT, STATUS_SAFETY_POLL};
use crate::testkit::fake_headset::{Behaviour, Emission, FakeHeadset};
use crate::testkit::rig::Rig;

pub fn plugging_in_reaches_ready<R: Rig>() {
    let mut rig = plugged::<R>(FakeHeadset::nova_5());
    rig.run_until(Phase::Ready, SETTLED);
    assert_eq!(
        rig.published_phases(),
        [
            Phase::Settling,
            Phase::Probing,
            Phase::Syncing,
            Phase::Ready
        ]
    );
    let identities = rig
        .updates()
        .into_iter()
        .filter(|(_, update)| matches!(update, DeviceUpdate::Identity { .. }))
        .count();
    assert!(identities >= 1);
}

pub fn a_headset_that_is_off_returns_on_its_link_event<R: Rig>() {
    let mut rig = headset_off::<R>(Behaviour::default());
    rig.link(Link::Connected);
    rig.run_until(Phase::Ready, 2_000);
}

pub fn a_headset_without_events_is_found_by_the_headset_off_poll<R: Rig>() {
    let mut rig = headset_off::<R>(Behaviour {
        sends_events: false,
        ..Behaviour::default()
    });
    rig.link(Link::Connected);
    rig.run_until(Phase::Ready, HEADSET_OFF_POLL + 2_000);
}

pub fn an_unanswered_probe_means_headset_off<R: Rig>() {
    let headset = FakeHeadset::nova_5().with_behaviour(Behaviour {
        reply_latency: None,
        ..Behaviour::default()
    });
    let mut rig = plugged::<R>(headset);
    rig.run_until(Phase::HeadsetOff, SETTLED + 3 * REPLY_TIMEOUT);
}

pub fn a_command_frame_while_settling_is_dropped<R: Rig>() {
    let mut rig = plugged::<R>(FakeHeadset::nova_5());
    rig.run_until(Phase::Settling, DISCOVERY);
    rig.run_for(1_000);
    let stale = Emission {
        at: rig.now(),
        interface: Interface::Command,
        bytes: FakeHeadset::other_claimant_reply(),
    };
    rig.emit(vec![stale]);
    rig.run_until(Phase::Ready, SETTLED);
    assert_eq!(
        rig.published_phases(),
        [
            Phase::Settling,
            Phase::Probing,
            Phase::Syncing,
            Phase::Ready
        ]
    );
}

pub fn an_idle_ready_dongle_asks_for_status_once_per_safety_poll<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    rig.run_for(STATUS_SAFETY_POLL + 1_000);
    assert_eq!(rig.sent_ids(), [Query::Status.id()]);
}

pub fn unplugging_while_settling_reaches_absent<R: Rig>() {
    let mut rig = plugged::<R>(FakeHeadset::nova_5());
    rig.run_until(Phase::Settling, DISCOVERY);
    assert_unplugging_reaches_absent(&mut rig);
}

pub fn unplugging_while_probing_reaches_absent<R: Rig>() {
    let mut rig = plugged::<R>(slow_headset());
    rig.run_until(Phase::Probing, SETTLED);
    assert_unplugging_reaches_absent(&mut rig);
}

pub fn unplugging_while_syncing_reaches_absent<R: Rig>() {
    let mut rig = plugged::<R>(slow_headset());
    rig.run_until(Phase::Syncing, SETTLED);
    rig.run_for(SLOW_REPLY / 2);
    assert_eq!(rig.phase(), Phase::Syncing);
    assert_unplugging_reaches_absent(&mut rig);
}

fn slow_headset() -> FakeHeadset {
    FakeHeadset::nova_5().with_behaviour(Behaviour {
        reply_latency: Some(SLOW_REPLY),
        ..Behaviour::default()
    })
}

fn assert_unplugging_reaches_absent<R: Rig>(rig: &mut R) {
    rig.clear_logs();
    rig.unplug();
    rig.run_until(Phase::Absent, 1_000);
    rig.run_for(2 * SLOW_REPLY);
    assert_eq!(rig.published_phases(), [Phase::Absent]);
    assert_eq!(rig.phase(), Phase::Absent);
}

pub fn unplugging_a_ready_dongle_reaches_absent<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    rig.unplug();
    rig.run_until(Phase::Absent, 1_000);
    assert_eq!(rig.published_phases(), [Phase::Absent]);
}

pub fn unplugging_while_the_headset_is_off_reaches_absent<R: Rig>() {
    let mut rig = headset_off::<R>(Behaviour::default());
    rig.unplug();
    rig.run_until(Phase::Absent, 1_000);
    assert_eq!(rig.published_phases(), [Phase::Absent]);
}

pub fn a_dongle_plugged_back_in_reaches_ready_again<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    rig.unplug();
    rig.run_until(Phase::Absent, 1_000);
    rig.plug();
    rig.run_until(Phase::Ready, SETTLED);
}

pub fn a_denied_dongle_waits_for_permission<R: Rig>() {
    let mut rig = R::start(FakeHeadset::nova_5());
    rig.deny();
    rig.run_until(Phase::PermissionDenied, DISCOVERY);
    rig.plug();
    rig.run_until(Phase::Settling, DISCOVERY);
}

pub fn a_denied_dongle_that_leaves_is_absent<R: Rig>() {
    let mut rig = R::start(FakeHeadset::nova_5());
    rig.deny();
    rig.run_until(Phase::PermissionDenied, DISCOVERY);
    rig.unplug();
    rig.run_until(Phase::Absent, DISCOVERY);
}
