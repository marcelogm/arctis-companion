use arctis_core::eq::response::preamp_db;
use arctis_core::eq::{Coefficients, Curve, Equalizer, library, response};

use crate::support::{RATE, assert_close, bits, peaking, processed, sine, stereo};

const SECOND: usize = 48_000;
const LAST_FIFTH: usize = 38_400;

fn peak(samples: &[f32]) -> f64 {
    f64::from(
        samples
            .iter()
            .fold(0.0_f32, |loudest, sample| loudest.max(sample.abs())),
    )
}

#[test]
fn flat_is_a_bit_exact_identity() {
    for rate in [44_100, 48_000] {
        let coefficients = Coefficients::design(library::flat().curve(), rate);
        let mut equalizer = Equalizer::new(&coefficients, 2);
        let input = stereo(&sine(997.0, RATE, 4_800));
        assert_eq!(bits(&processed(&mut equalizer, &input)), bits(&input));
    }
}

#[test]
fn each_channel_keeps_its_own_state() {
    let coefficients = Coefficients::design(&Curve::from_array([peaking(1_000, 12, 1_000)]), RATE);
    let left = sine(440.0, RATE, 2_400);
    let mono = processed(&mut Equalizer::new(&coefficients, 1), &left);
    let left_and_silence: Vec<f32> = left.iter().flat_map(|&sample| [sample, 0.0]).collect();
    let both = processed(&mut Equalizer::new(&coefficients, 2), &left_and_silence);
    let left_out: Vec<f32> = both.iter().step_by(2).copied().collect();
    assert_eq!(bits(&left_out), bits(&mono));
    assert!(
        both.iter()
            .skip(1)
            .step_by(2)
            .all(|sample| sample.to_bits() == 0)
    );
}

#[test]
fn processing_follows_the_response_minus_the_preamp() {
    let curve = Curve::from_array([peaking(1_000, 12, 1_000)]);
    for hz in [100.0, 1_000.0] {
        let mut equalizer = Equalizer::new(&Coefficients::design(&curve, RATE), 1);
        let input = sine(hz, RATE, SECOND);
        let output = processed(&mut equalizer, &input);
        let measured = 20.0 * (peak(&output[LAST_FIFTH..]) / peak(&input[LAST_FIFTH..])).log10();
        let expected = f64::from(response(&curve, &[hz])[0]) + preamp_db(&curve);
        assert_close(measured, expected, 0.1, "level");
    }
}

#[test]
fn zero_channels_leave_samples_alone() {
    let coefficients = Coefficients::design(&Curve::from_array([peaking(1_000, 12, 1_000)]), RATE);
    let mut samples = [0.25_f32, -0.5];
    Equalizer::new(&coefficients, 0).process(&mut samples);
    assert_eq!(bits(&samples), bits(&[0.25, -0.5]));
}
