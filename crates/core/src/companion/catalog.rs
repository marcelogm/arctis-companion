use crate::eq::curve::{Curve, equivalent};
use crate::eq::library::{self, FLAT_ID};
use crate::eq::preset::{Preset, PresetId, PresetKind};

pub const HEADSET_CURVE_NAME: &str = "Headset Curve";
const FALLBACK_SLUG: &str = "preset";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    presets: Vec<Preset>,
}

impl Catalog {
    #[must_use]
    pub fn new(custom: Vec<Preset>) -> Self {
        let mut catalog = Self {
            presets: library::library(),
        };
        for preset in custom {
            catalog.save(preset);
        }
        catalog
    }

    #[must_use]
    pub fn all(&self) -> &[Preset] {
        &self.presets
    }

    #[must_use]
    pub fn find(&self, id: &PresetId) -> Option<&Preset> {
        find_in(&self.presets, id)
    }

    #[must_use]
    pub fn contains(&self, id: &PresetId) -> bool {
        self.find(id).is_some()
    }

    #[must_use]
    pub fn resolve(&self, id: &PresetId) -> Preset {
        resolve_in(&self.presets, id)
    }

    #[must_use]
    pub fn resolve_id(&self, id: &PresetId) -> PresetId {
        resolve_id_in(&self.presets, id)
    }

    #[must_use]
    pub fn matching(&self, curve: &Curve) -> Option<PresetId> {
        self.presets
            .iter()
            .find(|preset| equivalent(preset.curve(), curve))
            .map(|preset| preset.id().clone())
    }

    pub fn save(&mut self, preset: Preset) -> bool {
        if preset.kind() == PresetKind::BuiltIn || library::find(preset.id()).is_some() {
            return false;
        }
        match self
            .presets
            .iter_mut()
            .find(|known| known.id() == preset.id())
        {
            Some(known) => *known = preset,
            None => self.presets.push(preset),
        }
        true
    }

    pub fn delete(&mut self, id: &PresetId) -> bool {
        let before = self.presets.len();
        self.presets
            .retain(|preset| preset.id() != id || preset.kind() == PresetKind::BuiltIn);
        self.presets.len() != before
    }

    #[must_use]
    pub fn fresh_id(&self, name: &str) -> PresetId {
        fresh_id(&self.presets, name)
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[must_use]
pub fn find_in<'a>(presets: &'a [Preset], id: &PresetId) -> Option<&'a Preset> {
    presets.iter().find(|preset| preset.id() == id)
}

#[must_use]
pub fn resolve_in(presets: &[Preset], id: &PresetId) -> Preset {
    find_in(presets, id).cloned().unwrap_or_else(library::flat)
}

#[must_use]
pub fn resolve_id_in(presets: &[Preset], id: &PresetId) -> PresetId {
    if find_in(presets, id).is_some() {
        id.clone()
    } else {
        PresetId::new(FLAT_ID)
    }
}

#[must_use]
pub fn fresh_id(taken: &[Preset], name: &str) -> PresetId {
    let base = slug(name);
    let is_free = |candidate: &PresetId| taken.iter().all(|preset| preset.id() != candidate);
    (1..=taken.len() + 1)
        .map(|n| numbered(&base, n))
        .find(is_free)
        .unwrap_or_else(|| PresetId::new(base.clone()))
}

fn numbered(base: &str, n: usize) -> PresetId {
    if n == 1 {
        PresetId::new(base)
    } else {
        PresetId::new(format!("{base}-{n}"))
    }
}

fn slug(name: &str) -> String {
    let words: Vec<String> = name
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    if words.is_empty() {
        FALLBACK_SLUG.to_owned()
    } else {
        words.join("-")
    }
}
