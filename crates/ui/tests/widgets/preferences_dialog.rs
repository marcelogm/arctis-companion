use adw::prelude::*;
use arctis_core::intent::Intent;
use arctis_core::testkit::snapshots::{percent, snapshot};
use arctis_core::view::preferences::{Background, LOW_BATTERY_MAX, LOW_BATTERY_MIN, preferences};
use arctis_ui::intents::IntentSink;
use arctis_ui::widget_values::nearest_step;
use arctis_ui::{PreferencesDialog, SHELL_EXTENSIONS_URI};
use gtk::glib;

use crate::support::{Log, init, named, recording_sink};

fn rendered(background: Background) -> (PreferencesDialog, IntentSink, Log) {
    init();
    let (sink, log) = recording_sink();
    let dialog: PreferencesDialog = glib::Object::new();
    dialog.connect_intents(&sink);
    sink.rendering(|| dialog.render(&preferences(&snapshot(), background)));
    (dialog, sink, log)
}

#[gtk::test]
fn run_in_background_shows_its_state() {
    let (dialog, _, _) = rendered(Background::On);
    let row = named::<adw::SwitchRow>(&dialog, "run-in-background");
    assert!(row.is_active());
    assert_eq!(row.subtitle().as_deref().unwrap_or_default(), "");
}

#[gtk::test]
fn a_denied_request_turns_the_switch_off_with_a_hint() {
    let (dialog, _, log) = rendered(Background::Denied);
    let row = named::<adw::SwitchRow>(&dialog, "run-in-background");
    assert!(!row.is_active());
    assert_eq!(
        row.subtitle().as_deref(),
        Some("Allow it in Settings → Apps → Arctis Companion")
    );
    assert!(log.borrow().is_empty());
}

#[gtk::test]
fn the_low_battery_warning_shows_the_threshold_and_range() {
    let (dialog, _, _) = rendered(Background::Off);
    let row = named::<adw::SpinRow>(&dialog, "low-battery");
    let adjustment = row.adjustment();
    assert_eq!(nearest_step(row.value(), 100), 15);
    assert_eq!(nearest_step(adjustment.lower(), 100), LOW_BATTERY_MIN);
    assert_eq!(nearest_step(adjustment.upper(), 100), LOW_BATTERY_MAX);
}

#[gtk::test]
fn each_control_emits_its_intent() {
    let (dialog, _, log) = rendered(Background::Off);
    named::<adw::SwitchRow>(&dialog, "run-in-background").set_active(true);
    named::<adw::SpinRow>(&dialog, "low-battery").set_value(20.0);
    assert_eq!(
        *log.borrow(),
        vec![
            Intent::SetRunInBackground(true),
            Intent::SetLowBatteryThreshold(percent(20)),
        ]
    );
}

#[gtk::test]
fn the_shell_integration_row_links_to_extensions() {
    let (dialog, _, _) = rendered(Background::Off);
    let row = named::<adw::ActionRow>(&dialog, "shell-integration");
    assert_eq!(row.title(), "GNOME Shell Integration");
    assert!(row.is_activatable());
    assert!(SHELL_EXTENSIONS_URI.starts_with("https://extensions.gnome.org/"));
}
