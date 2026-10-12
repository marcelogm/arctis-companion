use super::text::{Arg, Text, preset_arg, saturating_count};
use crate::companion::snapshot::Snapshot;
use crate::eq::curve::{Curve, MAX_FILTERS};
use crate::eq::decimal::thousandths_of;
use crate::eq::filter::{Filter, FilterKind, Gain, Hertz, Q};
use crate::eq::library;
use crate::eq::preset::{Preset, PresetKind};
use crate::l10n::msgid;

pub const NEW_BAND: Filter = match Filter::from_parts(FilterKind::Peaking, 1_000, 0, 1_410) {
    Ok(filter) => filter,
    Err(_) => panic!("the new band lies outside the envelope"),
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BandEdit {
    Kind(FilterKind),
    Frequency(f64),
    Gain(f64),
    Q(f64),
    Enabled(bool),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub preset: Preset,
    pub starts_burst: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkingCopy {
    preset: Preset,
    before_burst: Option<Preset>,
}

impl WorkingCopy {
    #[must_use]
    pub const fn opened(preset: Preset) -> Self {
        Self {
            preset,
            before_burst: None,
        }
    }

    #[must_use]
    pub const fn preset(&self) -> &Preset {
        &self.preset
    }

    pub fn published(&mut self, snapshot: &Snapshot) {
        if let Some(published) = snapshot.presets.find(self.preset.id()) {
            self.preset = published.clone();
        }
    }

    pub fn apply(&mut self, change: impl FnOnce(&Preset) -> Option<Preset>) -> Option<Applied> {
        let edited = change(&self.preset)?;
        let starts_burst = self.before_burst.is_none();
        if starts_burst {
            self.before_burst = Some(self.preset.clone());
        }
        self.preset = edited.clone();
        Some(Applied {
            preset: edited,
            starts_burst,
        })
    }

    pub fn burst_ended(&mut self) {
        self.before_burst = None;
    }

    pub fn undo(&mut self) -> Option<Preset> {
        let before = self.before_burst.take()?;
        self.preset = before.clone();
        Some(before)
    }
}

#[must_use]
pub fn apply_edit(preset: &Preset, index: usize, edit: BandEdit) -> Option<Preset> {
    let mut filters = editable_filters(preset)?;
    let band = filters.get_mut(index)?;
    *band = edited_band(*band, edit)?;
    with_filters(preset, filters)
}

#[must_use]
pub fn add_band(preset: &Preset) -> Option<Preset> {
    let mut filters = editable_filters(preset)?;
    if filters.len() >= MAX_FILTERS {
        return None;
    }
    filters.push(NEW_BAND);
    with_filters(preset, filters)
}

#[must_use]
pub fn remove_band(preset: &Preset, index: usize) -> Option<Preset> {
    let mut filters = editable_filters(preset)?;
    if index >= filters.len() {
        return None;
    }
    filters.remove(index);
    with_filters(preset, filters)
}

#[must_use]
pub fn renamed(preset: &Preset, name: &str) -> Option<Preset> {
    let name = name.trim();
    let changes = preset.kind() == PresetKind::Custom && !name.is_empty() && name != preset.name();
    changes.then(|| Preset::custom(preset.id().clone(), name.to_owned(), preset.curve().clone()))
}

#[must_use]
pub fn copy_name(preset: &Preset) -> Text {
    Text::format(
        msgid("{preset} (Copy)"),
        vec![("preset", preset_arg(preset))],
    )
}

#[must_use]
pub fn duplicate(snapshot: &Snapshot, source: &Preset, name: &str) -> Preset {
    Preset::custom(
        snapshot.fresh_preset_id(name),
        name.to_owned(),
        source.curve().clone(),
    )
}

#[must_use]
pub fn new_preset_name(number: u32) -> Text {
    if number <= 1 {
        return Text::plain(msgid("Custom Preset"));
    }
    Text::format(
        msgid("Custom Preset {number}"),
        vec![("number", Arg::Number(number))],
    )
}

#[must_use]
pub fn new_preset(snapshot: &Snapshot, name_of: impl Fn(&Text) -> String) -> Preset {
    let presets = &snapshot.presets.all;
    let is_free = |name: &String| presets.iter().all(|preset| preset.name() != name);
    let name = (1..=saturating_count(presets.len() + 1))
        .map(|number| name_of(&new_preset_name(number)))
        .find(is_free)
        .unwrap_or_else(|| name_of(&new_preset_name(1)));
    Preset::custom(
        snapshot.fresh_preset_id(&name),
        name,
        library::flat().curve().clone(),
    )
}

#[must_use]
pub fn saved_toast(preset: &Preset) -> Text {
    Text::format(
        msgid("Saved “{preset}”"),
        vec![("preset", preset_arg(preset))],
    )
}

fn editable_filters(preset: &Preset) -> Option<Vec<Filter>> {
    (preset.kind() == PresetKind::Custom).then(|| preset.curve().filters().to_vec())
}

fn with_filters(preset: &Preset, filters: Vec<Filter>) -> Option<Preset> {
    let curve = Curve::new(filters).ok()?;
    (curve != *preset.curve())
        .then(|| Preset::custom(preset.id().clone(), preset.name().to_owned(), curve))
}

fn edited_band(band: Filter, edit: BandEdit) -> Option<Filter> {
    let (kind, hz, gain, q) = match edit {
        BandEdit::Enabled(enabled) => return Some(band.with_enabled(enabled)),
        BandEdit::Kind(kind) => (kind, band.hz(), band.gain(), band.q()),
        BandEdit::Frequency(value) => (band.kind(), hertz_of(value)?, band.gain(), band.q()),
        BandEdit::Gain(db) => (band.kind(), band.hz(), gain_of(db)?, band.q()),
        BandEdit::Q(value) => (band.kind(), band.hz(), band.gain(), q_of(value)?),
    };
    Some(Filter::dropping_pass_gain(kind, hz, gain, q).with_enabled(band.is_enabled()))
}

fn hertz_of(value: f64) -> Option<Hertz> {
    Some(Hertz::nearest_milli(thousandths_of(value)?).value)
}

fn gain_of(db: f64) -> Option<Gain> {
    Some(Gain::nearest_milli_db(thousandths_of(db)?).value)
}

fn q_of(value: f64) -> Option<Q> {
    Some(Q::clamped_milli(thousandths_of(value)?).value)
}
