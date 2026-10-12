pub mod intent;
pub mod lifecycle;
pub mod rebinding;
pub mod verify;

use super::fake_headset::{Behaviour, FakeHeadset};
use super::rig::Rig;
use crate::messages::device::{DeviceUpdate, Phase};
use crate::protocol::{Command, encode};
use crate::session::timing::{ENUMERATION_SETTLE, RESCAN_ABSENT};
use crate::settings::headset_settings::{HeadsetSettings, Setting, SettingChange};
use crate::settings::values::Sidetone;

const DISCOVERY: u64 = RESCAN_ABSENT + 100;
const SETTLED: u64 = DISCOVERY + ENUMERATION_SETTLE + 1_000;

#[macro_export]
macro_rules! scenario_tests {
    (@for $rig:ty; $($module:ident: $($name:ident),+;)+) => {
        $($(
            #[test]
            fn $name() {
                $crate::testkit::scenarios::$module::$name::<$rig>();
            }
        )+)+
    };
    ($rig:ty) => {
        $crate::scenario_tests!(@for $rig;
            lifecycle:
                plugging_in_reaches_ready,
                a_headset_that_is_off_returns_on_its_link_event,
                a_headset_without_events_is_found_by_the_headset_off_poll,
                an_unanswered_probe_means_headset_off,
                a_command_frame_while_settling_is_dropped,
                an_idle_ready_dongle_asks_for_status_once_per_safety_poll,
                unplugging_while_settling_reaches_absent,
                unplugging_while_probing_reaches_absent,
                unplugging_while_syncing_reaches_absent,
                unplugging_a_ready_dongle_reaches_absent,
                unplugging_while_the_headset_is_off_reaches_absent,
                a_dongle_plugged_back_in_reaches_ready_again,
                a_denied_dongle_waits_for_permission,
                a_denied_dongle_that_leaves_is_absent;
            rebinding:
                a_flap_in_ready_is_silent,
                a_flap_every_20_s_for_ten_minutes_sends_nothing,
                an_unreadable_node_after_a_rebind_never_shows_permission_denied,
                a_stale_frame_after_a_rebind_is_dropped,
                a_flap_while_settling_resumes_settling,
                a_flap_while_probing_resumes_probing,
                a_command_node_that_never_returns_gives_up_into_absent,
                a_write_that_fails_is_redone_after_the_rebind,
                losing_the_dongle_while_rebinding_reaches_absent;
            verify:
                a_desired_change_is_written_then_verified,
                a_value_visible_only_at_the_last_readback_passes,
                a_dropped_write_is_reported,
                a_wireless_mode_change_rides_through_its_link_drop;
            intent:
                an_echo_on_the_events_interface_is_consumed,
                an_echo_on_the_command_interface_is_consumed,
                a_press_becomes_headset_changed,
                without_desired_settings_nothing_is_written,
                adopt_writes_nothing_and_publishes_observed,
                changes_made_while_the_headset_is_off_apply_when_it_turns_on;
        );
    };
}

fn plugged<R: Rig>(headset: FakeHeadset) -> R {
    let mut rig = R::start(headset);
    rig.plug();
    rig
}

fn ready<R: Rig>(behaviour: Behaviour) -> R {
    let mut rig = plugged::<R>(FakeHeadset::nova_5().with_behaviour(behaviour));
    rig.run_until(Phase::Ready, SETTLED);
    rig.clear_logs();
    rig
}

fn headset_off<R: Rig>(behaviour: Behaviour) -> R {
    let headset = FakeHeadset::nova_5()
        .with_behaviour(behaviour)
        .powered_off();
    let mut rig = plugged::<R>(headset);
    rig.run_until(Phase::HeadsetOff, SETTLED);
    rig.clear_logs();
    rig
}

fn with_sidetone(settings: &HeadsetSettings, level: u8) -> HeadsetSettings {
    HeadsetSettings {
        sidetone: sidetone(level),
        ..settings.clone()
    }
}

fn sidetone(level: u8) -> Sidetone {
    Sidetone::try_from(level).expect("a sidetone level in range")
}

fn setter(setting: Setting) -> u8 {
    let value = FakeHeadset::nova_5().settings().value_of(setting);
    encode(&Command::from(value)).payload()[0]
}

fn observed<R: Rig>(rig: &R) -> Option<HeadsetSettings> {
    rig.updates()
        .into_iter()
        .rev()
        .find_map(|(_, update)| match update {
            DeviceUpdate::Observed(settings) => Some(settings),
            _ => None,
        })
}

fn write_failures<R: Rig>(rig: &R) -> Vec<Setting> {
    rig.updates()
        .into_iter()
        .filter_map(|(_, update)| match update {
            DeviceUpdate::WriteFailed(setting) => Some(setting),
            _ => None,
        })
        .collect()
}

fn headset_changes<R: Rig>(rig: &R) -> Vec<SettingChange> {
    rig.updates()
        .into_iter()
        .filter_map(|(_, update)| match update {
            DeviceUpdate::HeadsetChanged(change) => Some(change),
            _ => None,
        })
        .collect()
}
