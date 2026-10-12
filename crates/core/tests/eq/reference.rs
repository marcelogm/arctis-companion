use arctis_core::eq::frequency_response::FrequencyResponse;
use arctis_core::eq::reference::{
    self, NO_TARGET, Overlays, SeriesId, Target, headset_response, headset_response_with,
    measurement,
};
use arctis_core::eq::{Curve, library};

use crate::support::{assert_close, peaking};

const GRID_POINTS: usize = 241;

fn frequencies(response: &FrequencyResponse) -> Vec<f64> {
    response.points().iter().map(|point| point.hz).collect()
}

fn on_the_shared_grid(response: &FrequencyResponse) {
    let hz = frequencies(response);
    assert_eq!(hz.len(), GRID_POINTS);
    assert_close(hz[0], 20.0, 1e-9, "first point");
    assert_close(hz[GRID_POINTS - 1], 20_000.0, 1e-9, "last point");
    assert_eq!(hz, frequencies(&measurement()));
}

#[test]
fn the_embedded_curves_share_one_twenty_fourth_octave_grid() {
    on_the_shared_grid(&measurement());
    for target in Target::ALL {
        on_the_shared_grid(&target.response());
    }
    let hz = frequencies(&measurement());
    for pair in hz[..GRID_POINTS - 1].windows(2) {
        assert_close(pair[1] / pair[0], 2f64.powf(1.0 / 24.0), 1e-3, "step");
    }
}

#[test]
fn the_measurement_is_rtings_data_as_published() {
    let raw = measurement();
    assert_close(raw.db_at(20.0), 5.32, 1e-9, "20 Hz");
    assert_close(raw.db_at(20_000.0), -21.06, 1e-9, "20 kHz");
}

#[test]
fn every_plotted_curve_is_level_aligned() {
    let curve = Curve::from_array([peaking(1_000, 12, 1_000)]);
    for series in reference::plot(&curve) {
        assert_close(series.response.level_db(), 0.0, 1e-9, series.id.id());
    }
}

#[test]
fn the_credits_name_the_measurement_and_the_project() {
    let credits = reference::CREDITS.join("\n");
    for source in ["RTINGS.com", "https://github.com/jaakkopasanen/AutoEq"] {
        assert!(credits.contains(source), "{source}");
    }
    assert_eq!(reference::CREDITS_TITLE, "Frequency response data");
    assert!(reference::AUTOEQ_COPYRIGHT.contains("Jaakko Pasanen"));
}

#[test]
fn targets_have_stable_ids_and_labels() {
    let ids: Vec<(&str, &str)> = Target::ALL
        .iter()
        .map(|target| (target.id(), target.label()))
        .collect();
    assert_eq!(
        ids,
        [
            ("harman-5128", "Harman (B&K 5128)"),
            ("diffuse-field-5128", "Diffuse field (B&K 5128)"),
        ]
    );
}

#[test]
fn the_plot_lists_headset_then_equalized_then_targets() {
    let series = reference::plot(&Curve::default());
    let ids: Vec<SeriesId> = series.iter().map(|series| series.id).collect();
    assert_eq!(
        ids,
        [
            SeriesId::Headset,
            SeriesId::HeadsetWithEq,
            SeriesId::Target(Target::Harman),
            SeriesId::Target(Target::DiffuseField),
        ]
    );
    let keys: Vec<(&str, &str)> = ids.iter().map(|id| (id.id(), id.label())).collect();
    assert_eq!(
        keys,
        [
            ("headset", "Headset"),
            ("headset-with-eq", "Headset with EQ"),
            ("harman-5128", "Harman (B&K 5128)"),
            ("diffuse-field-5128", "Diffuse field (B&K 5128)"),
        ]
    );
}

#[test]
fn without_filters_the_equalized_headset_is_the_headset() {
    assert_eq!(headset_response_with(&Curve::default()), headset_response());
    assert_eq!(
        headset_response_with(library::flat().curve()),
        headset_response()
    );
}

#[test]
fn a_boost_shows_on_the_equalized_headset_relative_to_the_rest() {
    let curve = Curve::from_array([peaking(100, 12, 1_000)]);
    let before = headset_response();
    let after = headset_response_with(&curve);
    let lift = |response: &FrequencyResponse| response.db_at(100.0) - response.db_at(1_000.0);
    assert_close(lift(&after) - lift(&before), 6.0, 0.1, "boost at 100 Hz");
}

#[test]
fn the_graph_shows_the_headset_and_harman_until_told_otherwise() {
    let shown = Overlays::default();
    assert!(shown.headset && shown.any());
    assert_eq!(shown.target, Some(Target::Harman));
    assert!(!Overlays::NONE.any());
    assert!(Overlays::NONE.with_headset(true).any());
    assert!(Overlays::NONE.with_target_setting("harman").any());
}

#[test]
fn targets_map_to_and_from_their_setting() {
    for target in Target::ALL {
        assert_eq!(Target::from_setting(target.setting()), Some(target));
    }
    assert_eq!(Target::Harman.setting(), "harman");
    assert_eq!(Target::DiffuseField.setting(), "diffuse-field");
    assert_eq!(Target::from_setting(NO_TARGET), None);
    assert_eq!(Target::from_setting("harman-5128"), None);
}

#[test]
fn overlays_change_one_choice_at_a_time() {
    let shown = Overlays::default();
    assert_eq!(shown.target_setting(), "harman");
    let hidden = shown.with_headset(false);
    assert_eq!(hidden.target, Some(Target::Harman));
    assert!(!hidden.headset);
    let diffuse = shown.with_target_setting("diffuse-field");
    assert_eq!(diffuse.target, Some(Target::DiffuseField));
    assert!(diffuse.headset);
    let none = shown.with_target_setting(NO_TARGET);
    assert_eq!(none.target, None);
    assert_eq!(none.target_setting(), NO_TARGET);
}
