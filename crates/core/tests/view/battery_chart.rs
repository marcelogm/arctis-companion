use arctis_core::battery::sample::{Reading, Sample, UnixSeconds};
use arctis_core::battery::series::{DAY_SECONDS, Window};
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::testkit::english::english;
use arctis_core::view::battery_chart::{
    BatteryChartModel, BatteryContent, ChartPoint, battery_page,
};
use arctis_core::view::text::{Arg, Text};

use crate::support::{assert_close, percent, snapshot};

const NOW: u64 = 1_000_000;

fn sample(ago: u64, value: u8, charging: Charging, link: Link) -> Sample {
    Sample {
        at: UnixSeconds(NOW - ago),
        reading: Reading {
            percent: percent(value),
            charging,
            link,
        },
    }
}

fn discharging(ago: u64, value: u8) -> Sample {
    sample(ago, value, Charging::Discharging, Link::Connected)
}

fn chart(history: Vec<Sample>, window: Window) -> BatteryChartModel {
    let mut s = snapshot();
    s.battery.history = history;
    match battery_page(&s, window, UnixSeconds(NOW)).content {
        BatteryContent::Chart(chart) => chart,
        BatteryContent::Empty(_) => panic!("a chart"),
    }
}

fn assert_point(point: ChartPoint, x: f64, y: f64) {
    assert_close(point.x, x, 1e-9);
    assert_close(point.y, y, 1e-9);
}

#[test]
fn no_history_shows_the_empty_state() {
    let page = battery_page(&snapshot(), Window::Day, UnixSeconds(NOW));
    let BatteryContent::Empty(empty) = page.content else {
        panic!("empty state");
    };
    assert_eq!(english(&empty.title), "No Battery History Yet");
    assert_eq!(
        english(&empty.description),
        "History appears after the headset has been connected for a while"
    );
}

#[test]
fn the_chart_is_normalized_to_the_window() {
    let drawn = chart(vec![discharging(DAY_SECONDS / 2, 80)], Window::Day);
    assert_eq!(drawn.lines.len(), 1);
    assert_point(drawn.lines[0][0], 0.5, 0.8);
    assert_point(drawn.lines[0][1], 1.0, 0.8);
}

#[test]
fn charging_spans_are_shaded() {
    let history = vec![
        sample(DAY_SECONDS / 2, 50, Charging::Charging, Link::Connected),
        discharging(DAY_SECONDS / 4, 60),
    ];
    let drawn = chart(history, Window::Day);
    assert_eq!(drawn.charging.len(), 1);
    assert_close(drawn.charging[0].from, 0.5, 1e-9);
    assert_close(drawn.charging[0].to, 0.75, 1e-9);
}

#[test]
fn gaps_break_the_line() {
    let history = vec![
        discharging(300, 80),
        sample(200, 80, Charging::Unknown, Link::HeadsetOff),
        discharging(100, 78),
    ];
    assert_eq!(chart(history, Window::Day).lines.len(), 2);
}

#[test]
fn the_description_states_the_range_for_each_window() {
    let history = vec![discharging(500, 80), discharging(100, 60)];
    let cases = [
        (
            Window::Day,
            "Battery level over the last day, between 60 % and 80 %",
        ),
        (
            Window::Week,
            "Battery level over the last week, between 60 % and 80 %",
        ),
    ];
    for (window, expected) in cases {
        assert_eq!(
            english(&chart(history.clone(), window).description),
            expected
        );
    }
}

#[test]
fn a_window_without_connected_readings_says_so() {
    let off = vec![sample(100, 0, Charging::Unknown, Link::HeadsetOff)];
    let cases = [
        (Window::Day, "No battery readings in the last day"),
        (Window::Week, "No battery readings in the last week"),
    ];
    for (window, expected) in cases {
        assert_eq!(english(&chart(off.clone(), window).description), expected);
    }
}

#[test]
fn the_last_full_charge_and_the_current_level() {
    let mut s = snapshot();
    s.battery.history = vec![
        sample(500, 100, Charging::PluggedIn, Link::Connected),
        discharging(100, 99),
    ];
    let page = battery_page(&s, Window::Day, UnixSeconds(NOW));
    let expected = Text::format("{time}", vec![("time", Arg::Time(UnixSeconds(NOW - 500)))]);
    assert_eq!(page.last_full_charge, expected);
    assert_eq!(
        page.current_level.map(|text| english(&text)).as_deref(),
        Some("73 %")
    );
    s.battery.history = vec![discharging(100, 99)];
    s.battery.reading = None;
    let never = battery_page(&s, Window::Day, UnixSeconds(NOW));
    assert_eq!(english(&never.last_full_charge), "Never");
    assert_eq!(never.current_level, None);
}

#[test]
fn a_synthetic_week_renders_inside_the_chart() {
    let history: Vec<Sample> = (0..7 * 24_u32)
        .rev()
        .map(|hour| {
            let charging = if hour % 24 < 3 {
                Charging::Charging
            } else {
                Charging::Discharging
            };
            let value = u8::try_from(hour % 100).expect("percent");
            sample(
                u64::from(hour) * 3_600 + 60,
                value,
                charging,
                Link::Connected,
            )
        })
        .collect();
    let drawn = chart(history, Window::Week);
    assert!(!drawn.lines.is_empty() && !drawn.charging.is_empty());
    for point in drawn.lines.iter().flatten() {
        assert!(
            (0.0..=1.0).contains(&point.x) && (0.0..=1.0).contains(&point.y),
            "{point:?}"
        );
    }
}
