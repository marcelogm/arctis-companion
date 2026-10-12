use arctis_core::eq::library;
use arctis_core::eq::preset::{Preset, PresetKind};
use arctis_core::eq::{Filter, FilterKind};
use arctis_core::testkit::english::english;
use arctis_core::view::preset_edits::{
    Applied, BandEdit, NEW_BAND, WorkingCopy, add_band, apply_edit, copy_name, duplicate,
    new_preset, remove_band, renamed, saved_toast,
};

use crate::support::{custom, custom_preset, filter, id, snapshot, with_preset};

fn peak() -> Preset {
    custom_preset(&[filter(FilterKind::Peaking, 250, 4, 1_000)])
}

fn first_band(preset: &Preset) -> Filter {
    preset.curve().filters()[0]
}

fn gain_up(preset: &Preset) -> Option<Preset> {
    apply_edit(
        preset,
        0,
        BandEdit::Gain(first_band(preset).gain().db() + 0.5),
    )
}

#[test]
fn a_gain_edit_snaps_to_half_decibels_and_keeps_the_identity() {
    let edited = apply_edit(&peak(), 0, BandEdit::Gain(2.7)).expect("edited");
    assert_eq!(first_band(&edited).gain().half_db(), 5);
    assert_eq!(edited.id(), &id("mine"));
    assert_eq!(edited.name(), "Mine");
    assert_eq!(edited.kind(), PresetKind::Custom);
}

#[test]
fn frequency_and_q_edits_stay_inside_the_envelope() {
    let low = apply_edit(&peak(), 0, BandEdit::Frequency(5.0)).expect("edited");
    assert_eq!(first_band(&low).hz().get(), 20);
    let precise = apply_edit(&peak(), 0, BandEdit::Q(0.707)).expect("edited");
    assert_eq!(first_band(&precise).q().milli(), 707);
    let widest = apply_edit(&peak(), 0, BandEdit::Q(40.0)).expect("edited");
    assert_eq!(first_band(&widest).q().milli(), 10_000);
}

#[test]
fn a_pass_filter_drops_its_gain_and_ignores_gain_edits() {
    let low_pass = apply_edit(&peak(), 0, BandEdit::Kind(FilterKind::LowPass)).expect("edited");
    assert_eq!(first_band(&low_pass).kind(), FilterKind::LowPass);
    assert_eq!(first_band(&low_pass).gain().half_db(), 0);
    assert_eq!(apply_edit(&low_pass, 0, BandEdit::Gain(3.0)), None);
}

#[test]
fn disabling_a_band_keeps_its_values() {
    let off = apply_edit(&peak(), 0, BandEdit::Enabled(false)).expect("edited");
    assert!(!first_band(&off).is_enabled());
    assert_eq!(first_band(&off).with_enabled(true), first_band(&peak()));
}

#[test]
fn an_unchanged_value_or_a_missing_band_is_no_edit() {
    assert_eq!(apply_edit(&peak(), 0, BandEdit::Frequency(250.0)), None);
    assert_eq!(apply_edit(&peak(), 0, BandEdit::Gain(2.0)), None);
    assert_eq!(apply_edit(&peak(), 0, BandEdit::Q(1.0)), None);
    assert_eq!(apply_edit(&peak(), 3, BandEdit::Gain(1.0)), None);
    assert_eq!(apply_edit(&peak(), 0, BandEdit::Gain(f64::NAN)), None);
}

#[test]
fn built_in_presets_take_no_edits() {
    let flat = library::flat();
    assert_eq!(apply_edit(&flat, 0, BandEdit::Gain(3.0)), None);
    assert_eq!(add_band(&flat), None);
    assert_eq!(remove_band(&flat, 0), None);
    assert_eq!(renamed(&flat, "Mine"), None);
}

#[test]
fn bands_are_added_as_a_neutral_peak_up_to_ten() {
    let two = add_band(&peak()).expect("added");
    assert_eq!(two.curve().filters(), [first_band(&peak()), NEW_BAND]);
    assert_eq!(NEW_BAND, filter(FilterKind::Peaking, 1_000, 0, 1_410));
    assert_eq!(add_band(&custom("Full")), None);
}

#[test]
fn a_band_is_removed_by_position() {
    let pass = filter(FilterKind::LowPass, 8_000, 0, 707);
    let two = custom_preset(&[filter(FilterKind::Peaking, 250, 4, 1_000), pass]);
    let one = remove_band(&two, 0).expect("removed");
    assert_eq!(one.curve().filters(), [pass]);
    assert_eq!(remove_band(&two, 2), None);
}

