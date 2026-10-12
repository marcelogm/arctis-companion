use adw::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::companion::routing::Route;
use arctis_core::intent::Intent;
use arctis_core::messages::device::Phase;
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::{id, snapshot, with};
use arctis_core::view::app_presets::{AppPresetsContent, add_rule_dialog, app_presets};
use arctis_core::view::presets::preset_items;
use arctis_ui::intents::IntentSink;
use arctis_ui::{AddRuleDialog, AppPresetsPage, AppRuleRow, Window};
use gtk::glib;

use crate::support::{
    Log, descendants, first, init, named, recording_sink, rendered_window, ruled, seen_app,
};

pub fn rendered_page(snapshot: &Snapshot) -> (AppPresetsPage, IntentSink, Log) {
    init();
    let (sink, log) = recording_sink();
    let page: AppPresetsPage = glib::Object::new();
    page.connect_intents(&sink);
    render(&page, &sink, snapshot);
    (page, sink, log)
}

fn render(page: &AppPresetsPage, sink: &IntentSink, snapshot: &Snapshot) {
    sink.rendering(|| page.render(&app_presets(snapshot), &add_rule_dialog(snapshot)));
}

fn rule_rows(page: &AppPresetsPage) -> Vec<AppRuleRow> {
    descendants(page)
        .into_iter()
        .filter_map(|widget| widget.downcast::<AppRuleRow>().ok())
        .collect()
}

fn state(page: &AppPresetsPage) -> Option<String> {
    named::<gtk::Stack>(page, "app-presets-stack")
        .visible_child_name()
        .map(Into::into)
}

#[gtk::test]
fn no_rules_show_the_empty_state_with_add_rule() {
    let s = snapshot();
    let (page, _, _) = rendered_page(&s);
    let AppPresetsContent::Empty(empty) = app_presets(&s).content else {
        panic!("expected the empty state");
    };
    assert_eq!(state(&page).as_deref(), Some("empty"));
    assert_eq!(
        first::<adw::StatusPage>(&page).title().as_str(),
        english(&empty.title)
    );
    let button = named::<gtk::Button>(&page, "status-button");
    assert_eq!(
        button.label().as_deref(),
        empty.button.as_ref().map(english).as_deref()
    );
    assert_eq!(
        button.action_name().as_deref(),
        Some("app-presets.add-rule")
    );
    assert!(!named::<gtk::Button>(&page, "add-rule").get_visible());
}

#[gtk::test]
fn rules_list_one_row_per_app() {
    let s = ruled(&[("mpv", "gone"), ("firefox", "footsteps")]);
    let (page, _, _) = rendered_page(&s);
    assert_eq!(state(&page).as_deref(), Some("rules"));
    let identifiers: Vec<String> = rule_rows(&page)
        .iter()
        .map(AppRuleRow::identifier)
        .collect();
    assert_eq!(identifiers, ["firefox", "mpv"]);
    let AppPresetsContent::Rules(models) = app_presets(&s).content else {
        panic!("expected rule rows");
    };
    let subtitles: Vec<Option<String>> = rule_rows(&page)
        .iter()
        .map(|row| row.subtitle().map(String::from))
        .collect();
    let expected: Vec<Option<String>> = models
        .iter()
        .map(|row| Some(english(&row.subtitle)))
        .collect();
    assert_eq!(subtitles, expected);
    let add_rule = named::<gtk::Button>(&page, "add-rule");
    assert!(add_rule.get_visible());
    assert_eq!(
        add_rule.action_name().as_deref(),
        Some("app-presets.add-rule")
    );
}

#[gtk::test]
fn the_page_explains_what_the_headset_keeps() {
    let s = ruled(&[("firefox", "footsteps")]);
    let (page, _, _) = rendered_page(&s);
    let rules = first::<adw::PreferencesPage>(&page);
    assert_eq!(
        rules.description().as_str(),
        english(&app_presets(&s).description)
    );
}

#[gtk::test]
fn a_warning_row_says_automatic_presets_are_off() {
    let mut s = ruled(&[("firefox", "footsteps")]);
    let (page, sink, _) = rendered_page(&s);
    let group = named::<adw::PreferencesGroup>(&page, "automatic-off-group");
    assert!(!group.get_visible());
    s.presets.automatic = false;
    render(&page, &sink, &s);
    let warning = app_presets(&s).automatic_off.expect("a warning");
    assert!(group.get_visible());
    assert_eq!(
        named::<adw::ActionRow>(&page, "automatic-off")
            .title()
            .as_str(),
        english(&warning)
    );
}

#[gtk::test]
fn a_new_preset_keeps_the_rows() {
    let (page, sink, log) = rendered_page(&ruled(&[("firefox", "footsteps")]));
    let before = rule_rows(&page);
    render(&page, &sink, &ruled(&[("firefox", "podcast")]));
    assert_eq!(rule_rows(&page), before);
    assert!(log.borrow().is_empty());
}

#[gtk::test]
fn a_changed_rule_set_rebuilds_the_rows() {
    let (page, sink, log) = rendered_page(&ruled(&[("firefox", "footsteps")]));
    render(
        &page,
        &sink,
        &ruled(&[("firefox", "footsteps"), ("mpv", "podcast")]),
    );
    assert_eq!(rule_rows(&page).len(), 2);
    render(&page, &sink, &snapshot());
    assert!(rule_rows(&page).is_empty());
    assert_eq!(state(&page).as_deref(), Some("empty"));
    assert!(log.borrow().is_empty());
}

