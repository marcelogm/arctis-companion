use arctis_core::eq::FilterKind;
use arctis_core::testkit::english::english;
use arctis_core::view::eq_graph::{LevelRange, graph};
use arctis_core::view::eq_plot::{
    Emphasis, HANDLE_RADIUS, Nudge, Plot, drag_edit, frequency_ticks, hit_handle, level_ticks,
    nudged, scrolled_q, zone_label_rows, zones,
};
use arctis_core::view::preset_edits::BandEdit;

use crate::support::{assert_close, custom_preset, filter};

const PLOT: Plot = Plot {
    left: 40.0,
    top: 20.0,
    width: 600.0,
    height: 240.0,
    levels: LevelRange::RELATIVE,
};

fn close(actual: f32, expected: f32) {
    assert_close(f64::from(actual), f64::from(expected), 1e-3);
}

#[test]
fn frequencies_spread_on_a_log_scale_across_the_plot() {
    close(PLOT.x_of(20.0), 40.0);
    close(PLOT.x_of(632.455_5), 340.0);
    close(PLOT.x_of(20_000.0), 640.0);
    close(PLOT.x_of(5.0), 40.0);
    close(PLOT.hz_at(340.0), 632.455_5);
    close(PLOT.hz_at(0.0), 20.0);
    close(PLOT.hz_at(9_999.0), 20_000.0);
}

#[test]
fn levels_spread_linearly_with_the_top_of_the_range_on_top() {
    close(PLOT.y_of(12.0), 20.0);
    close(PLOT.y_of(0.0), 140.0);
    close(PLOT.y_of(-12.0), 260.0);
    close(PLOT.y_of(-30.0), 260.0);
    close(PLOT.db_at(80.0), 6.0);
    close(PLOT.db_at(-50.0), 12.0);
}

#[test]
fn a_plot_leaves_room_for_the_labels_and_never_goes_negative() {
    let plot = Plot::within(680.0, 290.0, LevelRange::RELATIVE);
    assert!(plot.left > 0.0 && plot.top > 0.0);
    assert!(plot.left + plot.width < 680.0 && plot.top + plot.height < 290.0);
    let tiny = Plot::within(4.0, 4.0, LevelRange::RELATIVE);
    assert!(tiny.width >= 0.0 && tiny.height >= 0.0);
}

#[test]
fn the_frequency_axis_labels_the_standard_points() {
    let ticks = frequency_ticks();
    let labels: Vec<String> = ticks
        .iter()
        .filter_map(|tick| tick.label.as_ref().map(english))
        .collect();
    assert_eq!(
        labels,
        [
            "20", "50", "100", "200", "500", "1k", "2k", "5k", "10k", "20k"
        ]
    );
    for tick in &ticks {
        assert_eq!(tick.label.is_some(), tick.emphasis == Emphasis::Major);
        assert!((20.0..=20_000.0).contains(&tick.value));
    }
    let minor: Vec<f32> = ticks
        .iter()
        .filter(|tick| tick.emphasis == Emphasis::Minor)
        .map(|tick| tick.value)
        .take(5)
        .collect();
    assert_eq!(minor, [30.0, 40.0, 60.0, 70.0, 80.0]);
}

#[test]
fn the_level_axis_labels_every_three_decibels_and_marks_zero() {
    let ticks = level_ticks(LevelRange::RELATIVE, 240.0);
    let values: Vec<f32> = ticks.iter().map(|tick| tick.value).collect();
    assert_eq!(values, [-12.0, -9.0, -6.0, -3.0, 0.0, 3.0, 6.0, 9.0, 12.0]);
    let labels: Vec<String> = ticks
        .iter()
        .filter_map(|tick| tick.label.as_ref().map(english))
        .collect();
    assert_eq!(
        labels,
        ["−12", "−9", "−6", "−3", "0", "+3", "+6", "+9", "+12"]
    );
    let zero: Vec<f32> = ticks
        .iter()
        .filter(|tick| tick.emphasis == Emphasis::Zero)
        .map(|tick| tick.value)
        .collect();
    assert_eq!(zero, [0.0]);
}

#[test]
fn a_short_plot_spaces_the_level_labels_further_apart() {
    let values: Vec<f32> = level_ticks(LevelRange::RELATIVE, 100.0)
        .iter()
        .map(|tick| tick.value)
        .collect();
    assert_eq!(values, [-12.0, -6.0, 0.0, 6.0, 12.0]);
    let spl = LevelRange {
        low_db: 40.0,
        high_db: 100.0,
    };
    let steps: Vec<f32> = level_ticks(spl, 120.0)
        .iter()
        .map(|tick| tick.value)
        .collect();
    assert_eq!(steps, [48.0, 60.0, 72.0, 84.0, 96.0]);
}

