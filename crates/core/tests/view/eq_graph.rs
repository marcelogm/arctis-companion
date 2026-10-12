use arctis_core::eq::FilterKind;
use arctis_core::eq::preset::Preset;
use arctis_core::eq::reference::Overlays;
use arctis_core::testkit::english::english;
use arctis_core::view::eq_editor::eq_editor;
use arctis_core::view::eq_graph::{
    ColorRole, GRAPH_POINTS, LevelRange, LineStyle, graph, handle_label,
};

use crate::support::{assert_close, custom_preset, filter, id, snapshot};

fn two_bands() -> Preset {
    custom_preset(&[
        filter(FilterKind::Peaking, 250, 4, 1_000),
        filter(FilterKind::LowPass, 8_000, 0, 707),
    ])
}

#[test]
fn the_response_is_one_filled_accent_series_from_twenty_hertz_to_twenty_kilohertz() {
    let mut bare = snapshot();
    bare.presets.overlays = Overlays::NONE;
    let model = eq_editor(&bare, &id("flat")).expect("editor").graph;
    assert_eq!(model.series.len(), 1);
    let response = &model.series[0];
    assert_eq!(response.style, LineStyle::Solid);
    assert_eq!(response.role, ColorRole::Accent);
    assert!(response.filled);
    assert_eq!(response.label, None);
    assert_eq!(response.points.len(), usize::from(GRAPH_POINTS));
    let first = response.points.first().expect("first point");
    let last = response.points.last().expect("last point");
    assert_close(f64::from(first.hz), 20.0, 0.01);
    assert_close(f64::from(last.hz), 20_000.0, 1.0);
    for point in &response.points {
        assert_close(f64::from(point.db), 0.0, 0.001);
    }
}

#[test]
fn the_level_range_holds_the_whole_gain_envelope() {
    let model = graph(&two_bands());
    assert_eq!(model.levels, LevelRange::RELATIVE);
    assert!(model.levels.low_db <= -10.0 && model.levels.high_db >= 10.0);
}

#[test]
fn each_band_has_a_numbered_handle_at_its_frequency_and_gain() {
    let handles = graph(&two_bands()).handles;
    assert_eq!(handles.len(), 2);
    assert_eq!((handles[0].index, handles[0].number), (0, 1));
    assert_eq!(handles[0].filter, two_bands().curve().filters()[0]);
    assert_close(f64::from(handles[0].hz), 250.0, 1e-6);
    assert_close(f64::from(handles[0].db), 2.0, 1e-6);
    assert!(handles[0].gain_adjustable && handles[0].enabled);
    assert_eq!((handles[1].index, handles[1].number), (1, 2));
    assert_close(f64::from(handles[1].db), 0.0, 1e-6);
    assert!(!handles[1].gain_adjustable);
}

#[test]
fn a_handle_reads_out_its_band() {
    let handles = graph(&two_bands()).handles;
    assert_eq!(
        english(&handles[0].label),
        "Band 1, Peak, 250 Hz, +2.0 dB, Q 1.00"
    );
    assert_eq!(
        english(&handles[1].label),
        "Band 2, Low-Pass, 8000 Hz, Q 0.71"
    );
    assert_eq!(
        handles[0].label,
        handle_label(two_bands().curve().filters()[0], 0)
    );
}

#[test]
fn a_disabled_band_keeps_a_handle_marked_off() {
    let off = filter(FilterKind::Peaking, 250, 4, 1_000).with_enabled(false);
    let handles = graph(&custom_preset(&[off])).handles;
    assert!(!handles[0].enabled);
}
