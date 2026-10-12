use adw::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::companion::routing::Route;
use arctis_core::messages::device::Phase;
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::{snapshot, with};
use arctis_ui::HeadsetStatus;
use gtk::{AccessibleProperty, glib};

use crate::support::{first, headset_model, init, named};

fn rendered(snapshot: &Snapshot) -> HeadsetStatus {
    init();
    let status: HeadsetStatus = glib::Object::new();
    status.render(&headset_model(snapshot).header);
    status
}

#[gtk::test]
fn the_header_shows_battery_and_route() {
    let s = snapshot();
    let header = headset_model(&s).header;
    let status = rendered(&s);
    assert_eq!(
        named::<gtk::Label>(&status, "battery").label(),
        english(header.battery.as_ref().expect("a reading"))
    );
    assert_eq!(
        named::<gtk::Label>(&status, "route").label(),
        english(&header.route)
    );
    assert!(!named::<gtk::Box>(&status, "connection").get_visible());
}

#[gtk::test]
fn the_level_icon_comes_from_the_model() {
    let s = snapshot();
    let header = headset_model(&s).header;
    let status = rendered(&s);
    let icons: Vec<String> = crate::support::descendants(&status)
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Image>().ok())
        .filter_map(|image| image.icon_name().map(String::from))
        .collect();
    assert!(icons.contains(&header.icon), "{icons:?}");
    assert!(icons.contains(&"audio-headset-symbolic".to_owned()));
}

#[gtk::test]
fn connecting_shows_a_spinner_and_its_subtitle() {
    let status = rendered(&with(Phase::Syncing, Route::Dongle));
    assert!(named::<gtk::Box>(&status, "connection").get_visible());
    assert!(first::<adw::Spinner>(&status).get_visible());
    assert!(
        crate::support::descendants(&status)
            .into_iter()
            .filter_map(|widget| widget.downcast::<gtk::Label>().ok())
            .any(|label| label.label() == "Connecting…")
    );
}

#[gtk::test]
fn rebinding_reads_reconnecting() {
    let status = rendered(&with(Phase::Rebinding, Route::Dongle));
    assert!(
        crate::support::descendants(&status)
            .into_iter()
            .filter_map(|widget| widget.downcast::<gtk::Label>().ok())
            .any(|label| label.label() == "Reconnecting…")
    );
}

#[gtk::test]
fn an_unknown_battery_hides_the_level_text() {
    let mut s = snapshot();
    s.battery.reading = None;
    let status = rendered(&s);
    assert!(!named::<gtk::Label>(&status, "battery").get_visible());
}

#[gtk::test]
fn the_header_has_an_accessible_description() {
    let s = snapshot();
    let status = rendered(&s);
    assert!(gtk::test_accessible_has_property(
        &status,
        AccessibleProperty::Description
    ));
    assert_eq!(
        status.description(),
        english(&headset_model(&s).header.description)
    );
}
