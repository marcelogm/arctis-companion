use arctis_core::companion::seen_apps::SeenApp;
use arctis_core::eq::library;
use arctis_core::testkit::english::english;
use arctis_core::view::app_presets::{
    AppPresetsContent, RuleRowModel, add_rule_dialog, app_presets,
};
use arctis_core::view::presets::preset_items;
use arctis_core::view::text::Text;

use arctis_core::testkit::snapshots::seen_app as seen;

use crate::support::{id, rules, snapshot};

fn rows(content: AppPresetsContent) -> Vec<RuleRowModel> {
    match content {
        AppPresetsContent::Rules(rows) => rows,
        AppPresetsContent::Empty(_) => panic!("rule rows"),
    }
}

fn firefox() -> SeenApp {
    seen("Firefox", &["org.mozilla.firefox", "firefox"])
}

#[test]
fn no_rules_shows_the_empty_state() {
    let AppPresetsContent::Empty(empty) = app_presets(&snapshot()).content else {
        panic!("empty state");
    };
    assert_eq!(english(&empty.title), "No App Presets");
    assert_eq!(
        english(&empty.description),
        "Choose a preset that applies while an app plays sound"
    );
    assert_eq!(
        empty.button.map(|text| english(&text)).as_deref(),
        Some("Add Rule")
    );
}

#[test]
fn rules_list_the_app_name_and_identifier() {
    let mut s = snapshot();
    s.presets.seen_apps = vec![firefox()];
    s.presets.rules = rules(&[("firefox", "footsteps")]);
    let row = rows(app_presets(&s).content).remove(0);
    assert_eq!(row.identifier, "firefox");
    assert_eq!(row.title, Text::verbatim("Firefox"));
    assert_eq!(row.subtitle, Text::verbatim("firefox"));
    assert_eq!(row.preset, Some(id("footsteps")));
}

#[test]
fn an_unseen_app_is_named_by_its_identifier() {
    let mut s = snapshot();
    s.presets.rules = rules(&[("mpv", "podcast")]);
    assert_eq!(
        rows(app_presets(&s).content)[0].title,
        Text::verbatim("mpv")
    );
}

#[test]
fn a_rule_naming_a_missing_preset_says_so() {
    let mut s = snapshot();
    s.presets.rules = rules(&[("firefox", "gone")]);
    let row = rows(app_presets(&s).content).remove(0);
    assert_eq!(english(&row.subtitle), "Preset missing");
    assert_eq!(row.preset, None);
}

#[test]
fn the_page_warns_when_automatic_presets_are_off() {
    let mut s = snapshot();
    assert_eq!(app_presets(&s).automatic_off, None);
    s.presets.automatic = false;
    let warning = app_presets(&s).automatic_off.expect("warning row");
    assert_eq!(english(&warning), "Automatic App Presets is off");
}

#[test]
fn the_page_explains_what_the_headset_keeps() {
    let page = app_presets(&snapshot());
    assert_eq!(
        english(&page.description),
        "On the 2.4 GHz connection the headset keeps an app's preset if this computer stops unexpectedly"
    );
    assert_eq!(page.presets.len(), library::library().len());
}

#[test]
fn add_rule_offers_seen_apps_without_a_rule() {
    let mut s = snapshot();
    s.presets.seen_apps = vec![
        firefox(),
        seen("Spotify", &["com.spotify.Client", "spotify"]),
    ];
    s.presets.rules = rules(&[("FIREFOX", "footsteps")]);
    let dialog = add_rule_dialog(&s);
    let offered: Vec<&str> = dialog
        .seen
        .iter()
        .map(|row| row.identifier.as_str())
        .collect();
    assert_eq!(offered, ["com.spotify.Client"]);
    assert_eq!(dialog.seen[0].title, Text::verbatim("Spotify"));
}

#[test]
fn add_rule_without_seen_apps_offers_only_another_app() {
    let dialog = add_rule_dialog(&snapshot());
    assert!(dialog.seen.is_empty());
    assert_eq!(english(&dialog.other_title), "Other App…");
    assert_eq!(
        english(&dialog.other_hint),
        "Flatpak app id, application id or binary name"
    );
}

#[test]
fn rules_are_listed_by_identifier() {
    let mut s = snapshot();
    s.presets.rules = rules(&[("mpv", "podcast"), ("firefox", "footsteps")]);
    let identifiers: Vec<String> = rows(app_presets(&s).content)
        .into_iter()
        .map(|row| row.identifier)
        .collect();
    assert_eq!(identifiers, ["firefox", "mpv"]);
}

#[test]
fn a_rule_finds_its_app_whatever_the_case() {
    let mut s = snapshot();
    s.presets.seen_apps = vec![firefox()];
    s.presets.rules = rules(&[("ORG.MOZILLA.FIREFOX", "footsteps")]);
    assert_eq!(
        rows(app_presets(&s).content)[0].title,
        Text::verbatim("Firefox")
    );
}

#[test]
fn add_rule_names_a_seen_app_by_its_key() {
    let mut s = snapshot();
    s.presets.seen_apps = vec![firefox()];
    let dialog = add_rule_dialog(&s);
    assert_eq!(dialog.seen[0].identifier, "org.mozilla.firefox");
    assert_eq!(
        dialog.seen[0].subtitle,
        Text::verbatim("org.mozilla.firefox")
    );
}

#[test]
fn every_rule_chooses_from_the_whole_library() {
    let mut s = snapshot();
    s.presets.rules = rules(&[("firefox", "footsteps")]);
    let page = app_presets(&s);
    assert_eq!(page.presets, preset_items(&s));
    assert!(page.presets.iter().any(|item| item.id == id("footsteps")));
}
