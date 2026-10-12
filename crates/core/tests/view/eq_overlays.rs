use arctis_core::eq::library;
use arctis_core::eq::preset::Preset;
use arctis_core::eq::reference::{Overlays, SeriesId, Target};
use arctis_core::testkit::english::english;
use arctis_core::view::eq_editor::{eq_editor_for, eq_editor_with};
use arctis_core::view::eq_graph::{
    ColorRole, GraphModel, LevelRange, LineStyle, Series, Weight, graph, graph_with,
};
use arctis_core::view::eq_overlays::{OVERLAID, level_range, summary};

use crate::support::{assert_close, id, snapshot};

const ALL: Overlays = Overlays {
    headset: true,
    target: Some(Target::Harman),
};

fn built_in(slug: &str) -> Preset {
    library::find(&id(slug)).expect("built-in preset")
}

fn find(model: &GraphModel, reference: SeriesId) -> &Series {
    model
        .series
        .iter()
        .find(|series| series.reference == Some(reference))
        .unwrap_or_else(|| panic!("no {reference:?} series"))
}

fn references(model: &GraphModel) -> Vec<Option<SeriesId>> {
    model.series.iter().map(|series| series.reference).collect()
}

fn rms_to_target(preset: &Preset) -> f64 {
    let model = graph_with(preset, ALL);
    let target = find(&model, SeriesId::Target(Target::Harman));
    let equalized = find(&model, SeriesId::HeadsetWithEq);
    let squares: Vec<f64> = equalized
        .points
        .iter()
        .zip(&target.points)
        .filter(|(point, _)| point.hz <= 10_000.0)
        .map(|(point, aim)| f64::from(point.db - aim.db).powi(2))
        .collect();
    (squares.iter().sum::<f64>() / f64::from(u32::try_from(squares.len()).expect("count"))).sqrt()
}

#[test]
fn without_overlays_the_graph_is_the_equalizer_alone() {
    let model = graph(&built_in("bass-boost"));
    assert_eq!(model, graph_with(&built_in("bass-boost"), Overlays::NONE));
    assert_eq!(references(&model), [None]);
    assert_eq!(model.series[0].label, None);
    assert_eq!(model.levels, LevelRange::RELATIVE);
    assert!(model.legend.is_empty());
}

#[test]
fn overlays_draw_target_then_headset_then_equalized_under_the_equalizer() {
    let model = graph_with(&built_in("flat"), ALL);
    assert_eq!(
        references(&model),
        [
            Some(SeriesId::Target(Target::Harman)),
            Some(SeriesId::Headset),
            Some(SeriesId::HeadsetWithEq),
            None,
        ]
    );
    let equalizer = model.series.last().expect("equalizer");
    assert!(equalizer.filled);
    assert_eq!(equalizer.role, ColorRole::Accent);
    assert_eq!(equalizer.weight, Weight::Faint);
    let plain = graph(&built_in("flat"));
    assert_eq!(plain.series[0].weight, Weight::Regular);
}

#[test]
fn each_overlay_has_its_own_look() {
    let model = graph_with(&built_in("flat"), ALL);
    let look = |reference| {
        let series = find(&model, reference);
        (series.style, series.role, series.weight, series.filled)
    };
    assert_eq!(
        look(SeriesId::Headset),
        (
            LineStyle::Solid,
            ColorRole::Foreground,
            Weight::Faint,
            false
        )
    );
    assert_eq!(
        look(SeriesId::HeadsetWithEq),
        (LineStyle::Solid, ColorRole::Accent, Weight::Bold, false)
    );
    assert_eq!(
        look(SeriesId::Target(Target::Harman)),
        (
            LineStyle::Dashed,
            ColorRole::Foreground,
            Weight::Regular,
            false
        )
    );
}

#[test]
fn overlays_span_the_audible_range_on_the_measurement_grid() {
    let model = graph_with(&built_in("flat"), ALL);
    for reference in [SeriesId::Headset, SeriesId::Target(Target::Harman)] {
        let points = &find(&model, reference).points;
        assert_eq!(points.len(), 241, "{reference:?}");
        assert_close(f64::from(points[0].hz), 20.0, 1e-3);
        assert_close(f64::from(points[240].hz), 20_000.0, 0.5);
    }
}