#[test]
fn renaming_trims_and_ignores_empty_or_unchanged_names() {
    let renamed_preset = renamed(&peak(), "  Night  ").expect("renamed");
    assert_eq!(renamed_preset.name(), "Night");
    assert_eq!(renamed_preset.id(), &id("mine"));
    assert_eq!(renamed(&peak(), "   "), None);
    assert_eq!(renamed(&peak(), "Mine"), None);
}

#[test]
fn a_duplicate_is_a_custom_copy_under_a_fresh_id() {
    let source = library::find(&id("bass-boost")).expect("a built-in");
    let name = english(&copy_name(&source));
    assert_eq!(name, "Bass Boost (Copy)");
    let copy = duplicate(&snapshot(), &source, &name);
    assert_eq!(copy.id(), &id("bass-boost-copy"));
    assert_eq!(copy.name(), "Bass Boost (Copy)");
    assert_eq!(copy.kind(), PresetKind::Custom);
    assert_eq!(copy.curve(), source.curve());
}

#[test]
fn a_new_preset_is_a_flat_custom_preset() {
    let preset = new_preset(&snapshot(), english);
    assert_eq!(preset.name(), "Custom Preset");
    assert_eq!(preset.id(), &id("custom-preset"));
    assert_eq!(preset.kind(), PresetKind::Custom);
    assert_eq!(preset.curve(), library::flat().curve());
}

#[test]
fn a_new_preset_takes_the_first_free_numbered_name() {
    let mut s = snapshot();
    s.presets.all.push(custom("Custom Preset"));
    s.presets.all.push(custom("Custom Preset 3"));
    let preset = new_preset(&s, english);
    assert_eq!(preset.name(), "Custom Preset 2");
    assert!(s.presets.find(preset.id()).is_none());
}

#[test]
fn a_new_preset_keeps_the_base_name_when_numbering_never_frees_one() {
    let mut s = snapshot();
    s.presets.all.push(custom("Custom Preset"));
    let preset = new_preset(&s, |_| "Custom Preset".to_owned());
    assert_eq!(preset.name(), "Custom Preset");
    assert!(s.presets.find(preset.id()).is_none());
}

#[test]
fn the_saved_toast_names_the_preset() {
    assert_eq!(english(&saved_toast(&peak())), "Saved “Mine”");
}

#[test]
fn the_first_edit_of_a_burst_starts_it_and_later_ones_extend_it() {
    let mut copy = WorkingCopy::opened(peak());
    let first = copy.apply(gain_up).expect("applied");
    assert!(first.starts_burst);
    assert_eq!(first.preset, *copy.preset());
    let second = copy.apply(gain_up).expect("applied");
    assert!(!second.starts_burst);
    assert_eq!(first_band(copy.preset()).gain().half_db(), 6);
}

#[test]
fn a_change_that_edits_nothing_neither_applies_nor_starts_a_burst() {
    let mut copy = WorkingCopy::opened(peak());
    assert_eq!(copy.apply(|_| None), None);
    let applied: Applied = copy.apply(gain_up).expect("applied");
    assert!(applied.starts_burst);
}

#[test]
fn undo_restores_the_preset_as_it_was_before_the_burst() {
    let mut copy = WorkingCopy::opened(peak());
    copy.apply(gain_up);
    copy.apply(gain_up);
    assert_eq!(copy.undo(), Some(peak()));
    assert_eq!(copy.preset(), &peak());
    assert_eq!(copy.undo(), None);
}

#[test]
fn a_new_burst_after_a_pause_undoes_only_itself() {
    let mut copy = WorkingCopy::opened(peak());
    copy.apply(gain_up);
    let after_first = copy.preset().clone();
    copy.burst_ended();
    let second = copy.apply(gain_up).expect("applied");
    assert!(second.starts_burst);
    assert_eq!(copy.undo(), Some(after_first));
}

#[test]
fn reopening_forgets_the_previous_burst() {
    let mut copy = WorkingCopy::opened(peak());
    copy.apply(gain_up);
    let other = Preset::custom(id("other"), "Other".to_owned(), peak().curve().clone());
    copy = WorkingCopy::opened(other.clone());
    assert_eq!(copy.undo(), None);
    assert!(copy.apply(gain_up).expect("applied").starts_burst);
}

#[test]
fn a_published_copy_replaces_the_working_copy_only_when_it_exists() {
    let mut copy = WorkingCopy::opened(peak());
    copy.published(&snapshot());
    assert_eq!(copy.preset(), &peak());
    let night = Preset::custom(id("mine"), "Night".to_owned(), peak().curve().clone());
    copy.published(&with_preset(&night));
    assert_eq!(copy.preset(), &night);
}
