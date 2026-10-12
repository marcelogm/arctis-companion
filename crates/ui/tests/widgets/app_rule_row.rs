use adw::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::intent::Intent;
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::id;
use arctis_core::view::app_presets::{AppPresetsContent, RuleRowModel, app_presets};
use arctis_core::view::presets::{PresetItem, preset_items};
use arctis_ui::AppRuleRow;
use arctis_ui::app_icons::FALLBACK_ICON;
use arctis_ui::intents::IntentSink;

use crate::support::{Log, combo_text, init, named, recording_sink, ruled, seen_app};

fn rule_models(snapshot: &Snapshot) -> Vec<RuleRowModel> {
    match app_presets(snapshot).content {
        AppPresetsContent::Rules(rows) => rows,
        AppPresetsContent::Empty(_) => panic!("expected rule rows"),
    }
}

fn rendered(snapshot: &Snapshot) -> (AppRuleRow, IntentSink, Log) {
    init();
    let (sink, log) = recording_sink();
    let row = AppRuleRow::new(&sink);
    let presets = preset_items(snapshot);
    sink.rendering(|| row.render(&rule_models(snapshot)[0], &presets));
    (row, sink, log)
}

fn position_of(presets: &[PresetItem], preset: &str) -> u32 {
    let at = presets
        .iter()
        .position(|item| item.id == id(preset))
        .expect("a library preset");
    u32::try_from(at).expect("a short list")
}

#[gtk::test]
fn the_row_names_the_app_and_shows_its_preset() {
    let mut s = ruled(&[("firefox", "footsteps")]);
    s.presets.seen_apps = vec![seen_app("Firefox", &["org.mozilla.firefox", "firefox"])];
    let (row, _, _) = rendered(&s);
    let model = &rule_models(&s)[0];
    let presets = preset_items(&s);
    let footsteps = position_of(&presets, "footsteps");
    assert_eq!(row.title().as_str(), english(&model.title));
    assert_eq!(
        row.subtitle().as_deref(),
        Some(english(&model.subtitle).as_str())
    );
    assert_eq!(row.identifier(), "firefox");
    assert_eq!(row.selected(), footsteps);
    let index = usize::try_from(footsteps).expect("an index");
    assert_eq!(combo_text(row.upcast_ref()), english(&presets[index].title));
}

fn choice_count(row: &AppRuleRow) -> usize {
    let count = row.model().map(|model| model.n_items()).unwrap_or_default();
    usize::try_from(count).expect("a count")
}

#[gtk::test]
fn a_missing_preset_is_the_first_choice_and_says_so() {
    let s = ruled(&[("mpv", "gone")]);
    let (row, _, log) = rendered(&s);
    let missing = english(&rule_models(&s)[0].subtitle);
    assert_eq!(row.subtitle().as_deref(), Some(missing.as_str()));
    assert_eq!(row.selected(), 0);
    assert_eq!(combo_text(row.upcast_ref()), missing);
    assert_eq!(choice_count(&row), preset_items(&s).len() + 1);
    assert!(log.borrow().is_empty());
}

#[gtk::test]
fn picking_a_preset_repairs_a_missing_one() {
    let s = ruled(&[("mpv", "gone")]);
    let (row, _, log) = rendered(&s);
    row.set_selected(position_of(&preset_items(&s), "podcast") + 1);
    row.set_selected(0);
    assert_eq!(
        *log.borrow(),
        vec![Intent::SetAppRule {
            identifier: "mpv".to_owned(),
            preset: id("podcast"),
        }]
    );
}

#[gtk::test]
fn a_repaired_rule_drops_the_missing_choice() {
    let (row, sink, log) = rendered(&ruled(&[("mpv", "gone")]));
    let repaired = ruled(&[("mpv", "podcast")]);
    let presets = preset_items(&repaired);
    sink.rendering(|| row.render(&rule_models(&repaired)[0], &presets));
    assert_eq!(choice_count(&row), presets.len());
    assert_eq!(row.selected(), position_of(&presets, "podcast"));
    assert!(log.borrow().is_empty());
}

#[gtk::test]
fn an_app_without_an_icon_gets_the_generic_one() {
    let (row, _, _) = rendered(&ruled(&[("org.example.NoSuchPlayer", "podcast")]));
    let icon = named::<gtk::Image>(&row, "app-icon");
    assert_eq!(icon.icon_name().as_deref(), Some(FALLBACK_ICON));
}

#[gtk::test]
fn choosing_a_preset_sets_the_rule() {
    let s = ruled(&[("firefox", "footsteps")]);
    let (row, _, log) = rendered(&s);
    row.set_selected(position_of(&preset_items(&s), "podcast"));
    assert_eq!(
        *log.borrow(),
        vec![Intent::SetAppRule {
            identifier: "firefox".to_owned(),
            preset: id("podcast"),
        }]
    );
}

#[gtk::test]
fn the_remove_button_removes_the_rule() {
    let (row, _, log) = rendered(&ruled(&[("firefox", "footsteps")]));
    named::<gtk::Button>(&row, "remove-rule").emit_clicked();
    assert_eq!(
        *log.borrow(),
        vec![Intent::RemoveAppRule("firefox".to_owned())]
    );
}

#[gtk::test]
fn rendering_a_new_preset_emits_nothing() {
    let (row, sink, log) = rendered(&ruled(&[("firefox", "footsteps")]));
    let changed = ruled(&[("firefox", "podcast")]);
    let presets = preset_items(&changed);
    sink.rendering(|| row.render(&rule_models(&changed)[0], &presets));
    assert_eq!(row.selected(), position_of(&presets, "podcast"));
    assert!(log.borrow().is_empty());
}
