use arctis_core::eq::decimal::{
    parse_thousandths, render_thousandths, rounded_quotient, thousandths_of,
};

#[test]
fn decimal_text_parses_to_thousandths() {
    let cases = [
        ("5.5", Some(5_500)),
        ("-3.2", Some(-3_200)),
        ("+0.70", Some(700)),
        ("10", Some(10_000)),
        ("0.7071", Some(707)),
        ("1.9995", Some(2_000)),
        ("-0", Some(0)),
        ("", None),
        (".5", None),
        ("1.2.3", None),
        ("abc", None),
        ("1e3", None),
        ("99999999999999999999", None),
    ];
    for (text, expected) in cases {
        assert_eq!(parse_thousandths(text), expected, "{text}");
    }
}

#[test]
fn thousandths_render_as_short_decimals() {
    let cases = [
        (5_500, "5.5"),
        (-3_500, "-3.5"),
        (700, "0.7"),
        (707, "0.707"),
        (10_000, "10"),
        (0, "0"),
        (-500, "-0.5"),
    ];
    for (value, expected) in cases {
        assert_eq!(render_thousandths(value), expected, "{value}");
    }
}

#[test]
fn quotients_round_half_away_from_zero() {
    let cases = [
        (2_600, 500, 5),
        (2_750, 500, 6),
        (-2_750, 500, -6),
        (-3_200, 500, -6),
        (1_499, 1_000, 1),
        (1_500, 1_000, 2),
        (0, 500, 0),
        (i64::MAX, 1_000, 9_223_372_036_854_776),
        (i64::MIN, 1_000, -9_223_372_036_854_776),
        (i64::MAX, 500, 18_446_744_073_709_552),
    ];
    for (value, divisor, expected) in cases {
        assert_eq!(
            rounded_quotient(value, divisor),
            expected,
            "{value} / {divisor}"
        );
    }
}

#[test]
fn floats_convert_through_their_shortest_text() {
    assert_eq!(thousandths_of(0.71), Some(710));
    assert_eq!(thousandths_of(5.5), Some(5_500));
    assert_eq!(thousandths_of(-0.5), Some(-500));
    assert_eq!(thousandths_of(f64::NAN), None);
    assert_eq!(thousandths_of(f64::INFINITY), None);
}
