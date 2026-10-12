use arctis_core::eq::FilterKind;
use arctis_core::eq::biquad::{Biquad, Params, Section, SectionState};

use crate::support::assert_close;

const RATES: [f64; 2] = [44_100.0, 48_000.0];
const AT_CORNER: f64 = 0.05;
const AT_EDGE: f64 = 0.1;
const SHELF_CASES: [(f64, f64); 6] = [
    (10.0, 0.707),
    (-10.0, 0.707),
    (3.5, 0.707),
    (10.0, 1.0),
    (-10.0, 1.0),
    (3.5, 1.0),
];

fn magnitude(kind: FilterKind, (hz, gain_db, q): (f64, f64, f64), at: f64, rate: f64) -> f64 {
    Biquad::design(
        &Params {
            kind,
            hz,
            gain_db,
            q,
        },
        rate,
    )
    .magnitude_db(at, rate)
}

#[test]
fn peaking_reaches_its_gain_at_the_centre() {
    let cases = [
        (100.0, 10.0, 1.0),
        (1_000.0, -10.0, 0.7),
        (1_000.0, 3.5, 4.0),
        (10_000.0, 6.0, 1.41),
        (20.0, -10.0, 0.2),
        (20_000.0, 10.0, 10.0),
    ];
    for rate in RATES {
        for (hz, gain_db, q) in cases {
            let at_centre = magnitude(FilterKind::Peaking, (hz, gain_db, q), hz, rate);
            assert_close(at_centre, gain_db, AT_CORNER, "peaking centre");
        }
    }
}

#[test]
fn low_shelf_is_half_way_at_the_corner_and_flat_beyond() {
    for rate in RATES {
        for (gain_db, q) in SHELF_CASES {
            let at = |hz| magnitude(FilterKind::LowShelf, (1_000.0, gain_db, q), hz, rate);
            assert_close(at(1_000.0), gain_db / 2.0, AT_CORNER, "low shelf corner");
            assert_close(at(20.0), gain_db, AT_EDGE, "low shelf at 20 Hz");
            assert_close(at(20_000.0), 0.0, AT_EDGE, "low shelf at 20 kHz");
        }
    }
}

#[test]
fn high_shelf_is_half_way_at_the_corner_and_flat_beyond() {
    for rate in RATES {
        for (gain_db, q) in SHELF_CASES {
            let at = |hz| magnitude(FilterKind::HighShelf, (1_000.0, gain_db, q), hz, rate);
            assert_close(at(1_000.0), gain_db / 2.0, AT_CORNER, "high shelf corner");
            assert_close(at(20.0), 0.0, AT_EDGE, "high shelf at 20 Hz");
            assert_close(at(20_000.0), gain_db, AT_EDGE, "high shelf at 20 kHz");
        }
    }
}

#[test]
fn pass_filters_reach_q_at_the_corner() {
    for rate in RATES {
        assert_pass_corners(FilterKind::LowPass, rate);
        assert_pass_corners(FilterKind::HighPass, rate);
    }
}

fn assert_pass_corners(kind: FilterKind, rate: f64) {
    for (hz, q) in [
        (100.0, 0.5),
        (1_000.0, 0.707),
        (10_000.0, 2.0),
        (1_000.0, 10.0),
    ] {
        let at_corner = magnitude(kind, (hz, 0.0, q), hz, rate);
        assert_close(at_corner, 20.0 * q.log10(), AT_CORNER, "pass corner");
    }
}

#[test]
fn pass_filters_ignore_the_gain_field() {
    for kind in [FilterKind::LowPass, FilterKind::HighPass] {
        let plain = Params {
            kind,
            hz: 1_000.0,
            gain_db: 0.0,
            q: 0.707,
        };
        let boosted = Params {
            gain_db: 6.0,
            ..plain
        };
        assert_eq!(
            Biquad::design(&plain, 48_000.0),
            Biquad::design(&boosted, 48_000.0)
        );
    }
}

#[test]
fn the_identity_section_passes_samples_through() {
    let mut state = SectionState::default();
    for sample in [0.3_f32, -0.7, 0.0, 1.0] {
        assert_eq!(
            Section::IDENTITY.tick(&mut state, sample).to_bits(),
            sample.to_bits()
        );
    }
}

#[test]
fn a_section_runs_the_biquad_difference_equation() {
    let params = Params {
        kind: FilterKind::Peaking,
        hz: 1_000.0,
        gain_db: 6.0,
        q: 1.0,
    };
    let biquad = Biquad::design(&params, 48_000.0);
    let section = Section::from(biquad);
    let mut state = SectionState::default();
    let mut reference = DirectForm::default();
    for input in std::iter::once(1.0_f32).chain(std::iter::repeat_n(0.0, 63)) {
        let actual = f64::from(section.tick(&mut state, input));
        assert_close(
            actual,
            reference.step(&biquad, f64::from(input)),
            1e-5,
            "impulse",
        );
    }
}

#[derive(Default)]
struct DirectForm {
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl DirectForm {
    fn step(&mut self, biquad: &Biquad, x: f64) -> f64 {
        let feedforward = biquad.b0 * x + biquad.b1 * self.x1 + biquad.b2 * self.x2;
        let y = feedforward - biquad.a1 * self.y1 - biquad.a2 * self.y2;
        (self.x2, self.x1, self.y2, self.y1) = (self.x1, x, self.y1, y);
        y
    }
}
