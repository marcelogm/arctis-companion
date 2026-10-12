use std::rc::Rc;

use adw::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::companion::routing::Route;
use arctis_core::companion::selection::{ActivePreset, PresetReason};
use arctis_core::eq::apo;
use arctis_core::eq::library;
use arctis_core::eq::preset::PresetKind;
use arctis_core::intent::Intent;
use arctis_core::messages::device::Phase;
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::{id, snapshot, with};
use arctis_core::view::preset_files::export_text;
use arctis_core::view::presets::deleted_toast;

use crate::support::{
    band_rows, click, drain, named, one_peak, open_editor, rendered_window, saved, shown,
    shows_text, standalone_editor, step_up, visible_tag, with_preset,
};

fn picked(preset: &str) -> Snapshot {
    let mut s = snapshot();
    s.presets.active.dongle = ActivePreset {
        preset: id(preset),
        reason: PresetReason::Manual(None),
    };
    s
}

#[gtk::test]
fn the_duplicate_button_saves_a_custom_copy_and_edits_it() {
    let (window, log) = rendered_window(snapshot());
    let page = open_editor(&window, "bass-boost");
    named::<gtk::Button>(&page, "duplicate").emit_clicked();
    let copy = saved(&log);
    let source = library::find(&id("bass-boost")).expect("a built-in");
    assert_eq!(copy.name(), "Bass Boost (Copy)");
    assert_eq!(copy.id(), &id("bass-boost-copy"));
    assert_eq!(copy.kind(), PresetKind::Custom);
    assert_eq!(copy.curve(), source.curve());
    assert_eq!(page.title(), "Bass Boost (Copy)");
    assert!(shown(
        named::<gtk::MenuButton>(&page, "editor-menu").upcast_ref()
    ));
    assert!(!shown(
        named::<gtk::Button>(&page, "duplicate").upcast_ref()
    ));
}

#[gtk::test]
fn duplicating_twice_before_the_snapshot_returns_saves_one_copy() {
    let editor = standalone_editor(snapshot(), "bass-boost");
    editor
        .page
        .activate_action("editor.duplicate", None)
        .expect("the action exists");
    editor.page.activate_action("editor.duplicate", None).ok();
    let log = editor.log.borrow();
    let copies: Vec<_> = log
        .iter()
        .filter(|intent| matches!(intent, Intent::SavePreset(_)))
        .collect();
    assert_eq!(copies.len(), 1);
    assert_eq!(editor.page.title(), "Bass Boost (Copy)");
}

#[gtk::test]
fn custom_presets_cannot_be_duplicated_and_built_ins_cannot_be_deleted() {
    let custom = standalone_editor(with_preset(&one_peak()), "mine");
    custom.page.activate_action("editor.duplicate", None).ok();
    assert!(custom.log.borrow().is_empty());
    let built_in = standalone_editor(snapshot(), "flat");
    built_in.page.activate_action("editor.delete", None).ok();
    built_in.page.activate_action("editor.rename", None).ok();
    assert!(built_in.log.borrow().is_empty());
}

#[gtk::test]
fn rename_saves_the_trimmed_name_under_the_same_id() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    editor.page.rename_to("  Night ");
    let renamed = saved(&editor.log);
    assert_eq!(renamed.name(), "Night");
    assert_eq!(renamed.id(), &id("mine"));
    assert_eq!(editor.page.title(), "Night");
    editor.page.rename_to("   ");
    assert_eq!(editor.log.borrow().len(), 1);
}

#[gtk::test]
fn export_writes_the_format_of_the_chosen_name() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    editor.files.save_as.replace("mine.txt".to_owned());
    editor
        .page
        .activate_action("editor.export", None)
        .expect("the action exists");
    editor.files.save_as.replace("mine.json".to_owned());
    editor
        .page
        .activate_action("editor.export", None)
        .expect("the action exists");
    assert_eq!(
        *editor.files.saved.borrow(),
        vec![
            apo::render(one_peak().curve()),
            export_text(&one_peak(), "mine.json").expect("exported"),
        ]
    );
    assert!(editor.log.borrow().is_empty());
}

#[gtk::test]
fn a_failed_export_raises_a_toast() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    editor.files.refuse_writes.set(true);
    editor
        .page
        .activate_action("editor.export", None)
        .expect("the action exists");
    assert!(shows_text(&editor.toasts, "Could not export “Mine”"));
}

#[gtk::test]
fn delete_closes_the_editor_and_names_the_uses_with_undo() {
    let mut s = with_preset(&one_peak());
    s.presets.route_presets.dongle = id("mine");
    let toast = english(&deleted_toast(&s, &one_peak()));
    assert_eq!(toast, "Deleted “Mine”. Used by the 2.4 GHz preset");
    let (window, log) = rendered_window(s);
    let page = open_editor(&window, "mine");
    page.activate_action("editor.delete", None)
        .expect("the action exists");
    assert_eq!(*log.borrow(), vec![Intent::DeletePreset(id("mine"))]);
    assert_eq!(visible_tag(&window).as_deref(), Some("main"));
    assert!(shows_text(&window, &toast));
    click(&window, "Undo");
    assert_eq!(log.borrow().last(), Some(&Intent::SavePreset(one_peak())));
}

