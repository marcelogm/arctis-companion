use arctis_core::eq::Curve;
use arctis_core::messages::device::{Change, SyncPolicy};
use arctis_core::session::timing::{
    COMMAND_SPACING, ECHO_WINDOW, ENUMERATION_SETTLE, HEADSET_OFF_POLL, REBIND_GIVE_UP,
    REBIND_NOTICE, REBIND_SETTLE, REPLY_TIMEOUT, RESCAN_ABSENT, RESCAN_REBINDING,
    STATUS_SAFETY_POLL, STATUS_SAFETY_POLL_FLOOR, VERIFY_BACKOFF, WIRELESS_MODE_GRACE,
};
use arctis_core::session::{Cadence, Input};
use arctis_core::settings::headset_settings::HeadsetSettings;
use arctis_core::settings::values::{AutoPowerOff, MicVolume, MuteLight, Sidetone, WirelessMode};

#[test]
fn the_timing_constants_hold_the_designed_defaults() {
    let constants = [
        (COMMAND_SPACING, 50),
        (ENUMERATION_SETTLE, 3_000),
        (REBIND_SETTLE, 100),
        (REBIND_NOTICE, 5_000),
        (REBIND_GIVE_UP, 30_000),
        (REPLY_TIMEOUT, 500),
        (ECHO_WINDOW, 1_000),
        (WIRELESS_MODE_GRACE, 5_000),
        (HEADSET_OFF_POLL, 10_000),
        (STATUS_SAFETY_POLL, 300_000),
        (STATUS_SAFETY_POLL_FLOOR, 60_000),
        (RESCAN_ABSENT, 2_000),
        (RESCAN_REBINDING, 250),
    ];
    for (constant, designed) in constants {
        assert_eq!(constant, designed);
    }
    assert_eq!(VERIFY_BACKOFF, [50, 150, 400]);
}

#[test]
fn each_cadence_names_its_rescan_interval() {
    assert_eq!(Cadence::Absent.interval(), Some(RESCAN_ABSENT));
    assert_eq!(Cadence::Rebinding.interval(), Some(RESCAN_REBINDING));
    assert_eq!(Cadence::Off.interval(), None);
}

#[test]
fn a_desired_change_becomes_a_desired_input() {
    let settings = HeadsetSettings {
        mic_volume: MicVolume::try_from(15).expect("mic volume"),
        sidetone: Sidetone::try_from(0).expect("sidetone"),
        mute_light: MuteLight::High,
        volume_limiter: true,
        wireless_mode: WirelessMode::Standard,
        auto_power_off: AutoPowerOff::NEVER,
        equalizer: Curve::default(),
    };
    let change = Change::Desired {
        settings: settings.clone(),
        policy: SyncPolicy::Adopt,
    };
    assert_eq!(
        Input::from(change),
        Input::Desired {
            settings,
            policy: SyncPolicy::Adopt
        }
    );
}
