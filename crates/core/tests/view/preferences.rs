use arctis_core::testkit::english::english;
use arctis_core::view::preferences::{
    Background, LOW_BATTERY_MAX, LOW_BATTERY_MIN, PreferencesModel, preferences,
};

use crate::support::{percent, snapshot};

fn with_threshold(value: u8) -> PreferencesModel {
    let mut s = snapshot();
    s.battery.low_threshold = percent(value);
    preferences(&s, Background::On)
}

#[test]
fn run_in_background_follows_the_background_state() {
    let cases = [
        (Background::Off, false, None),
        (Background::On, true, None),
        (
            Background::Denied,
            false,
            Some("Allow it in Settings → Apps → Arctis Companion"),
        ),
    ];
    for (background, active, hint) in cases {
        let model = preferences(&snapshot(), background);
        assert_eq!(model.run_in_background, active, "{background:?}");
        assert_eq!(
            model.background_subtitle.as_ref().map(english).as_deref(),
            hint,
            "{background:?}"
        );
    }
}

#[test]
fn the_low_battery_warning_shows_the_threshold() {
    assert_eq!(with_threshold(15).low_battery, 15);
}

#[test]
fn the_low_battery_warning_stays_inside_its_range() {
    assert_eq!(with_threshold(2).low_battery, LOW_BATTERY_MIN);
    assert_eq!(with_threshold(80).low_battery, LOW_BATTERY_MAX);
}

#[test]
fn the_range_is_five_to_fifty_percent() {
    assert_eq!((LOW_BATTERY_MIN, LOW_BATTERY_MAX), (5, 50));
}

#[test]
fn the_background_starts_off() {
    assert_eq!(Background::default(), Background::Off);
}
