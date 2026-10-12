use arctis_core::eq::biquad::shaping_biquads;
use arctis_core::eq::response::preamp_db_at;
use arctis_core::eq::{Coefficients, Curve, Filter, FilterKind, Gain, Hertz, Q, library};
use proptest::prelude::*;

use crate::support::{any_filter, assert_close, filter, peaking, poles_inside_unit_circle};

const CORNERS: [(u16, u16); 4] = [
    (Hertz::MIN, Q::MIN_MILLI),
    (Hertz::MIN, Q::MAX_MILLI),
    (Hertz::MAX, Q::MIN_MILLI),
    (Hertz::MAX, Q::MAX_MILLI),
];

fn stable(candidate: Filter, rate: u32) -> bool {
    shaping_biquads(&Curve::from_array([candidate]), rate)
        .all(|biquad| poles_inside_unit_circle(&biquad))
}

fn gains_for(kind: FilterKind) -> Vec<i8> {
    if kind.has_gain() {
        vec![Gain::MIN_HALF_DB, Gain::MAX_HALF_DB]
    } else {
        vec![0]
    }
}

fn corner_filters() -> Vec<Filter> {
    let mut corners = Vec::new();
    for kind in FilterKind::ALL {
        for (hz, milli_q) in CORNERS {
            corners.extend(
                gains_for(kind)
                    .into_iter()
                    .map(|half_db| filter(kind, hz, half_db, milli_q)),
            );
        }
    }
    corners
}

#[test]
fn flat_designs_no_sections_and_a_unity_preamp() {
    let coefficients = Coefficients::design(library::flat().curve(), 48_000);
    assert!(coefficients.sections().is_empty());
    assert_eq!(coefficients.preamp().to_bits(), 1.0_f32.to_bits());
}

#[test]
fn only_filters_that_shape_sound_get_a_section() {
    let curve = Curve::from_array([
        peaking(100, 6, 1_000),
        peaking(200, 0, 1_000),
        peaking(300, 6, 1_000).with_enabled(false),
        filter(FilterKind::LowPass, 8_000, 0, 707),
    ]);
    assert_eq!(Coefficients::design(&curve, 48_000).sections().len(), 2);
}

#[test]
fn the_preamp_lowers_by_the_loudest_boost() {
    let curve = Curve::from_array([peaking(100, 12, 1_000)]);
    let preamp = f64::from(Coefficients::design(&curve, 48_000).preamp());
    assert_close(preamp, 10f64.powf(-6.0 / 20.0), 1e-6, "preamp");
}

#[test]
fn the_preamp_follows_overlapping_boosts() {
    let curve = Curve::from_array([peaking(1_000, 20, 1_000), peaking(1_100, 20, 1_000)]);
    let preamp = f64::from(Coefficients::design(&curve, 48_000).preamp());
    assert!(preamp < 10f64.powf(-19.0 / 20.0), "preamp {preamp}");
}

#[test]
fn the_preamp_follows_the_rate() {
    let curve = Curve::from_array([filter(FilterKind::HighShelf, 20_000, 20, 10_000)]);
    for rate in [44_100, 48_000] {
        let preamp = f64::from(Coefficients::design(&curve, rate).preamp());
        let expected = 10f64.powf(preamp_db_at(&curve, rate) / 20.0);
        assert_close(preamp, expected, 1e-6, &format!("{rate} Hz"));
    }
}

#[test]
fn filters_at_or_above_nyquist_get_no_section() {
    let curve = Curve::from_array([
        peaking(1_000, 6, 1_000),
        peaking(8_000, 6, 1_000),
        filter(FilterKind::HighPass, 20_000, 0, 707),
    ]);
    assert_eq!(Coefficients::design(&curve, 16_000).sections().len(), 1);
    assert_eq!(Coefficients::design(&curve, 32_000).sections().len(), 2);
    assert_eq!(Coefficients::design(&curve, 48_000).sections().len(), 3);
}

#[test]
fn envelope_corners_design_stable_sections() {
    for candidate in corner_filters() {
        for rate in [44_100, 48_000] {
            assert!(stable(candidate, rate), "{candidate:?} at {rate} Hz");
        }
    }
}

proptest! {
    #[test]
    fn every_filter_in_the_envelope_designs_stable_sections(
        candidate in any_filter(),
        rate in proptest::sample::select(vec![44_100_u32, 48_000]),
    ) {
        prop_assert!(stable(candidate, rate));
    }
}
