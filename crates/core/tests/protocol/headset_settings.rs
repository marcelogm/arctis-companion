use arctis_core::eq::{Curve, FilterKind};
use arctis_core::settings::headset_settings::{Setting, SettingChange};
use arctis_core::settings::values::{AutoPowerOff, MuteLight, WirelessMode};

use crate::support::{filter, mic, sample_settings, sidetone};

fn bass_curve() -> Curve {
    Curve::from_array([filter(FilterKind::LowShelf, 80, 7, 707)])
}

fn changed(setting: Setting) -> SettingChange {
    match setting {
        Setting::MicVolume => SettingChange::MicVolume(mic(7)),
        Setting::Sidetone => SettingChange::Sidetone(sidetone(6)),
        Setting::MuteLight => SettingChange::MuteLight(MuteLight::Low),
        Setting::VolumeLimiter => SettingChange::VolumeLimiter(false),
        Setting::WirelessMode => SettingChange::WirelessMode(WirelessMode::ExtendedRange),
        Setting::AutoPowerOff => SettingChange::AutoPowerOff(AutoPowerOff::NEVER),
        Setting::Equalizer => SettingChange::Equalizer(bass_curve()),
    }
}

#[test]
fn every_change_names_its_setting() {
    for setting in Setting::ALL {
        assert_eq!(changed(setting).setting(), setting);
    }
}

#[test]
fn value_of_reads_the_current_field() {
    let settings = sample_settings();
    assert_eq!(
        settings.value_of(Setting::MuteLight),
        SettingChange::MuteLight(MuteLight::High)
    );
    assert_eq!(
        settings.value_of(Setting::Equalizer),
        SettingChange::Equalizer(Curve::default())
    );
}

#[test]
fn applying_a_change_sets_that_setting() {
    for setting in Setting::ALL {
        let mut settings = sample_settings();
        assert_ne!(settings.value_of(setting), changed(setting), "{setting:?}");
        settings.apply(changed(setting));
        assert_eq!(settings.value_of(setting), changed(setting));
    }
}

#[test]
fn applying_a_change_leaves_every_other_setting_alone() {
    let original = sample_settings();
    for setting in Setting::ALL {
        let mut settings = original.clone();
        settings.apply(changed(setting));
        let untouched = Setting::ALL
            .into_iter()
            .filter(|other| *other != setting)
            .all(|other| settings.value_of(other) == original.value_of(other));
        assert!(untouched, "{setting:?} touched another setting");
    }
}
