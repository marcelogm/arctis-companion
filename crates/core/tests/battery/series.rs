use arctis_core::battery::sample::UnixSeconds;
use arctis_core::battery::series::{DAY_SECONDS, Point, Span, Window, series};
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::settings::values::Percent;

use crate::support::{discharging, reading, sample};

const NOW: u64 = 1_000_000;

fn point(at: u64, percent: u8) -> Point {
    Point {
        at: UnixSeconds(at),
        percent: Percent::try_from(percent).expect("percent"),
    }
}

fn span(start: u64, end: u64) -> Span {
    Span {
        start: UnixSeconds(start),
        end: UnixSeconds(end),
    }
}

#[test]
fn an_empty_history_has_no_line() {
    let empty = series(&[], Window::Day, UnixSeconds(NOW));
    assert_eq!(empty.span, span(NOW - DAY_SECONDS, NOW));
    assert!(empty.segments.is_empty());
    assert!(empty.charging.is_empty());
    assert_eq!(empty.last_full_charge, None);
}

#[test]
fn readings_hold_until_the_next_sample() {
    let samples = [
        sample(NOW - 100, discharging(80)),
        sample(NOW - 50, discharging(79)),
    ];
    let drawn = series(&samples, Window::Day, UnixSeconds(NOW));
    assert_eq!(
        drawn.segments,
        vec![vec![
            point(NOW - 100, 80),
            point(NOW - 50, 80),
            point(NOW - 50, 79),
            point(NOW, 79),
        ]]
    );
}

#[test]
fn a_disconnected_reading_breaks_the_line() {
    let samples = [
        sample(NOW - 300, discharging(80)),
        sample(NOW - 200, reading(80, Charging::Unknown, Link::HeadsetOff)),
        sample(NOW - 100, discharging(78)),
    ];
    let drawn = series(&samples, Window::Day, UnixSeconds(NOW));
    assert_eq!(
        drawn.segments,
        vec![
            vec![point(NOW - 300, 80), point(NOW - 200, 80)],
            vec![point(NOW - 100, 78), point(NOW, 78)],
        ]
    );
}

#[test]
fn the_sample_before_the_window_is_carried_to_its_start() {
    let start = NOW - DAY_SECONDS;
    let samples = [
        sample(start - 900, discharging(95)),
        sample(start - 500, discharging(90)),
        sample(NOW - 10, discharging(60)),
    ];
    let drawn = series(&samples, Window::Day, UnixSeconds(NOW));
    assert_eq!(drawn.segments[0][0], point(start, 90));
    assert_eq!(drawn.segments[0].len(), 4);
}

#[test]
fn charging_holds_merge_into_one_span() {
    let samples = [
        sample(NOW - 300, reading(50, Charging::Charging, Link::Connected)),
        sample(NOW - 200, reading(60, Charging::PluggedIn, Link::Connected)),
        sample(NOW - 100, discharging(60)),
    ];
    let drawn = series(&samples, Window::Day, UnixSeconds(NOW));
    assert_eq!(drawn.charging, vec![span(NOW - 300, NOW - 100)]);
}

#[test]
fn the_week_window_spans_seven_days() {
    let drawn = series(&[], Window::Week, UnixSeconds(NOW));
    assert_eq!(drawn.span, span(NOW - 7 * DAY_SECONDS, NOW));
}

#[test]
fn the_last_full_charge_is_the_latest_reading_at_full() {
    let samples = [
        sample(
            NOW - 900_000,
            reading(100, Charging::PluggedIn, Link::Connected),
        ),
        sample(NOW - 300, reading(100, Charging::Charging, Link::Connected)),
        sample(NOW - 100, discharging(99)),
    ];
    let drawn = series(&samples, Window::Day, UnixSeconds(NOW));
    assert_eq!(drawn.last_full_charge, Some(UnixSeconds(NOW - 300)));
}
