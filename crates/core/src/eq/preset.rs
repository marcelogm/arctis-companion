use std::fmt;

use serde::{Deserialize, Serialize};

use super::curve::Curve;
use super::decimal::thousandths_of;
use super::filter::{EnvelopeError, Filter, FilterKind, Gain};

pub const FORMAT: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "PresetDocument", into = "PresetDocument")]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetFormatError {
    UnsupportedFormat(u32),
    NotADecimal,
    OffGrid,
    Envelope(EnvelopeError),
}

impl fmt::Display for PresetFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormat(format) => {
                write!(formatter, "unsupported preset format {format}")
            }
            Self::NotADecimal => formatter.write_str("a gain or Q is not a plain decimal number"),
            Self::OffGrid => formatter.write_str("a gain is not a multiple of 0.5 dB"),
            Self::Envelope(EnvelopeError::TooManyFilters) => {
                formatter.write_str("the preset has more than ten filters")
            }
            Self::Envelope(error) => {
                write!(formatter, "a filter lies outside the envelope: {error}")
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct PresetDocument {
    format: u32,
    id: PresetId,
    name: String,
    filters: Vec<FilterDocument>,
}

#[derive(Serialize, Deserialize)]
struct FilterDocument {
    kind: FilterKind,
    hz: u16,
    gain_db: f64,
    q: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
}

impl From<Preset> for PresetDocument {
    fn from(preset: Preset) -> Self {
        let filters = preset
            .curve
            .filters()
            .iter()
            .map(|&filter| FilterDocument::from(filter));
        Self {
            format: FORMAT,
            filters: filters.collect(),
            id: preset.id,
            name: preset.name,
        }
    }
}

impl From<Filter> for FilterDocument {
    fn from(filter: Filter) -> Self {
        Self {
            kind: filter.kind(),
            hz: filter.hz().get(),
            gain_db: filter.gain().db(),
            q: filter.q().value(),
            enabled: (!filter.is_enabled()).then_some(false),
        }
    }
}

impl TryFrom<PresetDocument> for Preset {
    type Error = PresetFormatError;

    fn try_from(document: PresetDocument) -> Result<Self, Self::Error> {
        if document.format != FORMAT {
            return Err(PresetFormatError::UnsupportedFormat(document.format));
        }
        let filters = document
            .filters
            .iter()
            .map(FilterDocument::to_filter)
            .collect::<Result<Vec<_>, _>>()?;
        let curve = Curve::new(filters).map_err(PresetFormatError::Envelope)?;
        Ok(Self::custom(document.id, document.name, curve))
    }
}

impl FilterDocument {
    fn to_filter(&self) -> Result<Filter, PresetFormatError> {
        let half_db = steps(self.gain_db, Gain::MILLI_DB_PER_STEP, EnvelopeError::Gain)?;
        let milli_q = steps(self.q, 1, EnvelopeError::Q)?;
        let filter = Filter::from_parts(self.kind, self.hz, half_db, milli_q)
            .map_err(PresetFormatError::Envelope)?;
        Ok(filter.with_enabled(self.enabled.unwrap_or(true)))
    }
}

fn steps<T: TryFrom<i64>>(
    value: f64,
    step: i64,
    out_of_range: EnvelopeError,
) -> Result<T, PresetFormatError> {
    let thousandths = thousandths_of(value).ok_or(PresetFormatError::NotADecimal)?;
    if thousandths % step != 0 {
        return Err(PresetFormatError::OffGrid);
    }
    T::try_from(thousandths / step).map_err(|_| PresetFormatError::Envelope(out_of_range))
}
