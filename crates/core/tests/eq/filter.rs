use arctis_core::eq::filter::Clamped;
use arctis_core::eq::{EnvelopeError, Filter, FilterKind, Gain, Hertz, Q};

use crate::support::{filter, peaking};

#[test]
fn frequency_spans_twenty_hertz_to_twenty_kilohertz() {
    assert_eq!(Hertz::new(20).map(Hertz::get), Ok(20));
    assert_eq!(Hertz::new(20_000).map(Hertz::get), Ok(20_000));
    assert_eq!(Hertz::new(19), Err(EnvelopeError::Frequency));
    assert_eq!(Hertz::new(20_001), Err(EnvelopeError::Frequency));
}

#[test]
fn gain_spans_ten_decibels_either_way_in_half_steps() {
    assert_eq!(Gain::from_half_db(-20).map(Gain::half_db), Ok(-20));
    assert_eq!(Gain::from_half_db(20).map(Gain::half_db), Ok(20));
    assert_eq!(Gain::from_half_db(-21), Err(EnvelopeError::Gain));
    assert_eq!(Gain::from_half_db(21), Err(EnvelopeError::Gain));
}

#[test]
fn gain_converts_half_steps_to_decibels() {
    let gain = Gain::from_half_db(-11).expect("in range");
    assert!((gain.db() + 5.5).abs() < f64::EPSILON);
}

#[test]
fn q_spans_two_tenths_to_ten_in_thousandths() {
    assert_eq!(Q::from_milli(200).map(Q::milli), Ok(200));
    assert_eq!(Q::from_milli(10_000).map(Q::milli), Ok(10_000));
    assert_eq!(Q::from_milli(199), Err(EnvelopeError::Q));
    assert_eq!(Q::from_milli(10_001), Err(EnvelopeError::Q));
    assert!((Q::from_milli(1_410).expect("in range").value() - 1.41).abs() < 1e-12);
}

#[test]
fn pass_filters_refuse_gain() {
    for kind in [FilterKind::LowPass, FilterKind::HighPass] {
        assert_eq!(
            Filter::from_parts(kind, 1_000, 1, 707),
            Err(EnvelopeError::PassGain)
        );
        assert!(Filter::from_parts(kind, 1_000, 0, 707).is_ok());
    }
}

#[test]
fn shaping_filters_accept_gain() {
    for kind in [
        FilterKind::Peaking,
        FilterKind::LowShelf,
        FilterKind::HighShelf,
    ] {
        assert_eq!(filter(kind, 1_000, 20, 707).gain().half_db(), 20);
    }
}

#[test]
fn from_parts_reports_the_first_field_outside_the_envelope() {
    let kind = FilterKind::Peaking;
    assert_eq!(
        Filter::from_parts(kind, 10, 40, 50),
        Err(EnvelopeError::Frequency)
    );
    assert_eq!(
        Filter::from_parts(kind, 100, 40, 50),
        Err(EnvelopeError::Gain)
    );
    assert_eq!(Filter::from_parts(kind, 100, 4, 50), Err(EnvelopeError::Q));
}

#[test]
fn filters_start_enabled_and_can_be_disabled() {
    let enabled = peaking(1_000, 6, 1_000);
    assert!(enabled.is_enabled());
    assert!(!enabled.with_enabled(false).is_enabled());
}

#[test]
fn only_enabled_filters_with_an_effect_shape_sound() {
    assert!(peaking(1_000, 6, 1_000).shapes_sound());
    assert!(!peaking(1_000, 0, 1_000).shapes_sound());
    assert!(!peaking(1_000, 6, 1_000).with_enabled(false).shapes_sound());
    assert!(filter(FilterKind::LowPass, 1_000, 0, 707).shapes_sound());
}

fn kept<T>(value: Result<T, EnvelopeError>) -> Clamped<T> {
    Clamped {
        value: value.expect("inside the envelope"),
        clamped: false,
    }
}

fn pinned<T>(value: Result<T, EnvelopeError>) -> Clamped<T> {
    Clamped {
        clamped: true,
        ..kept(value)
    }
}

#[test]
fn clamped_frequencies_pin_to_the_envelope() {
    assert_eq!(Hertz::clamped(1_000), kept(Hertz::new(1_000)));
    assert_eq!(Hertz::clamped(i64::MIN), pinned(Hertz::new(Hertz::MIN)));
    assert_eq!(Hertz::clamped(i64::MAX), pinned(Hertz::new(Hertz::MAX)));
}

#[test]
fn clamped_gains_pin_to_the_envelope() {
    assert_eq!(Gain::clamped_half_db(-3), kept(Gain::from_half_db(-3)));
    assert_eq!(Gain::clamped_half_db(-21), pinned(Gain::from_half_db(-20)));
    assert_eq!(Gain::clamped_half_db(300), pinned(Gain::from_half_db(20)));
}

#[test]
fn clamped_qs_pin_to_the_envelope() {
    assert_eq!(Q::clamped_milli(707), kept(Q::from_milli(707)));
    assert_eq!(Q::clamped_milli(-1), pinned(Q::from_milli(Q::MIN_MILLI)));
    assert_eq!(
        Q::clamped_milli(70_000),
        pinned(Q::from_milli(Q::MAX_MILLI))
    );
}

#[test]
fn pass_filters_can_drop_their_gain() {
    let hz = Hertz::new(1_000).expect("inside");
    let gain = Gain::from_half_db(6).expect("inside");
    let q = Q::from_milli(707).expect("inside");
    let low_pass = Filter::dropping_pass_gain(FilterKind::LowPass, hz, gain, q);
    assert_eq!(low_pass, filter(FilterKind::LowPass, 1_000, 0, 707));
    let peak = Filter::dropping_pass_gain(FilterKind::Peaking, hz, gain, q);
    assert_eq!(peak, peaking(1_000, 6, 707));
}

#[test]
fn envelope_errors_read_as_sentences() {
    let cases = [
        (EnvelopeError::Frequency, "frequency outside 20..20000 Hz"),
        (EnvelopeError::Gain, "gain outside ±10 dB"),
        (EnvelopeError::Q, "Q outside 0.2..10"),
        (
            EnvelopeError::PassGain,
            "low-pass and high-pass take no gain",
        ),
        (EnvelopeError::TooManyFilters, "more than ten filters"),
    ];
    for (error, message) in cases {
        assert_eq!(error.to_string(), message);
    }
}
