use super::curve::Curve;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PresetId(String);

impl PresetId {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PresetKind {
    BuiltIn,
    Custom,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preset {
    id: PresetId,
    name: String,
    curve: Curve,
    kind: PresetKind,
}

impl Preset {
    #[must_use]
    pub fn built_in(id: &str, name: &str, curve: Curve) -> Self {
        Self {
            id: PresetId::new(id),
            name: name.to_owned(),
            curve,
            kind: PresetKind::BuiltIn,
        }
    }

    #[must_use]
    pub fn custom(id: PresetId, name: String, curve: Curve) -> Self {
        Self {
            id,
            name,
            curve,
            kind: PresetKind::Custom,
        }
    }

    #[must_use]
    pub fn id(&self) -> &PresetId {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn curve(&self) -> &Curve {
        &self.curve
    }

    #[must_use]
    pub fn kind(&self) -> PresetKind {
        self.kind
    }
}
