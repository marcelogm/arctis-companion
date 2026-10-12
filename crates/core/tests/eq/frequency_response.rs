use arctis_core::eq::frequency_response::{FrequencyResponse, ParseError, Point};
use arctis_core::eq::{Curve, response};

use crate::support::{assert_close, peaking};

fn parsed(text: &str) -> FrequencyResponse {
    FrequencyResponse::parse(text).expect("valid response")
}

fn response_at(curve: &Curve, hz: f32) -> f32 {
    response(curve, &[hz])[0]
}

fn two_octaves() -> FrequencyResponse {
    parsed("freq_hz,spl_db\n250,0\n1000,6\n")
}

#[test]
fn a_file_parses_into_points_in_order() {
    let response = parsed("freq_hz,spl_db\n20.00,5.32\n20.59,-5.40\n");
    assert_eq!(
        response.points(),
        [
            Point { hz: 20.0, db: 5.32 },
            Point {
                hz: 20.59,
                db: -5.4
            }
        ]
    );
}

#[test]
fn a_trailing_blank_line_and_carriage_returns_are_ignored() {
    let response = parsed("freq_hz,spl_db\r\n100,1\r\n200,2\r\n\n");
    assert_eq!(response.points().len(), 2);
}

#[test]
fn a_file_without_the_header_is_refused() {
    assert_eq!(
        FrequencyResponse::parse("frequency,raw\n20,1\n40,2\n"),
        Err(ParseError::Header)
    );
}

#[test]
fn a_malformed_line_is_named() {
    for line in ["40", "40,loud", "x,1", "-40,1", "0,1", "40,inf"] {
        let text = format!("freq_hz,spl_db\n20,1\n{line}\n");
        assert_eq!(
            FrequencyResponse::parse(&text),
            Err(ParseError::Malformed { line: 3 }),
            "{line}"
        );
    }
}

#[test]
fn frequencies_must_rise() {
    assert_eq!(
        FrequencyResponse::parse("freq_hz,spl_db\n40,1\n40,2\n"),
        Err(ParseError::NotAscending { line: 3 })
    );
}

#[test]
fn a_response_needs_two_points() {
    assert_eq!(
        FrequencyResponse::parse("freq_hz,spl_db\n40,1\n"),
        Err(ParseError::TooFewPoints)
    );
}

#[test]
fn interpolation_hits_every_point() {
    let response = two_octaves();
    assert_close(response.db_at(250.0), 0.0, 1e-12, "first");
    assert_close(response.db_at(1_000.0), 6.0, 1e-12, "last");
}

#[test]
fn interpolation_is_linear_in_log_frequency() {
    let response = two_octaves();
    assert_close(response.db_at(500.0), 3.0, 1e-9, "one octave up");
    assert_close(
        response.db_at(250.0 * 2f64.powf(0.5)),
        1.5,
        1e-9,
        "half an octave up",
    );
}

#[test]
fn interpolation_holds_the_ends_outside_the_data() {
    let response = two_octaves();
    assert_close(response.db_at(20.0), 0.0, 1e-12, "below");
    assert_close(response.db_at(20_000.0), 6.0, 1e-12, "above");
}

#[test]
fn the_level_is_the_mean_over_300_hz_to_3_khz() {
    let flat = parsed("freq_hz,spl_db\n20,-30\n299,-30\n300,4\n3000,4\n3001,30\n20000,30\n");
    assert_close(flat.level_db(), 4.0, 1e-9, "steps outside the band");
    let ramp = parsed("freq_hz,spl_db\n300,0\n3000,10\n");
    assert_close(ramp.level_db(), 5.0, 1e-9, "log-linear ramp");
}

#[test]
fn normalising_sets_the_level_to_zero_and_keeps_the_shape() {
    let response = parsed("freq_hz,spl_db\n20,9\n1000,12\n20000,3\n");
    let normalised = response.normalised();
    assert_close(normalised.level_db(), 0.0, 1e-9, "level");
    let offset = response.level_db();
    for (raw, aligned) in response.points().iter().zip(normalised.points()) {
        assert_close(aligned.hz, raw.hz, 0.0, "frequency");
        assert_close(aligned.db, raw.db - offset, 1e-9, "shape");
    }
}

#[test]
fn equalizing_with_no_filter_changes_nothing() {
    let response = two_octaves();
    assert_eq!(response.equalized(&Curve::default()), response);
}

#[test]
fn equalizing_adds_the_curve_response_at_each_point() {
    let response = two_octaves();
    let curve = Curve::from_array([peaking(1_000, 12, 1_000)]);
    let equalized = response.equalized(&curve);
    assert_close(equalized.db_at(1_000.0), 12.0, 0.01, "peak centre");
    let skirt = f64::from(response_at(&curve, 250.0));
    assert_close(equalized.db_at(250.0), skirt, 1e-6, "two octaves away");
}
