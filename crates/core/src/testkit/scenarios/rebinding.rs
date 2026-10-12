use super::{SETTLED, plugged, ready, setter, with_sidetone, write_failures};
use crate::messages::device::{Phase, SyncPolicy};
use crate::session::timing::{
    REBIND_GIVE_UP, REBIND_SETTLE, REPLY_TIMEOUT, RESCAN_ABSENT, RESCAN_REBINDING, VERIFY_BACKOFF,
};
use crate::settings::headset_settings::Setting;
use crate::settings::values::Percent;
use crate::testkit::fake_headset::{Behaviour, FakeHeadset};
use crate::testkit::rig::Rig;
use crate::testkit::simulation::Flap;

const QUICK_FLAP: Flap = Flap {
    node_back_after: 60,
    unreadable_for: 0,
    stale_frame: false,
};
const FLAP_DONE: u64 = 60 + RESCAN_REBINDING + REBIND_SETTLE + 50;
const FLAP_PERIOD: u64 = 20_000;
const TEN_MINUTES: u64 = 600_000;
const SLOW_REPLY: u64 = REPLY_TIMEOUT - 100;

pub fn a_flap_in_ready_is_silent<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    rig.flap(QUICK_FLAP);
    rig.run_for(FLAP_DONE);
    assert_eq!(rig.phase(), Phase::Ready);
    assert!(rig.writes().is_empty());
    assert!(rig.published_phases().is_empty());
}

pub fn a_flap_every_20_s_for_ten_minutes_sends_nothing<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    for flap in 0..TEN_MINUTES / FLAP_PERIOD {
        rig.flap(Flap {
            stale_frame: flap % 2 == 0,
            ..QUICK_FLAP
        });
        if flap % 3 == 0 {
            rig.battery(Percent::try_from(50).expect("a battery level in range"));
        }
        rig.run_for(FLAP_PERIOD);
    }
    assert!(rig.writes().is_empty());
    assert!(rig.published_phases().is_empty());
    assert_eq!(rig.phase(), Phase::Ready);
}

pub fn an_unreadable_node_after_a_rebind_never_shows_permission_denied<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    rig.flap(Flap {
        unreadable_for: 2 * RESCAN_REBINDING + 10,
        ..QUICK_FLAP
    });
    rig.run_for(2_000);
    assert_eq!(rig.phase(), Phase::Ready);
    assert!(rig.published_phases().is_empty());
}

pub fn a_stale_frame_after_a_rebind_is_dropped<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    rig.flap(Flap {
        stale_frame: true,
        ..QUICK_FLAP
    });
    rig.run_for(FLAP_DONE);
    assert_eq!(rig.phase(), Phase::Ready);
    assert!(rig.updates().is_empty());
}

pub fn a_flap_while_settling_resumes_settling<R: Rig>() {
    let mut rig = plugged::<R>(FakeHeadset::nova_5());
    rig.run_until(Phase::Settling, RESCAN_ABSENT);
    rig.run_for(1_000);
    rig.flap(QUICK_FLAP);
    rig.run_for(FLAP_DONE);
    assert_eq!(rig.phase(), Phase::Settling);
    rig.run_until(Phase::Ready, SETTLED);
}

pub fn a_flap_while_probing_resumes_probing<R: Rig>() {
    let headset = FakeHeadset::nova_5().with_behaviour(Behaviour {
        reply_latency: Some(SLOW_REPLY),
        ..Behaviour::default()
    });
    let mut rig = plugged::<R>(headset);
    rig.run_until(Phase::Probing, SETTLED);
    rig.flap(QUICK_FLAP);
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

pub fn a_command_node_that_never_returns_gives_up_into_absent<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    rig.remove_command_node();
    rig.run_for(REBIND_GIVE_UP - 1_000);
    assert_eq!(rig.phase(), Phase::Rebinding);
    rig.run_until(Phase::Absent, 2_000);
    assert_eq!(rig.published_phases(), [Phase::Rebinding, Phase::Absent]);
}

pub fn a_write_that_fails_is_redone_after_the_rebind<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    let desired = with_sidetone(rig.headset().settings(), 6);
    rig.fail_next_write();
    rig.desire(desired.clone(), SyncPolicy::Enforce);
    rig.run_for(FLAP_DONE + 4 * VERIFY_BACKOFF[2]);
    assert_eq!(rig.headset().settings(), &desired);
    assert!(rig.sent_ids().contains(&setter(Setting::Sidetone)));
    assert!(write_failures(&rig).is_empty());
    assert_eq!(rig.phase(), Phase::Ready);
}

pub fn losing_the_dongle_while_rebinding_reaches_absent<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    rig.remove_command_node();
    rig.run_for(100);
    rig.unplug();
    rig.run_until(Phase::Absent, 1_000);
}
