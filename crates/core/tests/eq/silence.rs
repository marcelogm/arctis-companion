use arctis_core::eq::{Coefficients, Curve, Equalizer, FilterKind, library};

use crate::support::{filter, peaking, processed, sine, stereo};

const RATE: u32 = 48_000;
const TEN_SECONDS: usize = 480_000;

fn harsh_curve() -> Curve {
    Curve::from_array([
        filter(FilterKind::LowPass, 20, 0, 10_000),
        filter(FilterKind::HighPass, 20_000, 0, 200),
        peaking(1_000, 20, 5_000),
        filter(FilterKind::LowShelf, 20, -20, 200),
        filter(FilterKind::HighShelf, 20_000, 20, 10_000),
    ])
}

fn clean(samples: &[f32]) -> bool {
    samples
        .iter()
        .all(|sample| sample.is_finite() && !sample.is_subnormal())
}

fn loud_then_silent(equalizer: &mut Equalizer) -> Vec<f32> {
    processed(equalizer, &stereo(&sine(1_000.0, 48_000.0, 4_800)));
    processed(equalizer, &vec![0.0; 2 * TEN_SECONDS])
}

#[test]
fn ten_seconds_of_silence_leave_no_nan_or_subnormal() {
    let mut equalizer = Equalizer::new(&Coefficients::design(&harsh_curve(), RATE), 2);
    assert!(clean(&loud_then_silent(&mut equalizer)));
}

#[test]
fn a_fade_into_silence_leaves_no_nan_or_subnormal() {
    let mut equalizer = Equalizer::new(&Coefficients::design(&harsh_curve(), RATE), 2);
    processed(&mut equalizer, &stereo(&sine(1_000.0, 48_000.0, 4_800)));
    let v_shape = library::library()
        .into_iter()
        .find(|preset| preset.id().as_str() == "v-shape");
    let target = v_shape.expect("built-in").curve().clone();
    equalizer.retarget(&Coefficients::design(&target, RATE));
    assert!(clean(&processed(
        &mut equalizer,
        &vec![0.0; 2 * TEN_SECONDS]
    )));
}
