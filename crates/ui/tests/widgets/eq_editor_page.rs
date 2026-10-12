use std::rc::Rc;

use adw::prelude::*;
use arctis_core::eq::FilterKind;
use arctis_core::eq::library;
use arctis_core::eq::preset::Preset;
use arctis_core::intent::Intent;
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::{id, snapshot};
use arctis_core::view::eq_editor::{eq_editor_for, kind_position};
use arctis_core::view::preset_edits::NEW_BAND;
use arctis_ui::{EqBandRow, ResponseGraph};
use gtk::AccessibleProperty;

use crate::support::{
    Mounted, assert_close, band_rows, click, count_text, custom_preset, dismiss_toast, drain,
    filter, first, has_accessible_text, named, one_peak, open_editor, rendered_window, saved,
    shows_text, standalone_editor, step_up, with_preset,
};

const EDIT_CONTROLS: [&str; 6] = [
    "band-kind",
    "band-frequency",
    "band-gain",
    "band-q",
    "band-enabled",
    "band-remove",
];

fn ten_peaks() -> Preset {
    Preset::custom(
        id("mine"),
        "Mine".to_owned(),
        library::flat().curve().clone(),
    )
}

fn spin(row: &EqBandRow, name: &str) -> adw::SpinRow {
    named::<adw::SpinRow>(row, name)
}

fn saved_band(editor: &Mounted<arctis_ui::EqEditorPage>, index: usize) -> arctis_core::eq::Filter {
    saved(&editor.log).curve().filters()[index]
}

#[gtk::test]
fn a_built_in_preset_opens_read_only() {
    let editor = standalone_editor(snapshot(), "flat");
    let page = &editor.page;
    assert_eq!(page.title(), "Flat");
    assert_eq!(band_rows(page).len(), 10);
    assert!(named::<gtk::Button>(page, "duplicate").is_visible());
    assert!(!named::<gtk::MenuButton>(page, "editor-menu").is_visible());
    assert!(!named::<gtk::Button>(page, "add-band").is_visible());
    assert!(!spin(&band_rows(page)[0], "band-frequency").is_sensitive());
    assert!(editor.log.borrow().is_empty());
}

#[gtk::test]
fn a_custom_preset_opens_editable() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    let page = &editor.page;
    assert_eq!(page.title(), "Mine");
    assert_eq!(band_rows(page).len(), 1);
    assert!(!named::<gtk::Button>(page, "duplicate").is_visible());
    assert!(named::<gtk::MenuButton>(page, "editor-menu").is_visible());
    let add_band = named::<gtk::Button>(page, "add-band");
    assert!(add_band.is_visible() && add_band.is_focusable());
    assert!(spin(&band_rows(page)[0], "band-frequency").is_sensitive());
    assert!(editor.files.saved.borrow().is_empty());
}

#[gtk::test]
fn the_graph_describes_the_shown_preset() {
    let s = with_preset(&one_peak());
    let editor = standalone_editor(s.clone(), "mine");
    let description = eq_editor_for(&s, &one_peak()).graph.description;
    assert!(has_accessible_text(
        &first::<ResponseGraph>(&editor.page),
        AccessibleProperty::Description,
        &english(&description)
    ));
}

#[gtk::test]
fn low_pass_and_high_pass_bands_have_no_gain_control() {
    let preset = custom_preset(&[
        filter(FilterKind::Peaking, 250, 4, 1_000),
        filter(FilterKind::LowPass, 8_000, 0, 707),
        filter(FilterKind::HighPass, 30, 0, 707),
    ]);
    let editor = standalone_editor(with_preset(&preset), "mine");
    let sensitive: Vec<bool> = band_rows(&editor.page)
        .iter()
        .map(|row| spin(row, "band-gain").is_sensitive())
        .collect();
    assert_eq!(sensitive, [true, false, false]);
}

#[gtk::test]
fn an_edit_saves_at_once_and_offers_undo() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    step_up(&spin(&band_rows(&editor.page)[0], "band-gain"));
    assert_eq!(saved_band(&editor, 0).gain().half_db(), 5);
    assert_eq!(band_rows(&editor.page)[0].title(), "250 Hz · +2.5 dB");
    assert!(shows_text(&editor.toasts, "Saved “Mine”"));
    click(&editor.toasts, "Undo");
    assert_eq!(saved(&editor.log), one_peak());
    assert_eq!(editor.log.borrow().len(), 2);
    assert_eq!(band_rows(&editor.page)[0].title(), "250 Hz · +2.0 dB");
}

