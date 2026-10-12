use super::biquad::{Biquad, Params, narrow};
use super::curve::Curve;
use super::filter::Hertz;

pub const RESPONSE_RATE: f64 = 48_000.0;
const PREAMP_PROBES: usize = 256;
const PREAMP_STEPS: f64 = 255.0;

#[must_use]
pub fn response(curve: &Curve, frequencies: &[f32]) -> Vec<f32> {
    let biquads = designed(curve);
    frequencies
        .iter()
        .map(|&hz| narrow(total_db(&biquads, f64::from(hz))))
        .collect()
}

#[must_use]
pub fn preamp_db(curve: &Curve) -> f64 {
    let biquads = designed(curve);
    let loudest = probes(curve)
        .map(|hz| total_db(&biquads, hz))
        .fold(0.0, f64::max);
    -loudest
}

fn designed(curve: &Curve) -> Vec<Biquad> {
    curve
        .filters()
        .iter()
        .copied()
        .filter(|filter| filter.shapes_sound())
        .map(|filter| Biquad::design(&Params::from(filter), RESPONSE_RATE))
        .collect()
}

fn probes(curve: &Curve) -> impl Iterator<Item = f64> {
    let centres = curve
        .filters()
        .iter()
        .map(|filter| f64::from(filter.hz().get()));
    audible_grid().chain(centres)
}

fn audible_grid() -> impl Iterator<Item = f64> {
    let (low, high) = (f64::from(Hertz::MIN), f64::from(Hertz::MAX));
    let ratio = (high / low).powf(PREAMP_STEPS.recip());
    std::iter::successors(Some(low), move |hz| Some(hz * ratio)).take(PREAMP_PROBES)
}

fn total_db(biquads: &[Biquad], hz: f64) -> f64 {
    biquads
        .iter()
        .map(|biquad| biquad.magnitude_db(hz, RESPONSE_RATE))
        .sum()
}
