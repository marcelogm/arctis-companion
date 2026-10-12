use std::iter;

use super::sample::{Sample, UnixSeconds};
use crate::settings::values::Percent;

pub const DAY_SECONDS: u64 = 24 * 60 * 60;
const FULL: u8 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Window {
    Day,
    Week,
}

impl Window {
    #[must_use]
    pub const fn seconds(self) -> u64 {
        match self {
            Self::Day => DAY_SECONDS,
            Self::Week => 7 * DAY_SECONDS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    pub at: UnixSeconds,
    pub percent: Percent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: UnixSeconds,
    pub end: UnixSeconds,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Series {
    pub span: Span,
    pub segments: Vec<Vec<Point>>,
    pub charging: Vec<Span>,
    pub last_full_charge: Option<UnixSeconds>,
}

#[must_use]
pub fn series(samples: &[Sample], window: Window, now: UnixSeconds) -> Series {
    let start = now.minus(window.seconds());
    let visible = visible(samples, start);
    Series {
        span: Span { start, end: now },
        segments: segments(&visible, now),
        charging: charging_spans(&visible, now),
        last_full_charge: last_full_charge(samples),
    }
}

#[must_use]
pub fn last_full_charge(samples: &[Sample]) -> Option<UnixSeconds> {
    samples
        .iter()
        .rev()
        .find(|sample| sample.reading.percent.get() == FULL)
        .map(|sample| sample.at)
}

fn visible(samples: &[Sample], start: UnixSeconds) -> Vec<Sample> {
    let first_inside = samples.partition_point(|sample| sample.at < start);
    let carried = first_inside
        .checked_sub(1)
        .and_then(|index| samples.get(index))
        .map(|sample| Sample {
            at: start,
            ..*sample
        });
    carried
        .into_iter()
        .chain(samples.iter().skip(first_inside).copied())
        .collect()
}

fn holds(visible: &[Sample], now: UnixSeconds) -> impl Iterator<Item = (Sample, UnixSeconds)> + '_ {
    let ends = visible
        .iter()
        .skip(1)
        .map(|sample| sample.at)
        .chain(iter::once(now));
    visible.iter().copied().zip(ends)
}

fn segments(visible: &[Sample], now: UnixSeconds) -> Vec<Vec<Point>> {
    let mut segments: Vec<Vec<Point>> = Vec::new();
    let mut open = false;
    for (sample, until) in holds(visible, now) {
        let connected = sample.reading.is_connected();
        if connected && !open {
            segments.push(Vec::new());
        }
        open = connected;
        if let (true, Some(segment)) = (connected, segments.last_mut()) {
            segment.extend([point(sample.at, &sample), point(until, &sample)]);
        }
    }
    segments
}

fn charging_spans(visible: &[Sample], now: UnixSeconds) -> Vec<Span> {
    let mut spans: Vec<Span> = Vec::new();
    let plugged = holds(visible, now).filter(|(sample, _)| sample.reading.is_plugged());
    for (sample, until) in plugged {
        match spans.last_mut() {
            Some(last) if last.end == sample.at => last.end = until,
            _ => spans.push(Span {
                start: sample.at,
                end: until,
            }),
        }
    }
    spans
}

const fn point(at: UnixSeconds, sample: &Sample) -> Point {
    Point {
        at,
        percent: sample.reading.percent,
    }
}