#[gtk::test]
fn one_toast_covers_a_burst_of_edits() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    let gain = spin(&band_rows(&editor.page)[0], "band-gain");
    step_up(&gain);
    step_up(&gain);
    assert_eq!(saved_band(&editor, 0).gain().half_db(), 6);
    assert_eq!(count_text(&editor.toasts, "Saved “Mine”"), 1);
    click(&editor.toasts, "Undo");
    drain();
    assert_eq!(saved(&editor.log), one_peak());
    assert!(!shows_text(&editor.toasts, "Saved “Mine”"));
}

#[gtk::test]
fn undo_after_a_pause_restores_the_start_of_the_new_burst() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    step_up(&spin(&band_rows(&editor.page)[0], "band-gain"));
    let after_first_burst = saved(&editor.log);
    dismiss_toast(&editor.toasts);
    step_up(&spin(&band_rows(&editor.page)[0], "band-q"));
    assert!(shows_text(&editor.toasts, "Saved “Mine”"));
    click(&editor.toasts, "Undo");
    assert_eq!(saved(&editor.log), after_first_burst);
}

#[gtk::test]
fn switching_presets_keeps_one_toast_per_burst() {
    let other = Preset::custom(id("other"), "Other".to_owned(), one_peak().curve().clone());
    let mut s = with_preset(&one_peak());
    s.presets.all.push(other);
    let editor = standalone_editor(s.clone(), "mine");
    step_up(&spin(&band_rows(&editor.page)[0], "band-gain"));
    editor.page.open(&Rc::new(s), &id("other"));
    drain();
    let gain = spin(&band_rows(&editor.page)[0], "band-gain");
    step_up(&gain);
    drain();
    step_up(&gain);
    assert_eq!(count_text(&editor.toasts, "Saved “Other”"), 1);
}

fn edit_every_field(editor: &Mounted<arctis_ui::EqEditorPage>, index: usize) {
    let row = band_rows(&editor.page).remove(index);
    let before = ten_peaks().curve().filters()[index];
    step_up(&spin(&row, "band-frequency"));
    assert_eq!(saved_band(editor, index).hz().get(), before.hz().get() + 1);
    step_up(&spin(&row, "band-gain"));
    assert_eq!(saved_band(editor, index).gain().half_db(), 1);
    step_up(&spin(&row, "band-q"));
    assert_eq!(
        saved_band(editor, index).q().milli(),
        before.q().milli() + 10
    );
    let shelf = u32::try_from(kind_position(FilterKind::HighShelf)).expect("small position");
    named::<adw::ComboRow>(&row, "band-kind").set_selected(shelf);
    assert_eq!(saved_band(editor, index).kind(), FilterKind::HighShelf);
    named::<adw::SwitchRow>(&row, "band-enabled").set_active(false);
    assert!(!saved_band(editor, index).is_enabled());
}

#[gtk::test]
fn every_band_is_editable_from_the_keyboard() {
    let editor = standalone_editor(with_preset(&ten_peaks()), "mine");
    for (index, row) in band_rows(&editor.page).iter().enumerate() {
        row.set_expanded(true);
        for name in EDIT_CONTROLS {
            let control = named::<gtk::Widget>(row, name);
            assert!(
                control.is_sensitive() && control.is_focusable(),
                "{name} of band {index}"
            );
        }
        edit_every_field(&editor, index);
    }
    assert_eq!(editor.log.borrow().len(), 50);
}

#[gtk::test]
fn a_kind_change_to_low_pass_turns_the_gain_off() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    let row = band_rows(&editor.page).remove(0);
    let low_pass = u32::try_from(kind_position(FilterKind::LowPass)).expect("small position");
    named::<adw::ComboRow>(&row, "band-kind").set_selected(low_pass);
    let band = saved_band(&editor, 0);
    assert_eq!(band.kind(), FilterKind::LowPass);
    assert_eq!(band.gain().half_db(), 0);
    assert!(!spin(&band_rows(&editor.page)[0], "band-gain").is_sensitive());
}

