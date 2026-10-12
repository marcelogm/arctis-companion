use adw::prelude::*;
use arctis_core::companion::routing::Route;
use arctis_core::messages::device::Phase;
use arctis_core::testkit::snapshots::{snapshot, with};
use arctis_ui::PermissionDialog;

use crate::app_presets_page::rendered_page;
use crate::support::{descendants, rendered_window, rich_snapshot, ruled, seen_app, shown};

const NARROWEST: i32 = 360;

fn minimum_width(widget: &impl IsA<gtk::Widget>) -> i32 {
    widget.measure(gtk::Orientation::Horizontal, -1).0
}

#[gtk::test]
fn the_headset_page_fits_the_narrowest_window() {
    let (window, _) = rendered_window(rich_snapshot());
    let content = window.content().expect("window content");
    assert!(
        minimum_width(&content) <= NARROWEST,
        "{}",
        minimum_width(&content)
    );
}

#[gtk::test]
fn the_status_pages_fit_the_narrowest_window() {
    for phase in [Phase::Absent, Phase::PermissionDenied] {
        let (window, _) = rendered_window(with(phase, Route::Dongle));
        let content = window.content().expect("window content");
        assert!(minimum_width(&content) <= NARROWEST, "{phase:?}");
    }
}

#[gtk::test]
fn the_dialogs_fit_the_narrowest_window() {
    let (window, _) = rendered_window(snapshot());
    assert!(minimum_width(&window.preferences_dialog()) <= NARROWEST);
    assert!(minimum_width(&PermissionDialog::default()) <= NARROWEST);
}

#[gtk::test]
fn nothing_shown_scrolls_horizontally() {
    let (window, _) = rendered_window(rich_snapshot());
    let scrolled: Vec<gtk::ScrolledWindow> = descendants(&window)
        .into_iter()
        .filter(shown)
        .filter_map(|widget| widget.downcast::<gtk::ScrolledWindow>().ok())
        .collect();
    assert!(!scrolled.is_empty(), "the headset page scrolls vertically");
    for view in scrolled {
        assert_eq!(view.hscrollbar_policy(), gtk::PolicyType::Never);
    }
}

#[gtk::test]
fn the_app_presets_page_and_dialog_fit_the_narrowest_window() {
    let mut s = ruled(&[("org.mozilla.firefox", "footsteps"), ("mpv", "gone")]);
    s.presets.seen_apps = vec![seen_app("Spotify", &["com.spotify.Client", "spotify"])];
    let (page, _, _) = rendered_page(&s);
    assert!(
        minimum_width(&page) <= NARROWEST,
        "{}",
        minimum_width(&page)
    );
    let dialog = page.add_rule_dialog();
    assert!(
        minimum_width(&dialog) <= NARROWEST,
        "{}",
        minimum_width(&dialog)
    );
}
