use super::eq_graph::{GraphModel, graph};
use super::text::{Arg, Text, preset_title};
use crate::companion::selection::PresetReason;
use crate::companion::snapshot::Snapshot;
use crate::eq::curve::MAX_FILTERS;
use crate::eq::filter::{Filter, FilterKind, Gain, Hertz, Q};
use crate::eq::preset::{Preset, PresetId, PresetKind};
use crate::l10n::msgid;

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
