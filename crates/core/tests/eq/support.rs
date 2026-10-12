use std::f32::consts::TAU;

use arctis_core::eq::biquad::Biquad;
use arctis_core::eq::{Curve, Equalizer, Filter, FilterKind, Gain, Hertz, Q};
use proptest::prelude::*;

const FACTORY_BANDS: [u16; 10] = [32, 64, 125, 250, 500, 1_000, 2_000, 4_000, 8_000, 16_000];

pub fn filter(kind: FilterKind, hz: u16, half_db: i8, milli_q: u16) -> Filter {
    Filter::from_parts(kind, hz, half_db, milli_q).expect("filter inside the envelope")
}

pub fn peaking(hz: u16, half_db: i8, milli_q: u16) -> Filter {
    filter(FilterKind::Peaking, hz, half_db, milli_q)
}

pub fn factory_curve(milli_q: u16) -> Curve {
    let filters = FACTORY_BANDS.iter().map(|&hz| peaking(hz, 0, milli_q));
    Curve::new(filters.collect()).expect("ten filters")
}

pub fn assert_close(actual: f64, expected: f64, tolerance: f64, case: &str) {
    let error = (actual - expected).abs();
    assert!(
        error <= tolerance,
        "{case}: got {actual}, expected {expected} ± {tolerance}"
    );
}

pub fn log_grid(points: u16) -> Vec<f32> {
    let ratio = 1_000f32.powf(1.0 / f32::from(points - 1));
    std::iter::successors(Some(20.0_f32), |hz| Some(hz * ratio))
        .take(usize::from(points))
        .collect()
}

pub fn any_filter() -> impl Strategy<Value = Filter> {
    let kind = proptest::sample::select(FilterKind::ALL.to_vec());
    let parts = (
        kind,
        Hertz::MIN..=Hertz::MAX,
        Gain::MIN_HALF_DB..=Gain::MAX_HALF_DB,
        Q::MIN_MILLI..=Q::MAX_MILLI,
    );
    parts.prop_map(|(kind, hz, half_db, milli_q)| {
        let half_db = if kind.has_gain() { half_db } else { 0 };
        filter(kind, hz, half_db, milli_q)
    })
}

pub const BLOCK: usize = 512;
pub const RATE: u32 = 48_000;

pub fn sine(hz: f32, rate: u32, frames: usize) -> Vec<f32> {
    let rate = f32::from(u16::try_from(rate).expect("rate below 65536 Hz"));
    let step = TAU * hz / rate;
    std::iter::successors(Some(0.0_f32), |phase| Some((phase + step) % TAU))
        .take(frames)
        .map(|phase| 0.5 * phase.sin())
        .collect()
}

pub fn stereo(mono: &[f32]) -> Vec<f32> {
    mono.iter().flat_map(|&sample| [sample, sample]).collect()
}

pub fn processed(equalizer: &mut Equalizer, input: &[f32]) -> Vec<f32> {
    let mut output = input.to_vec();
    for block in output.chunks_mut(BLOCK) {
        equalizer.process(block);
    }
    output
}

pub fn bits(samples: &[f32]) -> Vec<u32> {
    samples.iter().map(|sample| sample.to_bits()).collect()
}

#[expect(clippy::cast_possible_truncation)]
fn through_f32(coefficient: f64) -> f64 {
    f64::from(coefficient as f32)
}

pub fn poles_inside_unit_circle(biquad: &Biquad) -> bool {
    let (a1, a2) = (through_f32(biquad.a1), through_f32(biquad.a2));
    a2.abs() < 1.0 && a1.abs() < 1.0 + a2
}
