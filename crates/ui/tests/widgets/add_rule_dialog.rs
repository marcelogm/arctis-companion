use adw::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::intent::Intent;
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::snapshot;
use arctis_core::view::app_presets::add_rule_dialog;
use arctis_core::view::presets::preset_items;
use arctis_ui::AddRuleDialog;
use arctis_ui::intents::IntentSink;
use gtk::glib;

use crate::support::{Log, descendants, init, named, recording_sink, seen_app};

pub fn rendered_dialog(snapshot: &Snapshot) -> (AddRuleDialog, IntentSink, Log) {
    init();
    let (sink, log) = recording_sink();
    let dialog: AddRuleDialog = glib::Object::new();
    dialog.connect_intents(&sink);
    sink.rendering(|| dialog.render(&add_rule_dialog(snapshot), &preset_items(snapshot)));
    (dialog, sink, log)
}

pub fn spotify_seen() -> Snapshot {
    let mut s = snapshot();
    s.presets.seen_apps = vec![seen_app("Spotify", &["com.spotify.Client", "spotify"])];
    s
}

fn seen_rows(dialog: &AddRuleDialog) -> Vec<adw::ActionRow> {
    descendants(dialog)
        .into_iter()
        .filter(|widget| widget.widget_name() == "seen-app")
        .filter_map(|widget| widget.downcast().ok())
        .collect()
}

fn apply(entry: &adw::EntryRow, text: &str) {
    entry.set_text(text);
    entry.emit_by_name::<()>("apply", &[]);
}

fn presented(dialog: &AddRuleDialog) -> adw::Window {
    let window = adw::Window::new();
    window.present();
    dialog.open(&window);
    window
}

#[gtk::test]
fn seen_apps_are_offered_by_name_and_key() {
    let s = spotify_seen();
    let (dialog, _, _) = rendered_dialog(&s);
    let model = add_rule_dialog(&s);
    assert!(named::<adw::PreferencesGroup>(&dialog, "seen-group").get_visible());
    let rows = seen_rows(&dialog);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title().as_str(), english(&model.seen[0].title));
    assert_eq!(
        rows[0].subtitle().as_deref(),
        Some(english(&model.seen[0].subtitle).as_str())
    );
}

#[gtk::test]
fn without_seen_apps_only_another_app_is_offered() {
    let s = snapshot();
    let (dialog, _, _) = rendered_dialog(&s);
    let model = add_rule_dialog(&s);
    assert!(!named::<adw::PreferencesGroup>(&dialog, "seen-group").get_visible());
    assert_eq!(
        named::<adw::EntryRow>(&dialog, "other-app")
            .title()
            .as_str(),
        english(&model.other_title)
    );
    let other = named::<adw::PreferencesGroup>(&dialog, "other-group");
    assert_eq!(
        other.description().as_deref(),
        Some(english(&model.other_hint).as_str())
    );
}

#[gtk::test]
fn without_seen_apps_the_entry_takes_the_focus() {
    let (dialog, _, _) = rendered_dialog(&snapshot());
    let _window = presented(&dialog);
    let entry = named::<adw::EntryRow>(&dialog, "other-app");
    let focus = dialog.focus().expect("a focused widget");
    assert!(focus == entry.clone().upcast::<gtk::Widget>() || focus.is_ancestor(&entry));
}

#[gtk::test]
fn the_preset_row_offers_the_library() {
    let s = snapshot();
    let (dialog, _, _) = rendered_dialog(&s);
    let combo = named::<adw::ComboRow>(&dialog, "rule-preset");
    let count = combo
        .model()
        .map(|model| model.n_items())
        .unwrap_or_default();
    assert_eq!(
        usize::try_from(count).expect("a count"),
        preset_items(&s).len()
    );
    assert_eq!(combo.selected(), 0);
}

#[gtk::test]
fn picking_a_seen_app_adds_its_rule_with_the_chosen_preset() {
    let s = spotify_seen();
    let (dialog, _, log) = rendered_dialog(&s);
    let window = presented(&dialog);
    named::<adw::ComboRow>(&dialog, "rule-preset").set_selected(2);
    seen_rows(&dialog)[0].emit_by_name::<()>("activated", &[]);
    assert_eq!(
        *log.borrow(),
        vec![Intent::SetAppRule {
            identifier: "com.spotify.Client".to_owned(),
            preset: preset_items(&s)[2].id.clone(),
        }]
    );
    assert!(window.visible_dialog().is_none());
}

#[gtk::test]
fn another_app_is_added_by_its_trimmed_identifier() {
    let s = snapshot();
    let (dialog, _, log) = rendered_dialog(&s);
    let window = presented(&dialog);
    let entry = named::<adw::EntryRow>(&dialog, "other-app");
    apply(&entry, "  mpv ");
    assert_eq!(
        *log.borrow(),
        vec![Intent::SetAppRule {
            identifier: "mpv".to_owned(),
            preset: preset_items(&s)[0].id.clone(),
        }]
    );
    assert_eq!(entry.text().as_str(), "");
    assert!(window.visible_dialog().is_none());
}

#[gtk::test]
fn an_empty_identifier_adds_nothing_and_keeps_the_dialog_open() {
    let (dialog, _, log) = rendered_dialog(&snapshot());
    let window = presented(&dialog);
    apply(&named::<adw::EntryRow>(&dialog, "other-app"), "   ");
    assert!(log.borrow().is_empty());
    assert!(window.visible_dialog().is_some());
}

#[gtk::test]
fn a_chosen_preset_that_disappears_falls_back_to_the_first() {
    let s = snapshot();
    let (dialog, sink, log) = rendered_dialog(&s);
    let combo = named::<adw::ComboRow>(&dialog, "rule-preset");
    combo.set_selected(u32::try_from(preset_items(&s).len() - 1).expect("a short list"));
    let mut fewer = snapshot();
    fewer.presets.all.pop();
    sink.rendering(|| dialog.render(&add_rule_dialog(&fewer), &preset_items(&fewer)));
    assert_eq!(combo.selected(), 0);
    assert!(log.borrow().is_empty());
}

#[gtk::test]
fn rendering_keeps_the_chosen_preset_and_emits_nothing() {
    let (dialog, sink, log) = rendered_dialog(&spotify_seen());
    let combo = named::<adw::ComboRow>(&dialog, "rule-preset");
    combo.set_selected(3);
    let s = snapshot();
    sink.rendering(|| dialog.render(&add_rule_dialog(&s), &preset_items(&s)));
    assert_eq!(combo.selected(), 3);
    assert!(seen_rows(&dialog).is_empty());
    assert!(log.borrow().is_empty());
}