#[gtk::test]
fn add_band_appends_a_neutral_peak() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    editor
        .page
        .activate_action("editor.add-band", None)
        .expect("the action exists");
    assert_eq!(saved_band(&editor, 1), NEW_BAND);
    assert_eq!(band_rows(&editor.page).len(), 2);
}

#[gtk::test]
fn add_band_does_nothing_at_ten_bands() {
    let editor = standalone_editor(with_preset(&ten_peaks()), "mine");
    editor.page.activate_action("editor.add-band", None).ok();
    assert!(editor.log.borrow().is_empty());
    assert_eq!(band_rows(&editor.page).len(), 10);
}

#[gtk::test]
fn the_add_button_is_sensitive_only_while_there_is_room() {
    let sensitive = |preset: &Preset| {
        let (window, _) = rendered_window(with_preset(preset));
        let page = open_editor(&window, "mine");
        named::<gtk::Button>(&page, "add-band").is_sensitive()
    };
    assert!(sensitive(&one_peak()));
    assert!(!sensitive(&ten_peaks()));
}

#[gtk::test]
fn remove_band_drops_that_band() {
    let preset = custom_preset(&[
        filter(FilterKind::Peaking, 250, 4, 1_000),
        filter(FilterKind::LowPass, 8_000, 0, 707),
    ]);
    let editor = standalone_editor(with_preset(&preset), "mine");
    let second = band_rows(&editor.page).remove(1);
    named::<adw::ButtonRow>(&second, "band-remove").emit_by_name::<()>("activated", &[]);
    assert_eq!(
        saved(&editor.log).curve().filters(),
        [preset.curve().filters()[0]]
    );
    assert_eq!(band_rows(&editor.page).len(), 1);
}

#[gtk::test]
fn a_published_preset_replaces_the_working_copy_without_an_intent() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    let renamed = Preset::custom(id("mine"), "Night".to_owned(), one_peak().curve().clone());
    editor.page.render(&Rc::new(with_preset(&renamed)));
    assert_eq!(editor.page.title(), "Night");
    assert!(editor.log.borrow().is_empty());
}

#[gtk::test]
fn rendering_emits_nothing() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    editor.page.render(&Rc::new(snapshot()));
    editor.page.render(&Rc::new(with_preset(&ten_peaks())));
    assert!(editor.log.borrow().is_empty());
}

#[gtk::test]
fn intents_are_only_saves() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    step_up(&spin(&band_rows(&editor.page)[0], "band-q"));
    assert!(matches!(
        editor.log.borrow().as_slice(),
        [Intent::SavePreset(_)]
    ));
}

#[gtk::test]
fn an_off_grid_gain_that_saves_nothing_shows_the_saved_value() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    let gain = spin(&band_rows(&editor.page)[0], "band-gain");
    gain.set_value(2.2);
    assert_close(gain.value(), 2.0);
    assert!(editor.log.borrow().is_empty());
}

fn two_peaks() -> Preset {
    custom_preset(&[
        filter(FilterKind::Peaking, 250, 4, 1_000),
        filter(FilterKind::Peaking, 4_000, -2, 1_000),
    ])
}

#[gtk::test]
fn removing_a_band_leaves_the_next_band_collapsed_and_focused() {
    let (window, _) = rendered_window(with_preset(&two_peaks()));
    window.present();
    let page = open_editor(&window, "mine");
    let removed = band_rows(&page).remove(0);
    let next_title = band_rows(&page)[1].title();
    removed.set_expanded(true);
    let remove = named::<adw::ButtonRow>(&removed, "band-remove");
    assert!(remove.grab_focus());
    remove.emit_by_name::<()>("activated", &[]);
    let rows = band_rows(&page);
    assert_eq!(rows[0].title(), next_title);
    assert!(!rows[0].is_expanded());
    let focus = gtk::prelude::RootExt::focus(&window).expect("a focused widget");
    assert!(focus.is_ancestor(&rows[0]) || focus == *rows[0].upcast_ref::<gtk::Widget>());
    assert_ne!(focus.widget_name(), "band-remove");
}
