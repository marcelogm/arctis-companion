use super::{DISCOVERY, SETTLED, Scenario, headset_off, plugged, ready, scenario};
use crate::messages::device::{DeviceUpdate, Phase};
use crate::protocol::status::Link;
use crate::session::timing::{HEADSET_OFF_POLL, REPLY_TIMEOUT};
use crate::testkit::fake_headset::{Behaviour, FakeHeadset};
use crate::testkit::rig::Rig;

pub(super) fn scenarios<R: Rig>() -> Vec<Scenario> {
    vec![
        scenario(
            "plugging in reaches Ready through every connecting phase",
            plugging_in_reaches_ready::<R>,
        ),
        scenario(
            "a headset that is off returns on its link event",
            a_headset_that_is_off_returns_on_its_link_event::<R>,
        ),
        scenario(
            "a headset without events is found by the headset-off poll",
            a_headset_without_events_is_found_by_the_headset_off_poll::<R>,
        ),
        scenario(
            "an unanswered probe means HeadsetOff",
            an_unanswered_probe_means_headset_off::<R>,
        ),
        scenario(
            "unplugging a ready dongle reaches Absent",
            unplugging_a_ready_dongle_reaches_absent::<R>,
        ),
        scenario(
            "unplugging while the headset is off reaches Absent",
            unplugging_while_the_headset_is_off_reaches_absent::<R>,
        ),
        scenario(
            "a dongle plugged back in reaches Ready again",
            a_dongle_plugged_back_in_reaches_ready_again::<R>,
        ),
        scenario(
            "a denied dongle waits for permission",
            a_denied_dongle_waits_for_permission::<R>,
        ),
        scenario(
            "a denied dongle that leaves is Absent",
            a_denied_dongle_that_leaves_is_absent::<R>,
        ),
    ]
}

fn plugging_in_reaches_ready<R: Rig>() {
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

fn a_headset_that_is_off_returns_on_its_link_event<R: Rig>() {
    let mut rig = headset_off::<R>(Behaviour::default());
    rig.link(Link::Connected);
    rig.run_until(Phase::Ready, 2_000);
}

fn a_headset_without_events_is_found_by_the_headset_off_poll<R: Rig>() {
    let mut rig = headset_off::<R>(Behaviour {
        sends_events: false,
        ..Behaviour::default()
    });
    rig.link(Link::Connected);
    rig.run_until(Phase::Ready, HEADSET_OFF_POLL + 2_000);
}

fn an_unanswered_probe_means_headset_off<R: Rig>() {
    let headset = FakeHeadset::nova_5().with_behaviour(Behaviour {
        reply_latency: None,
        ..Behaviour::default()
    });
    let mut rig = plugged::<R>(headset);
    rig.run_until(Phase::HeadsetOff, SETTLED + 3 * REPLY_TIMEOUT);
}

fn unplugging_a_ready_dongle_reaches_absent<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    rig.unplug();
    rig.run_until(Phase::Absent, 1_000);
    assert_eq!(rig.published_phases(), [Phase::Absent]);
}

fn unplugging_while_the_headset_is_off_reaches_absent<R: Rig>() {
    let mut rig = headset_off::<R>(Behaviour::default());
    rig.unplug();
    rig.run_until(Phase::Absent, 1_000);
    assert_eq!(rig.published_phases(), [Phase::Absent]);
}

fn a_dongle_plugged_back_in_reaches_ready_again<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    rig.unplug();
    rig.run_until(Phase::Absent, 1_000);
    rig.plug();
    rig.run_until(Phase::Ready, SETTLED);
}

fn a_denied_dongle_waits_for_permission<R: Rig>() {
    let mut rig = R::start(FakeHeadset::nova_5());
    rig.deny();
    rig.run_until(Phase::PermissionDenied, DISCOVERY);
    rig.plug();
    rig.run_until(Phase::Settling, DISCOVERY);
}

fn a_denied_dongle_that_leaves_is_absent<R: Rig>() {
    let mut rig = R::start(FakeHeadset::nova_5());
    rig.deny();
    rig.run_until(Phase::PermissionDenied, DISCOVERY);
    rig.unplug();
    rig.run_until(Phase::Absent, DISCOVERY);
}
