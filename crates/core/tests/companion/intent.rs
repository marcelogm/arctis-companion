use arctis_core::intent::Intent;
use arctis_core::settings::headset_settings::{Setting, SettingChange};
use arctis_core::settings::values::{AutoPowerOff, MicVolume, MuteLight, Sidetone, WirelessMode};

use crate::support::id;

fn mic(level: u8) -> MicVolume {
    MicVolume::try_from(level).expect("mic volume")
}

fn sidetone(level: u8) -> Sidetone {
    Sidetone::try_from(level).expect("sidetone")
}

#[test]
fn scales_emit_their_setting_intents() {
    let cases = [
        (Setting::MicVolume, 7, Some(Intent::SetMicVolume(mic(7)))),
        (Setting::MicVolume, 16, None),
        (Setting::Sidetone, 6, Some(Intent::SetSidetone(sidetone(6)))),
        (Setting::Sidetone, 11, None),
        (
            Setting::MuteLight,
            0,
            Some(Intent::SetMuteLight(MuteLight::Off)),
        ),
        (
            Setting::MuteLight,
            3,
            Some(Intent::SetMuteLight(MuteLight::High)),
        ),
        (Setting::MuteLight, 4, None),
        (Setting::VolumeLimiter, 1, None),
    ];
    for (setting, step, expected) in cases {
        assert_eq!(
            Intent::from_scale(setting, step),
            expected,
            "{setting:?} {step}"
        );
    }
}

#[test]
fn switches_emit_their_setting_intents() {
    assert_eq!(
        Intent::from_switch(Setting::VolumeLimiter, true),
        Some(Intent::SetVolumeLimiter(true))
    );
    assert_eq!(
        Intent::from_switch(Setting::WirelessMode, false),
        Some(Intent::SetExtendedRange(false))
    );
    assert_eq!(Intent::from_switch(Setting::Sidetone, true), None);
}

#[test]
fn the_power_off_combo_emits_its_choice() {
    let fifteen = AutoPowerOff::try_from(15).expect("choice");
    assert_eq!(
        Intent::from_auto_power_off_choice(4),
        Some(Intent::SetAutoPowerOff(fifteen))
    );
    assert_eq!(Intent::from_auto_power_off_choice(10), None);
}

#[test]
fn headset_intents_are_setting_changes() {
    let cases = [
        (
            Intent::SetMicVolume(mic(3)),
            SettingChange::MicVolume(mic(3)),
        ),
        (
            Intent::SetSidetone(sidetone(10)),
            SettingChange::Sidetone(sidetone(10)),
        ),
        (
            Intent::SetMuteLight(MuteLight::Low),
            SettingChange::MuteLight(MuteLight::Low),
        ),
        (
            Intent::SetVolumeLimiter(false),
            SettingChange::VolumeLimiter(false),
        ),
        (
            Intent::SetExtendedRange(true),
            SettingChange::WirelessMode(WirelessMode::ExtendedRange),
        ),
        (
            Intent::SetExtendedRange(false),
            SettingChange::WirelessMode(WirelessMode::Standard),
        ),
        (
            Intent::SetAutoPowerOff(AutoPowerOff::NEVER),
            SettingChange::AutoPowerOff(AutoPowerOff::NEVER),
        ),
    ];
    for (intent, change) in cases {
        assert_eq!(intent.setting_change(), Some(change), "{intent:?}");
    }
}

#[test]
fn other_intents_are_not_setting_changes() {
    assert_eq!(Intent::ActivatePreset(id("flat")).setting_change(), None);
    assert_eq!(Intent::EndPreview.setting_change(), None);
    assert_eq!(Intent::SetRunInBackground(true).setting_change(), None);
}
