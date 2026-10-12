use super::text::{Arg, Text, preset_arg, preset_title};
use crate::companion::snapshot::Snapshot;
use crate::eq::curve::MAX_FILTERS;
use crate::eq::filter::{Filter, FilterKind, Gain, Hertz, Q};
use crate::eq::preset::{Preset, PresetId, PresetKind};
use crate::eq::response::response;

pub const GRAPH_POINTS: u16 = 96;
const GRAPH_LOW_HZ: f32 = 20.0;
const GRAPH_SPAN: f32 = 1_000.0;

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
    pub preview_available: bool,
}

#[must_use]
pub fn eq_editor(snapshot: &Snapshot, id: &PresetId) -> Option<EqEditorModel> {
    let preset = snapshot.presets.find(id)?;
    let filters = preset.curve().filters();
    Some(EqEditorModel {
        id: id.clone(),
        title: preset_title(preset),
        editable: preset.kind() == PresetKind::Custom,
        bands: filters
            .iter()
            .zip(0_usize..)
            .map(|(filter, index)| band(*filter, index))
            .collect(),
        can_add_band: filters.len() < MAX_FILTERS,
        graph: graph(preset),
        preview_available: snapshot.picked_path().is_ok(),
    })
}

#[must_use]
pub fn band_title(filter: Filter) -> Text {
    let frequency = ("frequency", Arg::Hertz(filter.hz()));
    if filter.kind().has_gain() {
        return Text::format(
            "{frequency} · {gain}",
            vec![frequency, ("gain", Arg::Decibels(filter.gain()))],
        );
    }
    Text::format(
        "{frequency} · {kind}",
        vec![
            frequency,
            ("kind", Arg::Msgid(kind_name(filter.kind()).to_owned())),
        ],
    )
}

#[must_use]
pub const fn kind_name(kind: FilterKind) -> &'static str {
    match kind {
        FilterKind::Peaking => "Peak",
        FilterKind::LowShelf => "Low Shelf",
        FilterKind::HighShelf => "High Shelf",
        FilterKind::LowPass => "Low-Pass",
        FilterKind::HighPass => "High-Pass",
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
            [
                "Nominal frequency response of {preset}, with {n} active band",
                "Nominal frequency response of {preset}, with {n} active bands",
            ],
            u32::try_from(active).unwrap_or(u32::MAX),
            vec![("preset", preset_arg(preset))],
        ),
    }
}
