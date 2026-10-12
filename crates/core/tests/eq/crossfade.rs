use arctis_core::eq::{Coefficients, Curve, Equalizer, FilterKind, PresetId, library};

use crate::support::{RATE, bits, filter, peaking, processed, sine};

const SETTLE: usize = 24_000;
const WINDOW: usize = 4_800;
const FADE_FRAMES: usize = (RATE / 100) as usize;
const F32_NOISE: f32 = 1e-6;

fn built_in(id: &str) -> Curve {
    let preset = library::find(&PresetId::new(id)).expect("built-in");
    preset.curve().clone()
}

fn flat() -> Curve {
    built_in(library::FLAT_ID)
}

fn fade_frames() -> f32 {
    f32::from(u16::try_from(FADE_FRAMES).expect("short fade"))
}

fn bass() -> Curve {
    Curve::from_array([filter(FilterKind::LowShelf, 100, 20, 707)])
}

fn notch() -> Curve {
    Curve::from_array([peaking(1_000, -20, 1_000)])
}

fn air() -> Curve {
    Curve::from_array([filter(FilterKind::HighShelf, 2_000, 20, 707)])
}

fn equalizer_for(curve: &Curve) -> Equalizer {
    Equalizer::new(&Coefficients::design(curve, RATE), 1)
}

fn largest_step(samples: &[f32]) -> f32 {
    samples
        .windows(2)
        .fold(0.0, |largest, pair| largest.max((pair[1] - pair[0]).abs()))
}

fn largest_difference(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .fold(0.0, |largest, (x, y)| largest.max((x - y).abs()))
}

fn switched(from: &Curve, retargets: &[Curve], input: &[f32]) -> Vec<f32> {
    let mut equalizer = equalizer_for(from);
    let (head, tail) = input.split_at(SETTLE);
    let mut output = processed(&mut equalizer, head);
    for curve in retargets {
        equalizer.retarget(&Coefficients::design(curve, RATE));
    }
    output.extend(processed(&mut equalizer, tail));
    output
}

fn assert_smooth_retarget(from: &Curve, to: &Curve) {
    let input = sine(1_000.0, RATE, 2 * SETTLE);
    let before = processed(&mut equalizer_for(from), &input);
    let after = processed(&mut equalizer_for(to), &input);
    let output = switched(from, std::slice::from_ref(to), &input);
    let window = SETTLE - WINDOW..SETTLE;
    let steady_step =
        largest_step(&before[window.clone()]).max(largest_step(&after[2 * SETTLE - WINDOW..]));
    let steady_difference = largest_difference(&before[window.clone()], &after[window]);
    let fade_step = largest_step(&output[SETTLE - 1..=SETTLE + FADE_FRAMES]);
    let bound = steady_step + steady_difference / fade_frames() + F32_NOISE;
    assert!(
        fade_step <= bound,
        "{from:?} -> {to:?}: step {fade_step} above {bound}"
    );
}

#[test]
fn a_retarget_crossfades_without_a_step() {
    let pairs = [
        (flat(), bass()),
        (bass(), flat()),
        (flat(), notch()),
        (notch(), air()),
        (air(), bass()),
        (built_in("v-shape"), built_in("vocal-clarity")),
        (built_in("vocal-clarity"), built_in("v-shape")),
        (built_in("v-shape"), flat()),
        (flat(), built_in("v-shape")),
    ];
    for (from, to) in pairs {
        assert_smooth_retarget(&from, &to);
    }
}

#[test]
fn retargeting_to_the_same_curve_changes_nothing() {
    let input = sine(1_000.0, RATE, 2 * SETTLE);
    let reference = processed(&mut equalizer_for(&bass()), &input);
    assert_eq!(
        bits(&switched(&bass(), &[bass()], &input)),
        bits(&reference)
    );
}

#[test]
fn retargets_during_a_fade_end_on_the_latest_curve() {
    let input = sine(1_000.0, RATE, 2 * SETTLE);
    let reference = processed(&mut equalizer_for(&air()), &input);
    let output = switched(&flat(), &[notch(), bass(), air()], &input);
    let settled = 2 * SETTLE - WINDOW;
    assert!(largest_difference(&output[settled..], &reference[settled..]) < 1e-4);
}

fn quieter() -> Curve {
    Curve::from_array([peaking(20_000, 12, 10_000)])
}

fn channel(samples: &[f32], channels: usize) -> Vec<f32> {
    samples.iter().step_by(channels).copied().collect()
}

fn faded(rate: u32, frames: usize, channels: usize) -> Vec<f32> {
    let target = Coefficients::design(&quieter(), rate);
    let mut equalizer = Equalizer::new(&Coefficients::design(&flat(), rate), channels);
    equalizer.retarget(&target);
    let output = processed(&mut equalizer, &vec![0.5; 2 * frames * channels]);
    assert!(
        output.chunks(channels).all(|frame| frame
            .iter()
            .all(|&sample| sample.to_bits() == frame[0].to_bits())),
        "{channels} channels move together"
    );
    channel(&output, channels)
}

fn assert_fade_lasts(rate: u32, frames: usize) {
    let settled = 0.5 * Coefficients::design(&quieter(), rate).preamp();
    let midway = f32::midpoint(0.5, settled);
    for channels in [1, 2] {
        let output = faded(rate, frames, channels);
        let case = format!("{rate} Hz, {channels} channels");
        assert!(
            (output[frames / 2 - 1] - midway).abs() < 1e-3,
            "{case} half-way"
        );
        assert!(
            (output[frames - 2] - settled).abs() > 1e-4,
            "{case} still fading"
        );
        let tail = &output[frames - 1..];
        assert!(
            tail.iter().all(|sample| (sample - settled).abs() < 1e-4),
            "{case} settled"
        );
    }
}

#[test]
fn a_crossfade_lasts_ten_milliseconds() {
    assert_fade_lasts(48_000, 480);
    assert_fade_lasts(44_100, 441);
}

fn rumbling() -> Curve {
    Curve::from_array([
        filter(FilterKind::LowShelf, 60, 20, 707),
        peaking(60, 20, 2_000),
        peaking(100, 20, 1_000),
    ])
}

fn ringing() -> Curve {
    Curve::from_array([
        peaking(1_000, 12, 1_000),
        peaking(30, 20, 10_000),
        peaking(60, 20, 5_000),
    ])
}

#[test]
fn sections_unused_by_a_shorter_curve_restart_silent() {
    let mut equalizer = equalizer_for(&rumbling());
    processed(&mut equalizer, &sine(60.0, RATE, SETTLE));
    equalizer.retarget(&Coefficients::design(&notch(), RATE));
    processed(&mut equalizer, &vec![0.0; 2 * SETTLE]);
    equalizer.retarget(&Coefficients::design(&ringing(), RATE));
    let output = processed(&mut equalizer, &vec![0.0; SETTLE]);
    assert!(output.iter().all(|sample| sample.to_bits() == 0));
}
