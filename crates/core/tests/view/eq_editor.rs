use arctis_core::companion::routing::Route;
use arctis_core::companion::selection::{ActivePreset, PresetReason};
use arctis_core::eq::FilterKind;
use arctis_core::eq::preset::Preset;
use arctis_core::messages::device::Phase;
use arctis_core::testkit::english::english;
use arctis_core::view::eq_editor::{
    PreviewState, band_title, eq_editor, eq_editor_for, kind_at, kind_items, kind_position,
};

use crate::support::{custom_preset, filter, id, snapshot, with};

fn two_bands() -> Preset {
    custom_preset(&[
        filter(FilterKind::Peaking, 250, 4, 1_000),
        filter(FilterKind::LowPass, 8_000, 0, 707),
    ])
}

#[test]
fn a_built_in_preset_opens_read_only() {
    let editor = eq_editor(&snapshot(), &id("flat")).expect("editor");
    assert!(!editor.editable);
    assert_eq!(editor.bands.len(), 10);
    assert!(!editor.can_add_band);
    assert_eq!(english(&editor.title), "Flat");
}

#[test]
fn a_custom_preset_is_editable_with_room_for_bands() {
    let mut s = snapshot();
    s.presets.all.push(two_bands());
    let editor = eq_editor(&s, &id("mine")).expect("editor");
    assert!(editor.editable && editor.can_add_band);
    assert_eq!(editor.bands.len(), 2);
    assert_eq!(editor.bands[1].index, 1);
}

#[test]
fn band_titles_show_frequency_and_gain_or_kind() {
    assert_eq!(
        english(&band_title(filter(FilterKind::Peaking, 250, 4, 1_000))),
        "250 Hz · +2.0 dB"
    );
    assert_eq!(
        english(&band_title(filter(FilterKind::LowPass, 8_000, 0, 707))),
        "8000 Hz · Low-Pass"
    );
}

#[test]
fn pass_filters_have_no_gain_control() {
    let mut s = snapshot();
    s.presets.all.push(two_bands());
    let editor = eq_editor(&s, &id("mine")).expect("editor");
    let sensitive: Vec<bool> = editor
        .bands
        .iter()
        .map(|band| band.gain_sensitive)
        .collect();
    assert_eq!(sensitive, [true, false]);
}

#[test]
fn the_graph_describes_the_nominal_curve() {
    let cases = [
        (
            "bass-boost",
            "Nominal frequency response of Bass Boost, with 1 active band",
        ),
        (
            "flat",
            "Nominal frequency response of Flat, with 10 active bands",
        ),
    ];
    for (preset, expected) in cases {
        let graph = eq_editor(&snapshot(), &id(preset)).expect("editor").graph;
        assert_eq!(english(&graph.description), expected);
    }
}

#[test]
fn preview_is_offered_only_with_a_picked_path() {
    assert!(
        eq_editor(&snapshot(), &id("flat"))
            .expect("editor")
            .preview
            .is_available()
    );
    let alone = with(Phase::Absent, Route::None);
    assert!(
        !eq_editor(&alone, &id("flat"))
            .expect("editor")
            .preview
            .is_available()
    );
}

#[test]
fn the_editor_knows_when_its_preset_is_the_manual_pick() {
    let picked = |s: &arctis_core::companion::Snapshot, preset: &str| {
        eq_editor(s, &id(preset)).expect("editor").preview == PreviewState::Picked
    };
    let mut s = snapshot();
    assert!(!picked(&s, "flat"));
    s.presets.active.dongle = ActivePreset {
        preset: id("flat"),
        reason: PresetReason::Manual(None),
    };
    assert!(picked(&s, "flat"));
    assert!(!picked(&s, "bass-boost"));
    s.audio.route = Route::Bluetooth;
    assert!(!picked(&s, "flat"));
}

#[test]
fn an_unknown_preset_has_no_editor() {
    assert!(eq_editor(&snapshot(), &id("nope")).is_none());
}

#[test]
fn every_filter_kind_has_a_name() {
    let names: Vec<String> = kind_items().iter().map(english).collect();
    assert_eq!(
        names,
        ["Peak", "Low Shelf", "High Shelf", "Low-Pass", "High-Pass"]
    );
}

#[test]
fn the_editor_model_can_come_from_a_working_copy() {
    let s = snapshot();
    let editor = eq_editor_for(&s, &two_bands());
    assert_eq!(editor.id, id("mine"));
    assert_eq!(editor.bands.len(), 2);
    assert!(editor.editable);
    assert_eq!(eq_editor(&s, &id("mine")), None);
}

#[test]
fn filter_kinds_map_to_combo_positions() {
    for (position, kind) in FilterKind::ALL.into_iter().enumerate() {
        assert_eq!(kind_position(kind), position);
        assert_eq!(kind_at(position), Some(kind));
    }
    assert_eq!(kind_at(FilterKind::ALL.len()), None);
    assert_eq!(kind_items().len(), FilterKind::ALL.len());
}
