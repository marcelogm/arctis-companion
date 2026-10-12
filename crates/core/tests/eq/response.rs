use arctis_core::eq::response::preamp_db;
use arctis_core::eq::{Curve, FilterKind, library, response};

use crate::support::{assert_close, filter, log_grid, peaking};

fn dense_peak(curve: &Curve) -> f64 {
    let gains = response(curve, &log_grid(2_000));
    f64::from(
        gains
            .iter()
            .fold(0.0_f32, |loudest, &gain| loudest.max(gain)),
    )
}

#[test]
fn flat_has_no_response() {
    let gains = response(library::flat().curve(), &log_grid(64));
    assert!(gains.iter().all(|gain| gain.abs() < 1e-6));
}

#[test]
fn a_peak_shows_its_gain_without_the_preamp() {
    let curve = Curve::from_array([peaking(1_000, 12, 1_000)]);
    assert_close(
        f64::from(response(&curve, &[1_000.0])[0]),
        6.0,
        0.05,
        "peak",
    );
}

#[test]
fn disabled_filters_have_no_response() {
    let curve = Curve::from_array([peaking(1_000, 12, 1_000).with_enabled(false)]);
    assert!(
        response(&curve, &log_grid(16))
            .iter()
            .all(|gain| gain.abs() < 1e-6)
    );
}

#[test]
fn filters_add_up_in_decibels() {
    let low = peaking(100, 6, 1_000);
    let high = filter(FilterKind::HighShelf, 8_000, -8, 707);
    let grid = log_grid(32);
    let both = response(&Curve::from_array([low, high]), &grid);
    let low_only = response(&Curve::from_array([low]), &grid);
    let high_only = response(&Curve::from_array([high]), &grid);
    for ((sum, a), b) in both.iter().zip(&low_only).zip(&high_only) {
        assert_close(f64::from(*sum), f64::from(a + b), 1e-4, "sum");
    }
}

#[test]
fn every_built_in_stays_within_ten_decibels() {
    let grid = log_grid(200);
    for preset in library::library() {
        let gains = response(preset.curve(), &grid);
        assert!(
            gains.iter().all(|gain| gain.abs() <= 10.0),
            "{}",
            preset.name()
        );
    }
}

#[test]
fn the_preamp_is_zero_when_nothing_boosts() {
    let cuts = Curve::from_array([
        peaking(100, -6, 1_000),
        filter(FilterKind::HighShelf, 8_000, -8, 707),
    ]);
    for curve in [library::flat().curve().clone(), cuts, Curve::default()] {
        assert_close(preamp_db(&curve), 0.0, 1e-9, "no boost");
    }
}

#[test]
fn the_preamp_cancels_a_lone_boost() {
    let curve = Curve::from_array([
        peaking(1_000, 12, 1_000),
        peaking(5_000, 18, 1_000).with_enabled(false),
    ]);
    assert_close(preamp_db(&curve), -6.0, 1e-6, "lone peak");
}

#[test]
fn the_preamp_cancels_overlapping_boosts() {
    let curve = Curve::from_array([peaking(1_000, 20, 1_000), peaking(1_100, 20, 1_000)]);
    assert_close(preamp_db(&curve), -dense_peak(&curve), 0.05, "overlap");
    assert!(preamp_db(&curve) < -19.0);
}

#[test]
fn the_preamp_cancels_a_resonant_pass_filter() {
    let curve = Curve::from_array([filter(FilterKind::LowPass, 1_000, 0, 10_000)]);
    assert_close(preamp_db(&curve), -20.0, 0.05, "resonance");
}
