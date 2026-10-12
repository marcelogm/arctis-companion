use super::eq_editor::kind_name;
use super::text::{Arg, Text, preset_arg, saturating_count};
use crate::eq::filter::Filter;
use crate::eq::preset::Preset;
use crate::eq::response::response;
use crate::l10n::{forms, msgid};

pub const GRAPH_POINTS: u16 = 160;
pub const LOW_HZ: f32 = 20.0;
pub const HIGH_HZ: f32 = 20_000.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphPoint {
    pub hz: f32,
    pub db: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineStyle {
    Solid,
    Dashed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorRole {
    Accent,
    Foreground,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Series {
    pub points: Vec<GraphPoint>,
    pub style: LineStyle,
    pub role: ColorRole,
    pub filled: bool,
    pub label: Option<Text>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LevelRange {
    pub low_db: f32,
    pub high_db: f32,
}

impl LevelRange {
    pub const RELATIVE: Self = Self {
        low_db: -12.0,
        high_db: 12.0,
    };

    #[must_use]
    pub fn span(self) -> f32 {
        (self.high_db - self.low_db).max(f32::EPSILON)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Handle {
    pub index: usize,
    pub filter: Filter,
    pub number: u32,
    pub hz: f32,
    pub db: f32,
    pub gain_adjustable: bool,
    pub enabled: bool,
    pub label: Text,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphModel {
    pub series: Vec<Series>,
    pub levels: LevelRange,
    pub handles: Vec<Handle>,
    pub description: Text,
}

#[must_use]
pub fn graph(preset: &Preset) -> GraphModel {
    let filters = preset.curve().filters();
    GraphModel {
        series: vec![response_series(preset)],
        levels: LevelRange::RELATIVE,
        handles: filters
            .iter()
            .zip(0_usize..)
            .map(|(filter, index)| handle(*filter, index))
            .collect(),
        description: description(preset),
    }
}

#[must_use]
pub fn graph_frequencies() -> Vec<f32> {
    let last = f32::from(GRAPH_POINTS - 1);
    let span = HIGH_HZ / LOW_HZ;
    (0..GRAPH_POINTS)
        .map(|step| LOW_HZ * span.powf(f32::from(step) / last))
        .collect()
}

#[must_use]
pub fn handle_label(filter: Filter, index: usize) -> Text {
    let mut args = vec![
        ("number", Arg::Number(saturating_count(index + 1))),
        ("kind", Arg::Msgid(kind_name(filter.kind()).to_owned())),
        ("frequency", Arg::Hertz(filter.hz())),
        ("q", Arg::Quality(filter.q())),
    ];
    if !filter.kind().has_gain() {
        return Text::format(msgid("Band {number}, {kind}, {frequency}, Q {q}"), args);
    }
    args.push(("gain", Arg::Decibels(filter.gain())));
    Text::format(
        msgid("Band {number}, {kind}, {frequency}, {gain}, Q {q}"),
        args,
    )
}

fn handle(filter: Filter, index: usize) -> Handle {
    let gain_adjustable = filter.kind().has_gain();
    Handle {
        index,
        filter,
        number: saturating_count(index + 1),
        hz: f32::from(filter.hz().get()),
        db: if gain_adjustable {
            f32::from(filter.gain().half_db()) / 2.0
        } else {
            0.0
        },
        gain_adjustable,
        enabled: filter.is_enabled(),
        label: handle_label(filter, index),
    }
}

fn response_series(preset: &Preset) -> Series {
    let frequencies = graph_frequencies();
    let levels = response(preset.curve(), &frequencies);
    Series {
        points: frequencies
            .into_iter()
            .zip(levels)
            .map(|(hz, db)| GraphPoint { hz, db })
            .collect(),
        style: LineStyle::Solid,
        role: ColorRole::Accent,
        filled: true,
        label: None,
    }
}

fn description(preset: &Preset) -> Text {
    let active = preset
        .curve()
        .filters()
        .iter()
        .filter(|filter| filter.is_enabled())
        .count();
    Text::plural(
        forms(
            "Nominal frequency response of {preset}, with {n} active band",
            "Nominal frequency response of {preset}, with {n} active bands",
        ),
        saturating_count(active),
        vec![("preset", preset_arg(preset))],
    )
}