#[gtk::test]
fn preview_starts_and_ends_a_manual_pick() {
    let editor = standalone_editor(snapshot(), "flat");
    let preview = named::<gtk::ToggleButton>(&editor.page, "preview");
    assert!(preview.is_visible());
    preview.set_active(true);
    preview.set_active(false);
    assert_eq!(
        *editor.log.borrow(),
        vec![Intent::StartPreview(id("flat")), Intent::EndPreview]
    );
}

#[gtk::test]
fn preview_is_hidden_without_a_picked_path() {
    let editor = standalone_editor(with(Phase::Absent, Route::None), "flat");
    assert!(!named::<gtk::ToggleButton>(&editor.page, "preview").is_visible());
}

#[gtk::test]
fn losing_the_picked_path_turns_preview_off_without_an_intent() {
    let editor = standalone_editor(snapshot(), "flat");
    let preview = named::<gtk::ToggleButton>(&editor.page, "preview");
    preview.set_active(true);
    editor
        .page
        .render(&Rc::new(with(Phase::Absent, Route::None)));
    assert!(!preview.is_active());
    assert_eq!(*editor.log.borrow(), vec![Intent::StartPreview(id("flat"))]);
}

#[gtk::test]
fn closing_the_editor_ends_its_preview() {
    let (window, log) = rendered_window(snapshot());
    let page = open_editor(&window, "flat");
    named::<gtk::ToggleButton>(&page, "preview").set_active(true);
    assert!(named::<adw::NavigationView>(&window, "navigation").pop());
    assert_eq!(
        *log.borrow(),
        vec![Intent::StartPreview(id("flat")), Intent::EndPreview]
    );
}

#[gtk::test]
fn edits_while_previewing_keep_the_preview() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    let preview = named::<gtk::ToggleButton>(&editor.page, "preview");
    preview.set_active(true);
    step_up(&named::<adw::SpinRow>(
        &band_rows(&editor.page)[0],
        "band-gain",
    ));
    assert!(preview.is_active());
    assert_eq!(editor.log.borrow()[0], Intent::StartPreview(id("mine")));
    assert!(matches!(editor.log.borrow()[1], Intent::SavePreset(_)));
}

#[gtk::test]
fn deleting_while_previewing_ends_the_preview_first() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    named::<gtk::ToggleButton>(&editor.page, "preview").set_active(true);
    editor
        .page
        .activate_action("editor.delete", None)
        .expect("the action exists");
    assert_eq!(
        *editor.log.borrow(),
        vec![
            Intent::StartPreview(id("mine")),
            Intent::EndPreview,
            Intent::DeletePreset(id("mine")),
        ]
    );
}

#[gtk::test]
fn duplicating_ends_the_preview_first() {
    let editor = standalone_editor(snapshot(), "bass-boost");
    named::<gtk::ToggleButton>(&editor.page, "preview").set_active(true);
    editor
        .page
        .activate_action("editor.duplicate", None)
        .expect("the action exists");
    let log = editor.log.borrow();
    assert_eq!(log[0], Intent::StartPreview(id("bass-boost")));
    assert_eq!(log[1], Intent::EndPreview);
    assert!(matches!(log[2], Intent::SavePreset(_)));
}

#[gtk::test]
fn closing_the_window_ends_the_preview() {
    let (window, log) = rendered_window(snapshot());
    window.present();
    let page = open_editor(&window, "flat");
    named::<gtk::ToggleButton>(&page, "preview").set_active(true);
    window.close();
    drain();
    assert_eq!(
        *log.borrow(),
        vec![Intent::StartPreview(id("flat")), Intent::EndPreview]
    );
}

#[gtk::test]
fn the_preview_turns_off_without_an_intent_when_the_companion_ends_it() {
    let editor = standalone_editor(snapshot(), "flat");
    let preview = named::<gtk::ToggleButton>(&editor.page, "preview");
    preview.set_active(true);
    editor.page.render(&Rc::new(picked("flat")));
    assert!(preview.is_active());
    editor
        .page
        .render(&Rc::new(with(Phase::Ready, Route::Bluetooth)));
    assert!(!preview.is_active());
    assert_eq!(*editor.log.borrow(), vec![Intent::StartPreview(id("flat"))]);
}

#[gtk::test]
fn a_render_before_the_pick_lands_keeps_the_preview() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    let preview = named::<gtk::ToggleButton>(&editor.page, "preview");
    preview.set_active(true);
    editor.page.render(&Rc::new(with_preset(&one_peak())));
    assert!(preview.is_active());
}
