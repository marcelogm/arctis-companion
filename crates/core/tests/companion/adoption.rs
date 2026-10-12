use arctis_core::companion::adoption::{DongleIdentity, adopt};
use arctis_core::companion::app_settings::HeadsetValues;
use arctis_core::companion::catalog::Catalog;
use arctis_core::eq::preset::Preset;

use crate::support::{curve_of, custom, id, observed, observed_with, unique_curve};

#[test]
fn an_identity_without_a_serial_is_the_pid() {
    assert_eq!(DongleIdentity::new(0x2232, None).to_setting(), "2232");
    assert_eq!(
        DongleIdentity::new(0x2232, Some("")),
        DongleIdentity::new(0x2232, None)
    );
}

#[test]
fn an_identity_with_a_serial_keeps_it() {
    let identity = DongleIdentity::new(0x2253, Some("ABC123"));
    assert_eq!(identity.to_setting(), "2253:ABC123");
    assert_eq!(DongleIdentity::from_setting("2253:ABC123"), Some(identity));
}

#[test]
fn an_empty_or_broken_setting_was_never_adopted() {
    assert_eq!(DongleIdentity::from_setting(""), None);
    assert_eq!(DongleIdentity::from_setting("zz"), None);
}

#[test]
fn adoption_copies_every_observed_value() {
    let adoption = adopt(&observed(), &Catalog::default());
    assert_eq!(adoption.values, HeadsetValues::from_settings(&observed()));
}

#[test]
fn a_factory_curve_adopts_flat() {
    let adoption = adopt(&observed(), &Catalog::default());
    assert_eq!(adoption.preset, id("flat"));
    assert_eq!(adoption.new_preset, None);
}

#[test]
fn a_library_curve_adopts_that_preset() {
    let adoption = adopt(&observed_with(curve_of("bass-boost")), &Catalog::default());
    assert_eq!(adoption.preset, id("bass-boost"));
}

#[test]
fn a_known_custom_curve_adopts_that_custom_preset() {
    let catalog = Catalog::new(vec![custom("mine", unique_curve())]);
    let adoption = adopt(&observed_with(unique_curve()), &catalog);
    assert_eq!(adoption.preset, id("mine"));
    assert_eq!(adoption.new_preset, None);
}

#[test]
fn an_unknown_curve_becomes_the_headset_curve() {
    let adoption = adopt(&observed_with(unique_curve()), &Catalog::default());
    let expected = Preset::custom(
        id("headset-curve"),
        "Headset Curve".to_owned(),
        unique_curve(),
    );
    assert_eq!(adoption.preset, id("headset-curve"));
    assert_eq!(adoption.new_preset, Some(expected));
}

#[test]
fn a_second_unknown_curve_gets_a_free_id() {
    let catalog = Catalog::new(vec![custom("headset-curve", curve_of("podcast"))]);
    let adoption = adopt(&observed_with(unique_curve()), &catalog);
    assert_eq!(adoption.preset, id("headset-curve-2"));
}
