use arctis_core::eq::{Curve, Filter, FilterKind};
use arctis_core::settings::headset_settings::{HeadsetSettings, Setting, SettingChange};
use arctis_core::settings::values::{AutoPowerOff, MicVolume, MuteLight, Sidetone, WirelessMode};

fn sample_settings() -> HeadsetSettings {
    HeadsetSettings {
        mic_volume: MicVolume::try_from(15).expect("mic volume"),
        sidetone: Sidetone::try_from(0).expect("sidetone"),
        mute_light: MuteLight::High,
        volume_limiter: true,
        wireless_mode: WirelessMode::Standard,
        auto_power_off: AutoPowerOff::try_from(30).expect("auto power off"),
        equalizer: Curve::default(),
    }
}

fn bass_curve() -> Curve {
    let filter = Filter::from_parts(FilterKind::LowShelf, 80, 7, 707).expect("filter in envelope");
    Curve::from_array([filter])
}

fn changed(setting: Setting) -> SettingChange {
    match setting {
        Setting::MicVolume => SettingChange::MicVolume(MicVolume::try_from(7).expect("mic")),
        Setting::Sidetone => SettingChange::Sidetone(Sidetone::try_from(6).expect("sidetone")),
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
