use arctis_core::companion::routing::Route;
use arctis_core::eq::preset::Preset;
use arctis_core::eq::{Curve, Filter, FilterKind};
use arctis_core::messages::device::Phase;
use arctis_core::testkit::english::english;
use arctis_core::view::eq_editor::{GRAPH_POINTS, band_title, eq_editor, kind_items};

use crate::support::{assert_close, id, snapshot, with};

fn filter(kind: FilterKind, hz: u16, half_db: i8, milli_q: u16) -> Filter {
    Filter::from_parts(kind, hz, half_db, milli_q).expect("filter in the envelope")
}

fn two_bands() -> Preset {
    let bands = [
        filter(FilterKind::Peaking, 250, 4, 1_000),
        filter(FilterKind::LowPass, 8_000, 0, 707),
    ];
    Preset::custom(id("mine"), "Mine".to_owned(), Curve::from_array(bands))
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
fn the_graph_spans_twenty_hertz_to_twenty_kilohertz() {
    let graph = eq_editor(&snapshot(), &id("flat")).expect("editor").graph;
    assert_eq!(graph.points.len(), usize::from(GRAPH_POINTS));
    let first = graph.points.first().expect("first point");
    let last = graph.points.last().expect("last point");
    assert_close(f64::from(first.hz), 20.0, 0.01);
    assert_close(f64::from(last.hz), 20_000.0, 1.0);
    for point in &graph.points {
        assert_close(f64::from(point.db), 0.0, 0.001);
    }
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
            .preview_available
    );
    let alone = with(Phase::Absent, Route::None);
    assert!(
        !eq_editor(&alone, &id("flat"))
            .expect("editor")
            .preview_available
    );
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
