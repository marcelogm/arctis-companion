use super::text::{Arg, Text, preset_arg, preset_title, saturating_count};
use crate::companion::selection::PresetReason;
use crate::companion::snapshot::Snapshot;
use crate::eq::curve::MAX_FILTERS;
use crate::eq::filter::{Filter, FilterKind, Gain, Hertz, Q};
use crate::eq::preset::{Preset, PresetId, PresetKind};
use crate::eq::response::response;
use crate::l10n::{forms, msgid};

pub const GRAPH_POINTS: u16 = 96;
const GRAPH_LOW_HZ: f32 = 20.0;
const GRAPH_SPAN: f32 = 1_000.0;
const GRAPH_DB_RANGE: f32 = 10.0;
pub const GRID_HZ: [f32; 3] = [100.0, 1_000.0, 10_000.0];
pub const GRID_DB: [f32; 5] = [-10.0, -5.0, 0.0, 5.0, 10.0];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BandModel {
    pub index: usize,
    pub title: Text,
    pub kind: FilterKind,
    pub hz: Hertz,
    pub gain: Gain,
    pub q: Q,
    pub enabled: bool,
    pub gain_sensitive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphPoint {
    pub hz: f32,
    pub db: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphModel {
    pub points: Vec<GraphPoint>,
    pub description: Text,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EqEditorModel {
    pub id: PresetId,
    pub title: Text,
    pub editable: bool,
    pub bands: Vec<BandModel>,
    pub can_add_band: bool,
    pub graph: GraphModel,
    pub preview: PreviewState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewState {
    Unavailable,
    Available,
    Picked,
}

impl PreviewState {
    #[must_use]
    pub const fn is_available(self) -> bool {
        !matches!(self, Self::Unavailable)
    }
}

#[must_use]
pub fn eq_editor(snapshot: &Snapshot, id: &PresetId) -> Option<EqEditorModel> {
    snapshot
        .presets
        .find(id)
        .map(|preset| eq_editor_for(snapshot, preset))
}

#[must_use]
pub fn eq_editor_for(snapshot: &Snapshot, preset: &Preset) -> EqEditorModel {
    let filters = preset.curve().filters();
    EqEditorModel {
        id: preset.id().clone(),
        title: preset_title(preset),
        editable: preset.kind() == PresetKind::Custom,
        bands: filters
            .iter()
            .zip(0_usize..)
            .map(|(filter, index)| band(*filter, index))
            .collect(),
        can_add_band: filters.len() < MAX_FILTERS,
        graph: graph(preset),
        preview: preview_state(snapshot, preset.id()),
    }
}

fn preview_state(snapshot: &Snapshot, id: &PresetId) -> PreviewState {
    let Ok(path) = snapshot.picked_path() else {
        return PreviewState::Unavailable;
    };
    let active = snapshot.presets.active.get(path);
    if active.preset == *id && matches!(active.reason, PresetReason::Manual(_)) {
        PreviewState::Picked
    } else {
        PreviewState::Available
    }
}

#[must_use]
pub fn x_fraction(hz: f32) -> f32 {
    ((hz / GRAPH_LOW_HZ).log10() / GRAPH_SPAN.log10()).clamp(0.0, 1.0)
}

#[must_use]
pub fn y_fraction(db: f32) -> f32 {
    ((GRAPH_DB_RANGE - db) / (2.0 * GRAPH_DB_RANGE)).clamp(0.0, 1.0)
}

#[must_use]
pub fn kind_at(position: usize) -> Option<FilterKind> {
    FilterKind::ALL.get(position).copied()
}

#[must_use]
pub const fn kind_position(kind: FilterKind) -> usize {
    match kind {
        FilterKind::Peaking => 0,
        FilterKind::LowShelf => 1,
        FilterKind::HighShelf => 2,
        FilterKind::LowPass => 3,
        FilterKind::HighPass => 4,
    }
}

#[must_use]
pub fn band_title(filter: Filter) -> Text {
    let frequency = ("frequency", Arg::Hertz(filter.hz()));
    if filter.kind().has_gain() {
        return Text::format(
            msgid("{frequency} · {gain}"),
            vec![frequency, ("gain", Arg::Decibels(filter.gain()))],
        );
    }
    Text::format(
        msgid("{frequency} · {kind}"),
        vec![
            frequency,
            ("kind", Arg::Msgid(kind_name(filter.kind()).to_owned())),
        ],
    )
}

#[must_use]
pub const fn kind_name(kind: FilterKind) -> &'static str {
    match kind {
        FilterKind::Peaking => msgid("Peak"),
        FilterKind::LowShelf => msgid("Low Shelf"),
        FilterKind::HighShelf => msgid("High Shelf"),
        FilterKind::LowPass => msgid("Low-Pass"),
        FilterKind::HighPass => msgid("High-Pass"),
    }
}

#[must_use]
pub fn kind_items() -> Vec<Text> {
    FilterKind::ALL
        .into_iter()
        .map(|kind| Text::plain(kind_name(kind)))
        .collect()
}

#[must_use]
pub fn graph_frequencies() -> Vec<f32> {
    let last = f32::from(GRAPH_POINTS - 1);
    (0..GRAPH_POINTS)
        .map(|step| GRAPH_LOW_HZ * GRAPH_SPAN.powf(f32::from(step) / last))
        .collect()
}

fn band(filter: Filter, index: usize) -> BandModel {
    BandModel {
        index,
        title: band_title(filter),
        kind: filter.kind(),
        hz: filter.hz(),
        gain: filter.gain(),
        q: filter.q(),
        enabled: filter.is_enabled(),
        gain_sensitive: filter.kind().has_gain(),
    }
}

fn graph(preset: &Preset) -> GraphModel {
    let frequencies = graph_frequencies();
    let levels = response(preset.curve(), &frequencies);
    let active = preset
        .curve()
        .filters()
        .iter()
        .filter(|filter| filter.is_enabled())
        .count();
    GraphModel {
        points: frequencies
            .into_iter()
            .zip(levels)
            .map(|(hz, db)| GraphPoint { hz, db })
            .collect(),
        description: Text::plural(
            forms(
                "Nominal frequency response of {preset}, with {n} active band",
                "Nominal frequency response of {preset}, with {n} active bands",
            ),
            saturating_count(active),
            vec![("preset", preset_arg(preset))],
        ),
    }
}
