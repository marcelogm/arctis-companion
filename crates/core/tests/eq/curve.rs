use arctis_core::eq::curve::{self, MAX_FILTERS};
use arctis_core::eq::{Curve, EnvelopeError, Filter, FilterKind};

use crate::support::{filter, peaking};

fn single(filter: Filter) -> Curve {
    Curve::from_array([filter])
}

#[test]
fn a_curve_holds_at_most_ten_filters() {
    assert!(Curve::new(vec![peaking(1_000, 0, 1_000); MAX_FILTERS]).is_ok());
    let eleven = vec![peaking(1_000, 0, 1_000); MAX_FILTERS + 1];
    assert_eq!(Curve::new(eleven), Err(EnvelopeError::TooManyFilters));
}

#[test]
fn equivalence_follows_the_device_tolerance_table() {
    let base = peaking(1_000, 6, 1_000);
    let cases = [
        ("identical", peaking(1_000, 6, 1_000), true),
        ("one hertz above", peaking(1_001, 6, 1_000), true),
        ("one hertz below", peaking(999, 6, 1_000), true),
        ("two hertz above", peaking(1_002, 6, 1_000), false),
        ("q one percent above", peaking(1_000, 6, 1_010), true),
        ("q one percent below", peaking(1_000, 6, 990), true),
        ("q over one percent above", peaking(1_000, 6, 1_011), false),
        ("q over one percent below", peaking(1_000, 6, 989), false),
        ("half a decibel louder", peaking(1_000, 7, 1_000), false),
        (
            "another kind",
            filter(FilterKind::LowShelf, 1_000, 6, 1_000),
            false,
        ),
        ("disabled", base.with_enabled(false), false),
    ];
    for (case, other, expected) in cases {
        assert_eq!(
            curve::equivalent(&single(base), &single(other)),
            expected,
            "{case}"
        );
        assert_eq!(
            curve::equivalent(&single(other), &single(base)),
            expected,
            "{case} reversed"
        );
    }
}

#[test]
fn disabled_slots_are_equal_whatever_they_hold() {
    let quiet = peaking(1_000, 6, 1_000).with_enabled(false);
    let loud = filter(FilterKind::HighShelf, 8_000, -20, 5_000).with_enabled(false);
    assert!(curve::equivalent(&single(quiet), &single(loud)));
}

#[test]
fn missing_slots_count_as_disabled() {
    let short = single(peaking(1_000, 6, 1_000));
    let padded = Curve::from_array([
        peaking(1_000, 6, 1_000),
        peaking(50, 4, 700).with_enabled(false),
    ]);
    assert!(curve::equivalent(&short, &padded));
    let lone_disabled = single(peaking(50, 4, 700).with_enabled(false));
    assert!(curve::equivalent(&Curve::default(), &lone_disabled));
}

#[test]
fn enabled_filters_compare_in_order() {
    let forward = Curve::from_array([peaking(100, 2, 1_000), peaking(1_000, 4, 1_000)]);
    let swapped = Curve::from_array([peaking(1_000, 4, 1_000), peaking(100, 2, 1_000)]);
    assert!(!curve::equivalent(&forward, &swapped));
}

#[test]
fn a_disabled_filter_between_enabled_ones_is_skipped() {
    let written = Curve::from_array([
        peaking(100, 2, 1_000),
        peaking(500, 4, 700).with_enabled(false),
        peaking(1_000, 4, 1_000),
    ]);
    let read_back = Curve::from_array([peaking(100, 2, 1_000), peaking(1_000, 4, 1_000)]);
    assert!(curve::equivalent(&written, &read_back));
    assert!(curve::equivalent(&read_back, &written));
}

#[test]
fn a_stored_q_of_1410_matches_a_written_1414() {
    let written = single(peaking(32, 0, 1_414));
    let stored = single(peaking(32, 0, 1_410));
    assert!(curve::equivalent(&written, &stored));
}
