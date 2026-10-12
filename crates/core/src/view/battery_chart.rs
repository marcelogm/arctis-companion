use super::status_page::StatusPageModel;
use super::text::{Arg, Text};
use crate::battery::sample::UnixSeconds;
use crate::battery::series::{Point, Series, Span, Window, last_full_charge, series};
use crate::companion::snapshot::Snapshot;
use crate::settings::values::Percent;

const FULL: f64 = 100.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartSpan {
    pub from: f64,
    pub to: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BatteryChartModel {
    pub window: Window,
    pub lines: Vec<Vec<ChartPoint>>,
    pub charging: Vec<ChartSpan>,
    pub description: Text,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BatteryContent {
    Empty(Box<StatusPageModel>),
    Chart(BatteryChartModel),
}

#[derive(Clone, Debug, PartialEq)]
pub struct BatteryPageModel {
    pub content: BatteryContent,
    pub last_full_charge: Text,
    pub current_level: Option<Text>,
}

#[must_use]
pub fn battery_page(snapshot: &Snapshot, window: Window, now: UnixSeconds) -> BatteryPageModel {
    let battery = &snapshot.battery;
    let content = if battery.history.is_empty() {
        BatteryContent::Empty(Box::new(no_history()))
    } else {
        BatteryContent::Chart(battery_chart(
            &series(&battery.history, window, now),
            window,
        ))
    };
    BatteryPageModel {
        content,
        last_full_charge: last_full_charge(&battery.history).map_or_else(
            || Text::plain("Never"),
            |at| Text::format("{time}", vec![("time", Arg::Time(at))]),
        ),
        current_level: battery
            .reading
            .filter(|reading| reading.is_connected())
            .map(|reading| {
                Text::format(
                    "{percent}",
                    vec![("percent", Arg::Percent(reading.percent))],
                )
            }),
    }
}

#[must_use]
pub fn battery_chart(series: &Series, window: Window) -> BatteryChartModel {
    let span = series.span;
    BatteryChartModel {
        window,
        lines: series
            .segments
            .iter()
            .map(|segment| line(segment, span))
            .collect(),
        charging: series
            .charging
            .iter()
            .map(|charge| ChartSpan {
                from: x_of(span, charge.start),
                to: x_of(span, charge.end),
            })
            .collect(),
        description: chart_description(series, window),
    }
}

fn line(segment: &[Point], span: Span) -> Vec<ChartPoint> {
    segment
        .iter()
        .map(|point| ChartPoint {
            x: x_of(span, point.at),
            y: f64::from(point.percent.get()) / FULL,
        })
        .collect()
}

fn x_of(span: Span, at: UnixSeconds) -> f64 {
    let length = span.end.since(span.start);
    if length == 0 {
        return 0.0;
    }
    seconds(at.since(span.start)) / seconds(length)
}

fn seconds(value: u64) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

fn chart_description(series: &Series, window: Window) -> Text {
    let levels = series.segments.iter().flatten().map(|point| point.percent);
    let low = levels.clone().min_by_key(|level| level.get());
    let high = levels.max_by_key(|level| level.get());
    match (low.zip(high), window) {
        (None, Window::Day) => Text::plain("No battery readings in the last day"),
        (None, Window::Week) => Text::plain("No battery readings in the last week"),
        (Some(range), Window::Day) => range_text(
            "Battery level over the last day, between {low} and {high}",
            range,
        ),
        (Some(range), Window::Week) => range_text(
            "Battery level over the last week, between {low} and {high}",
            range,
        ),
    }
}

fn range_text(msgid: &str, (low, high): (Percent, Percent)) -> Text {
    Text::format(
        msgid,
        vec![("low", Arg::Percent(low)), ("high", Arg::Percent(high))],
    )
}

fn no_history() -> StatusPageModel {
    StatusPageModel::new(
        "battery-symbolic",
        "No Battery History Yet",
        "History appears after the headset has been connected for a while",
        None,
    )
}