#[test]
fn each_choice_shows_only_what_it_names() {
    let flat = built_in("flat");
    let headset = graph_with(&flat, Overlays::NONE.with_headset(true));
    assert_eq!(
        references(&headset),
        [Some(SeriesId::Headset), Some(SeriesId::HeadsetWithEq), None]
    );
    let diffuse = graph_with(&flat, Overlays::NONE.with_target_setting("diffuse-field"));
    assert_eq!(
        references(&diffuse),
        [Some(SeriesId::Target(Target::DiffuseField)), None]
    );
}

#[test]
fn the_level_range_widens_to_hold_the_targets() {
    assert_eq!(level_range(Overlays::NONE), LevelRange::RELATIVE);
    assert_eq!(level_range(ALL), OVERLAID);
    assert_eq!((OVERLAID.low_db, OVERLAID.high_db), (-18.0, 12.0));
    let model = graph_with(&built_in("flat"), ALL);
    assert_eq!(model.levels, OVERLAID);
    for target in Target::ALL {
        let overlays = Overlays::NONE.with_target_setting(target.setting());
        let model = graph_with(&built_in("flat"), overlays);
        let peak = find(&model, SeriesId::Target(target))
            .points
            .iter()
            .map(|point| point.db)
            .fold(f32::MIN, f32::max);
        assert!(peak < OVERLAID.high_db, "{target:?} peaks at {peak}");
    }
}

#[test]
fn the_equalized_headset_follows_the_curve() {
    let flat = graph_with(&built_in("flat"), ALL);
    let boosted = graph_with(&built_in("bass-boost"), ALL);
    let at_40_hz = |model: &GraphModel| {
        let series = find(model, SeriesId::HeadsetWithEq);
        let point = series
            .points
            .iter()
            .find(|point| point.hz >= 40.0)
            .expect("a point at 40 Hz");
        point.db
    };
    assert!(at_40_hz(&boosted) > at_40_hz(&flat) + 3.0);
    assert_eq!(
        find(&flat, SeriesId::Headset),
        find(&boosted, SeriesId::Headset)
    );
}

#[test]
fn the_harman_preset_brings_the_headset_close_to_the_harman_target() {
    let flat = rms_to_target(&built_in("flat"));
    let harman = rms_to_target(&built_in("harman"));
    assert!(harman < flat / 2.0, "harman {harman}, flat {flat}");
    assert!(harman < 2.0, "harman {harman}");
}

#[test]
fn the_editor_shows_the_overlays_the_settings_ask_for() {
    let mut s = snapshot();
    assert_eq!(s.presets.overlays, Overlays::default());
    let flat = built_in("flat");
    assert_eq!(eq_editor_for(&s, &flat).graph, graph_with(&flat, ALL));
    s.presets.overlays = Overlays::NONE;
    assert_eq!(eq_editor_for(&s, &flat).graph, graph(&flat));
    let shown = eq_editor_with(&s, &flat, ALL);
    assert_eq!(shown.graph, graph_with(&flat, ALL));
}

#[test]
fn the_description_summarises_the_overlays() {
    let flat = built_in("flat");
    assert_eq!(
        english(&graph(&flat).description),
        "Nominal frequency response of Flat, with 10 active bands"
    );
    assert_eq!(
        english(&graph_with(&flat, ALL).description),
        "Nominal frequency response of Flat, with 10 active bands. Showing the measured headset \
         response, with and without the equalizer, and the Harman target (B&K 5128)."
    );
}

#[test]
fn every_choice_has_its_own_summary() {
    assert_eq!(summary(Overlays::NONE), None);
    let choices = [
        Overlays::NONE.with_headset(true),
        Overlays::NONE.with_target_setting("harman"),
        Overlays::NONE.with_target_setting("diffuse-field"),
        ALL,
        ALL.with_target_setting("diffuse-field"),
    ];
    let mut summaries: Vec<&str> = choices
        .iter()
        .map(|choice| summary(*choice).expect("a summary"))
        .collect();
    summaries.sort_unstable();
    summaries.dedup();
    assert_eq!(summaries.len(), choices.len());
}