#[gtk::test]
fn each_row_control_emits_its_intent() {
    let s = ruled(&[("firefox", "footsteps")]);
    let (page, _, log) = rendered_page(&s);
    let podcast = preset_items(&s)
        .iter()
        .position(|item| item.id == id("podcast"))
        .expect("the podcast preset");
    let row = rule_rows(&page).remove(0);
    row.set_selected(u32::try_from(podcast).expect("a short list"));
    named::<gtk::Button>(&row, "remove-rule").emit_clicked();
    assert_eq!(
        *log.borrow(),
        vec![
            Intent::SetAppRule {
                identifier: "firefox".to_owned(),
                preset: id("podcast"),
            },
            Intent::RemoveAppRule("firefox".to_owned()),
        ]
    );
}

#[gtk::test]
fn the_dialog_follows_the_latest_snapshot() {
    let (page, sink, _) = rendered_page(&snapshot());
    let dialog = page.add_rule_dialog();
    assert!(!named::<adw::PreferencesGroup>(&dialog, "seen-group").get_visible());
    let mut s = snapshot();
    s.presets.seen_apps = vec![seen_app("Spotify", &["com.spotify.Client"])];
    render(&page, &sink, &s);
    assert!(named::<adw::PreferencesGroup>(&dialog, "seen-group").get_visible());
    assert_eq!(page.add_rule_dialog(), dialog);
}

fn open_app_presets(window: &Window) -> AppPresetsPage {
    let row = named::<adw::ActionRow>(window, "app-presets-row");
    let action = row.action_name().expect("the row has an action");
    WidgetExt::activate_action(&row, &action, row.action_target_value().as_ref())
        .expect("the action exists");
    named::<adw::NavigationView>(window, "navigation")
        .visible_page()
        .and_downcast::<AppPresetsPage>()
        .expect("the app presets page is shown")
}

fn add_rule_through(window: &Window, page: &AppPresetsPage) -> AddRuleDialog {
    named::<gtk::Button>(page, "status-button").emit_clicked();
    window
        .visible_dialog()
        .and_downcast::<AddRuleDialog>()
        .expect("the add-rule dialog is presented")
}

#[gtk::test]
fn the_app_presets_row_opens_the_page() {
    let (window, _) = rendered_window(snapshot());
    assert_eq!(
        open_app_presets(&window).tag().as_deref(),
        Some("app-presets")
    );
}

#[gtk::test]
fn the_window_renders_the_rules_and_forwards_their_intents() {
    let (window, log) = rendered_window(ruled(&[("firefox", "footsteps")]));
    let page = open_app_presets(&window);
    assert!(log.borrow().is_empty());
    named::<gtk::Button>(&page, "remove-rule").emit_clicked();
    assert_eq!(
        *log.borrow(),
        vec![Intent::RemoveAppRule("firefox".to_owned())]
    );
}

#[gtk::test]
fn rules_stay_editable_in_bluetooth_only_mode() {
    let mut s = with(Phase::Absent, Route::Bluetooth);
    s.presets.rules = ruled(&[("firefox", "footsteps")]).presets.rules;
    let (window, log) = rendered_window(s);
    let page = open_app_presets(&window);
    let row = named::<AppRuleRow>(&page, "app-rule");
    assert!(row.is_sensitive());
    named::<gtk::Button>(&row, "remove-rule").emit_clicked();
    assert_eq!(
        *log.borrow(),
        vec![Intent::RemoveAppRule("firefox".to_owned())]
    );
}

#[gtk::test]
fn a_rule_added_in_the_dialog_reaches_the_window_handler() {
    let (window, log) = rendered_window(snapshot());
    window.present();
    let page = open_app_presets(&window);
    let entry = named::<adw::EntryRow>(&add_rule_through(&window, &page), "other-app");
    entry.set_text("mpv");
    entry.emit_by_name::<()>("apply", &[]);
    assert_eq!(
        *log.borrow(),
        vec![Intent::SetAppRule {
            identifier: "mpv".to_owned(),
            preset: preset_items(&snapshot())[0].id.clone(),
        }]
    );
    assert!(window.visible_dialog().is_none());
}

#[gtk::test]
fn the_presented_dialog_follows_new_snapshots() {
    let (window, _) = rendered_window(snapshot());
    window.present();
    let dialog = add_rule_through(&window, &open_app_presets(&window));
    let mut s = snapshot();
    s.presets.seen_apps = vec![seen_app("Spotify", &["com.spotify.Client"])];
    window.render(&std::rc::Rc::new(s));
    assert!(named::<adw::PreferencesGroup>(&dialog, "seen-group").get_visible());
}

#[gtk::test]
fn the_header_button_presents_the_dialog_when_rules_exist() {
    let (window, _) = rendered_window(ruled(&[("firefox", "footsteps")]));
    window.present();
    let page = open_app_presets(&window);
    named::<gtk::Button>(&page, "add-rule").emit_clicked();
    assert!(
        window
            .visible_dialog()
            .and_downcast::<AddRuleDialog>()
            .is_some()
    );
}
