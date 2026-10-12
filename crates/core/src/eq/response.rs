use super::biquad::{Biquad, narrow, shaping_biquads};
use super::curve::Curve;
use super::filter::Hertz;

pub const RESPONSE_RATE: u32 = 48_000;
const PREAMP_PROBES: u16 = 512;
const REFINEMENTS: usize = 48;
const GOLDEN_RATIO: f64 = 0.618_033_988_749_894_9;

#[must_use]
pub fn response(curve: &Curve, frequencies: &[f32]) -> Vec<f32> {
    let biquads: Vec<Biquad> = shaping_biquads(curve, RESPONSE_RATE).collect();
    frequencies
        .iter()
        .map(|&hz| narrow(total_db(&biquads, f64::from(hz), RESPONSE_RATE)))
        .collect()
}

#[must_use]
pub fn preamp_db(curve: &Curve) -> f64 {
    preamp_db_at(curve, RESPONSE_RATE)
}

#[must_use]
pub fn preamp_db_at(curve: &Curve, rate: u32) -> f64 {
    let biquads: Vec<Biquad> = shaping_biquads(curve, rate).collect();
    let gain_at = |hz: f64| total_db(&biquads, hz, rate);
    let nyquist = f64::from(rate) / 2.0;
    let grid: Vec<(f64, f64)> = log_grid(f64::from(Hertz::MIN), nyquist)
        .map(|hz| (hz, gain_at(hz)))
        .collect();
    let refined = grid
        .windows(3)
        .filter(|window| is_peak(window))
        .map(|window| refined_peak(window[0].0, window[2].0, gain_at));
    let sampled = grid.iter().map(|&(_, db)| db);
    let centres = centres_below(curve, nyquist).map(gain_at);
    -sampled.chain(centres).chain(refined).fold(0.0, f64::max)
}

fn is_peak(window: &[(f64, f64)]) -> bool {
    matches!(window, [(_, left), (_, middle), (_, right)] if middle > left && middle >= right)
}

fn refined_peak(mut low: f64, mut high: f64, gain_at: impl Fn(f64) -> f64) -> f64 {
    for _ in 0..REFINEMENTS {
        let span = (high - low) * GOLDEN_RATIO;
        let (left, right) = (high - span, low + span);
        if gain_at(left) < gain_at(right) {
            low = left;
        } else {
            high = right;
        }
    }
    gain_at(f64::midpoint(low, high))
}

fn centres_below(curve: &Curve, nyquist: f64) -> impl Iterator<Item = f64> {
    curve
        .filters()
        .iter()
        .map(|filter| f64::from(filter.hz().get()))
        .filter(move |&hz| hz < nyquist)
}

fn log_grid(low: f64, high: f64) -> impl Iterator<Item = f64> {
    let ratio = (high / low).powf(f64::from(PREAMP_PROBES - 1).recip());
    std::iter::successors(Some(low), move |hz| Some(hz * ratio)).take(usize::from(PREAMP_PROBES))
}

fn total_db(biquads: &[Biquad], hz: f64, rate: u32) -> f64 {
    let rate = f64::from(rate);
    biquads
        .iter()
        .map(|biquad| biquad.magnitude_db(hz, rate))
        .sum()
}