#[test]
fn reference_zones_cover_the_audible_range_in_order() {
    let zones = zones();
    let names: Vec<String> = zones.iter().map(|zone| english(&zone.name)).collect();
    assert_eq!(
        names,
        [
            "Sub-bass",
            "Bass",
            "Low mids",
            "Mids",
            "High mids",
            "Presence",
            "Brilliance"
        ]
    );
    let edges: Vec<(f32, f32)> = zones
        .iter()
        .map(|zone| (zone.low_hz, zone.high_hz))
        .collect();
    assert_eq!(
        edges,
        [
            (20.0, 60.0),
            (60.0, 250.0),
            (250.0, 500.0),
            (500.0, 2_000.0),
            (2_000.0, 4_000.0),
            (4_000.0, 6_000.0),
            (6_000.0, 20_000.0),
        ]
    );
}

#[test]
fn zone_labels_take_one_row_then_two_staggered_rows_then_hide() {
    assert_eq!(zone_label_rows(&PLOT, &[30.0; 7]), Some(1));
    assert_eq!(zone_label_rows(&PLOT, &[55.0; 7]), Some(2));
    let narrow = Plot {
        width: 200.0,
        ..PLOT
    };
    assert_eq!(zone_label_rows(&narrow, &[55.0; 7]), None);
    assert_eq!(zone_label_rows(&PLOT, &[30.0; 6]), None);
}

#[test]
fn a_press_near_a_handle_hits_the_nearest_one() {
    let handles = graph(&custom_preset(&[
        filter(FilterKind::Peaking, 1_000, 0, 1_000),
        filter(FilterKind::Peaking, 1_100, 0, 1_000),
    ]))
    .handles;
    let (x, y) = (PLOT.x_of(1_000.0), PLOT.y_of(0.0));
    assert_eq!(hit_handle(&PLOT, &handles, x - 3.0, y + 2.0), Some(0));
    assert_eq!(
        hit_handle(&PLOT, &handles, PLOT.x_of(1_100.0) + 1.0, y),
        Some(1)
    );
    assert_eq!(
        hit_handle(&PLOT, &handles, x, y + HANDLE_RADIUS + 1.0),
        None
    );
    const { assert!(HANDLE_RADIUS * 2.0 >= 24.0) };
}

#[test]
fn a_drag_moves_frequency_and_gain_from_where_the_handle_started() {
    let start = (PLOT.x_of(1_000.0), PLOT.y_of(0.0));
    let edit = drag_edit(&PLOT, start, (PLOT.x_of(2_000.0) - start.0, -20.0));
    let BandEdit::Point { hz, db } = edit else {
        panic!("expected a point edit, got {edit:?}");
    };
    assert_close(hz, 2_000.0, 0.5);
    assert_close(db, 2.0, 1e-3);
}

#[test]
fn keys_nudge_frequency_by_a_semitone_and_gain_by_half_a_decibel() {
    let peak = filter(FilterKind::Peaking, 1_000, 4, 1_000);
    let Some(BandEdit::Frequency(higher)) = nudged(peak, Nudge::Higher) else {
        panic!("expected a frequency edit");
    };
    assert_close(higher, 1_059.46, 0.01);
    let Some(BandEdit::Frequency(lower)) = nudged(peak, Nudge::Lower) else {
        panic!("expected a frequency edit");
    };
    assert_close(lower, 943.87, 0.01);
    assert_eq!(nudged(peak, Nudge::Up), Some(BandEdit::Gain(2.5)));
    assert_eq!(nudged(peak, Nudge::Down), Some(BandEdit::Gain(1.5)));
    let pass = filter(FilterKind::LowPass, 8_000, 0, 707);
    assert_eq!(nudged(pass, Nudge::Up), None);
}

#[test]
fn q_grows_narrower_up_and_wider_down_within_the_envelope() {
    let peak = filter(FilterKind::Peaking, 1_000, 4, 1_000);
    let Some(BandEdit::Q(narrower)) = nudged(peak, Nudge::Narrower) else {
        panic!("expected a Q edit");
    };
    let Some(BandEdit::Q(wider)) = nudged(peak, Nudge::Wider) else {
        panic!("expected a Q edit");
    };
    assert!(narrower > 1.0 && wider < 1.0);
    assert_close(narrower * wider, 1.0, 1e-9);
    assert_eq!(scrolled_q(peak, 0.0), BandEdit::Q(1.0));
    let BandEdit::Q(more) = scrolled_q(peak, 2.0) else {
        panic!("expected a Q edit");
    };
    assert_close(more, narrower * narrower, 1e-9);
}
