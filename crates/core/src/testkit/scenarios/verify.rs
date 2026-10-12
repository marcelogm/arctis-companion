use super::{observed, ready, setter, with_sidetone, write_failures};
use crate::messages::device::{Phase, SyncPolicy};
use crate::protocol::Query;
use crate::protocol::status::Link;
use crate::session::timing::VERIFY_BACKOFF;
use crate::settings::headset_settings::{HeadsetSettings, Setting};
use crate::settings::values::WirelessMode;
use crate::testkit::fake_headset::Behaviour;
use crate::testkit::rig::Rig;

pub fn a_desired_change_is_written_then_verified<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    let desired = with_sidetone(rig.headset().settings(), 6);
    rig.desire(desired.clone(), SyncPolicy::Enforce);
    rig.run_for(1_000);
    assert_eq!(
        rig.sent_ids(),
        [setter(Setting::Sidetone), Query::Audio.id()]
    );
    assert_eq!(rig.headset().settings(), &desired);
    assert_eq!(observed(&rig), Some(desired));
}

pub fn a_value_visible_only_at_the_last_readback_passes<R: Rig>() {
    let mut rig = ready::<R>(Behaviour {
        readback_latency: VERIFY_BACKOFF[2] - 10,
        ..Behaviour::default()
    });
    let desired = with_sidetone(rig.headset().settings(), 6);
    rig.desire(desired.clone(), SyncPolicy::Enforce);
    rig.run_for(2_000);
    assert!(write_failures(&rig).is_empty());
    assert_eq!(observed(&rig), Some(desired));
}

pub fn a_dropped_write_is_reported<R: Rig>() {
    let mut rig = ready::<R>(Behaviour {
        drops_writes: true,
        ..Behaviour::default()
    });
    let desired = with_sidetone(rig.headset().settings(), 6);
    rig.desire(desired, SyncPolicy::Enforce);
    rig.run_for(3_000);
    assert_eq!(write_failures(&rig), [Setting::Sidetone]);
}

pub fn a_wireless_mode_change_rides_through_its_link_drop<R: Rig>() {
    let mut rig = ready::<R>(Behaviour::default());
    let desired = HeadsetSettings {
        wireless_mode: WirelessMode::ExtendedRange,
        ..rig.headset().settings().clone()
    };
    rig.desire(desired.clone(), SyncPolicy::Enforce);
    rig.run_for(10);
    rig.link(Link::HeadsetOff);
    rig.run_for(2_000);
    rig.link(Link::Connected);
    rig.run_for(1_000);
    assert_eq!(rig.phase(), Phase::Ready);
    assert_eq!(observed(&rig), Some(desired));
}
