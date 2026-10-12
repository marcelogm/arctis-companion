use crate::eq::curve::{Curve, equivalent};
use crate::eq::library::{self, FLAT_ID};
use crate::eq::preset::{Preset, PresetId, PresetKind};

pub const HEADSET_CURVE_NAME: &str = "Headset Curve";
const FALLBACK_SLUG: &str = "preset";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Catalog {
    custom: Vec<Preset>,
}

impl Catalog {
    #[must_use]
    pub const fn new(custom: Vec<Preset>) -> Self {
        Self { custom }
    }

    #[must_use]
    pub fn all(&self) -> Vec<Preset> {
        library::library()
            .into_iter()
            .chain(self.custom.iter().cloned())
            .collect()
    }

    #[must_use]
    pub fn custom(&self) -> &[Preset] {
        &self.custom
    }

    #[must_use]
    pub fn find(&self, id: &PresetId) -> Option<Preset> {
        library::find(id).or_else(|| self.custom.iter().find(|preset| preset.id() == id).cloned())
    }

    #[must_use]
    pub fn contains(&self, id: &PresetId) -> bool {
        self.find(id).is_some()
    }

    #[must_use]
    pub fn resolve(&self, id: &PresetId) -> Preset {
        self.find(id).unwrap_or_else(library::flat)
    }

    #[must_use]
    pub fn resolve_id(&self, id: &PresetId) -> PresetId {
        if self.contains(id) {
            id.clone()
        } else {
            PresetId::new(FLAT_ID)
        }
    }

    #[must_use]
    pub fn matching(&self, curve: &Curve) -> Option<PresetId> {
        self.all()
            .into_iter()
            .find(|preset| equivalent(preset.curve(), curve))
            .map(|preset| preset.id().clone())
    }

    pub fn save(&mut self, preset: Preset) -> bool {
        if preset.kind() == PresetKind::BuiltIn || library::find(preset.id()).is_some() {
            return false;
        }
        match self
            .custom
            .iter_mut()
            .find(|known| known.id() == preset.id())
        {
            Some(known) => *known = preset,
            None => self.custom.push(preset),
        }
        true
    }

    pub fn delete(&mut self, id: &PresetId) -> bool {
        let before = self.custom.len();
        self.custom.retain(|preset| preset.id() != id);
        self.custom.len() != before
    }

    #[must_use]
    pub fn fresh_id(&self, name: &str) -> PresetId {
        fresh_id(&self.all(), name)
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
