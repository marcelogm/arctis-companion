use arctis_core::eq::biquad::shaping_biquads;
use arctis_core::eq::response::{preamp_db, preamp_db_at};
use arctis_core::eq::{Curve, FilterKind, library, response};

use crate::support::{assert_close, filter, log_grid, peaking};

const SWEEP_POINTS: u32 = 200_000;

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

fn swept_peak_db(curve: &Curve, rate: u32) -> f64 {
    let biquads: Vec<_> = shaping_biquads(curve, rate).collect();
    let nyquist = f64::from(rate) / 2.0;
    let step = (nyquist - 20.0) / f64::from(SWEEP_POINTS);
    (0..=SWEEP_POINTS)
        .map(|index| 20.0 + step * f64::from(index))
        .map(|hz| {
            let rate = f64::from(rate);
            biquads
                .iter()
                .map(|biquad| biquad.magnitude_db(hz, rate))
                .sum::<f64>()
        })
        .fold(0.0, f64::max)
}

fn high_shelf(hz: u16) -> Curve {
    Curve::from_array([filter(FilterKind::HighShelf, hz, 20, 10_000)])
}

#[test]
fn the_preamp_cancels_boosts_up_to_nyquist() {
    let curves = [
        high_shelf(20_000),
        high_shelf(16_000),
        Curve::from_array([
            filter(FilterKind::LowPass, 18_000, 0, 10_000),
            filter(FilterKind::HighShelf, 15_000, 20, 707),
        ]),
    ];
    for curve in &curves {
        for rate in [44_100, 48_000] {
            let case = format!("{curve:?} at {rate} Hz");
            assert_close(
                preamp_db_at(curve, rate),
                -swept_peak_db(curve, rate),
                0.05,
                &case,
            );
        }
    }
}

#[test]
fn the_exported_preamp_is_the_one_at_the_graph_rate() {
    let curve = high_shelf(20_000);
    assert_close(
        preamp_db(&curve),
        preamp_db_at(&curve, 48_000),
        1e-12,
        "graph rate",
    );
}
