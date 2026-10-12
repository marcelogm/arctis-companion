use arctis_core::companion::app_settings::{AppRules, HeadsetValues, RoutePresets};
use arctis_core::messages::audio::{AppIdentifiers, SoundPath};
use arctis_core::settings::headset_settings::SettingChange;
use arctis_core::settings::values::{MicVolume, MuteLight, Sidetone, WirelessMode};

use crate::support::{curve_of, id, observed, unique_curve};

fn rules(entries: &[(&str, &str)]) -> AppRules {
    entries
        .iter()
        .map(|(identifier, preset)| ((*identifier).to_owned(), id(preset)))
        .collect()
}

fn firefox() -> AppIdentifiers {
    AppIdentifiers::new(Some("org.mozilla.firefox"), None, Some("firefox"))
}

#[test]
fn headset_values_round_trip_through_settings() {
    let values = HeadsetValues::from_settings(&observed());
    assert_eq!(values.with_curve(curve_of("flat")), observed());
}

#[test]
fn applying_a_change_updates_one_value() {
    let changes = [
        SettingChange::MicVolume(MicVolume::try_from(1).expect("mic")),
        SettingChange::Sidetone(Sidetone::try_from(10).expect("sidetone")),
        SettingChange::MuteLight(MuteLight::High),
        SettingChange::VolumeLimiter(true),
        SettingChange::WirelessMode(WirelessMode::Standard),
    ];
    for change in changes {
        let mut values = HeadsetValues::from_settings(&observed());
        values.apply(&change);
        let mut expected = observed();
        expected.apply(change.clone());
        assert_eq!(
            values,
            HeadsetValues::from_settings(&expected),
            "{change:?}"
        );
    }
}

#[test]
fn an_equalizer_change_leaves_headset_values_alone() {
    let mut values = HeadsetValues::from_settings(&observed());
    values.apply(&SettingChange::Equalizer(unique_curve()));
    assert_eq!(values, HeadsetValues::from_settings(&observed()));
}

#[test]
fn rules_match_any_identifier_in_any_case() {
    assert_eq!(
        rules(&[("FIREFOX", "footsteps")]).matching(&firefox()),
        Some(&id("footsteps"))
    );
    assert_eq!(
        rules(&[("chromium", "footsteps")]).matching(&firefox()),
        None
    );
}

#[test]
fn rules_follow_the_identifier_order_of_the_stream() {
    let both = rules(&[("firefox", "footsteps"), ("org.mozilla.firefox", "podcast")]);
    assert_eq!(both.matching(&firefox()), Some(&id("podcast")));
}

#[test]
fn rules_can_be_set_and_removed() {
    let mut app_rules = AppRules::default();
    assert!(app_rules.is_empty());
    app_rules.set("firefox".to_owned(), id("footsteps"));
    app_rules.set("chromium".to_owned(), id("podcast"));
    assert_eq!(app_rules.len(), 2);
    assert!(!app_rules.is_empty());
    app_rules.remove("firefox");
    assert_eq!(app_rules.len(), 1);
}

#[test]
fn route_presets_are_kept_per_path() {
    let mut presets = RoutePresets {
        dongle: id("flat"),
        bluetooth: id("flat"),
    };
    presets.set(SoundPath::Bluetooth, id("podcast"));
    assert_eq!(presets.get(SoundPath::Bluetooth), &id("podcast"));
    assert_eq!(presets.get(SoundPath::Dongle), &id("flat"));
}
