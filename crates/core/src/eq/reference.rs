use std::sync::LazyLock;

use super::curve::Curve;
use super::frequency_response::FrequencyResponse;
use crate::l10n::msgid;

pub const CREDITS_TITLE: &str = msgid("Frequency response data");
pub const CREDITS: [&str; 2] = [
    "RTINGS.com, Brüel & Kjær 5128 https://www.rtings.com",
    "AutoEq by Jaakko Pasanen https://github.com/jaakkopasanen/AutoEq",
];
pub const AUTOEQ_COPYRIGHT: &str = "© 2018-2022 Jaakko Pasanen";
pub const NO_TARGET: &str = "none";

const MEASUREMENT: &str = include_str!("../../data/nova5_rtings_5128.csv");
const HARMAN: &str = include_str!("../../data/target_lmg_5128_0.6.csv");
const DIFFUSE_FIELD: &str = include_str!("../../data/target_df_5128.csv");

static MEASURED: LazyLock<FrequencyResponse> = LazyLock::new(|| embedded(MEASUREMENT));
static HARMAN_TARGET: LazyLock<FrequencyResponse> = LazyLock::new(|| embedded(HARMAN).normalised());
static DIFFUSE_FIELD_TARGET: LazyLock<FrequencyResponse> =
    LazyLock::new(|| embedded(DIFFUSE_FIELD).normalised());

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    Harman,
    DiffuseField,
}

impl Target {
    pub const ALL: [Self; 2] = [Self::Harman, Self::DiffuseField];

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Harman => "harman-5128",
            Self::DiffuseField => "diffuse-field-5128",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Harman => msgid("Harman (B&K 5128)"),
            Self::DiffuseField => msgid("Diffuse field (B&K 5128)"),
        }
    }

    #[must_use]
    pub const fn setting(self) -> &'static str {
        match self {
            Self::Harman => "harman",
            Self::DiffuseField => "diffuse-field",
        }
    }

    #[must_use]
    pub fn from_setting(setting: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|target| target.setting() == setting)
    }

    #[must_use]
    pub fn response(self) -> FrequencyResponse {
        match self {
            Self::Harman => HARMAN_TARGET.clone(),
            Self::DiffuseField => DIFFUSE_FIELD_TARGET.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Overlays {
    pub headset: bool,
    pub target: Option<Target>,
}

impl Default for Overlays {
    fn default() -> Self {
        Self {
            headset: true,
            target: Some(Target::Harman),
        }
    }
}

impl Overlays {
    pub const NONE: Self = Self {
        headset: false,
        target: None,
    };

    #[must_use]
    pub const fn any(self) -> bool {
        self.headset || self.target.is_some()
    }

    #[must_use]
    pub const fn with_headset(self, headset: bool) -> Self {
        Self { headset, ..self }
    }

    #[must_use]
    pub fn with_target_setting(self, setting: &str) -> Self {
        Self {
            target: Target::from_setting(setting),
            ..self
        }
    }

    #[must_use]
    pub const fn target_setting(self) -> &'static str {
        match self.target {
            Some(target) => target.setting(),
            None => NO_TARGET,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SeriesId {
    Headset,
    HeadsetWithEq,
    Target(Target),
}

impl SeriesId {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Headset => "headset",
            Self::HeadsetWithEq => "headset-with-eq",
            Self::Target(target) => target.id(),
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Headset => msgid("Headset"),
            Self::HeadsetWithEq => msgid("Headset with EQ"),
            Self::Target(target) => target.label(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Series {
    pub id: SeriesId,
    pub response: FrequencyResponse,
}

#[must_use]
pub fn measurement() -> FrequencyResponse {
    MEASURED.clone()
}

#[must_use]
pub fn headset_response() -> FrequencyResponse {
    MEASURED.normalised()
}

#[must_use]
pub fn headset_response_with(curve: &Curve) -> FrequencyResponse {
    MEASURED.equalized(curve).normalised()
}

#[must_use]
pub fn plot(curve: &Curve) -> Vec<Series> {
    let headset = [
        (SeriesId::Headset, headset_response()),
        (SeriesId::HeadsetWithEq, headset_response_with(curve)),
    ];
    let targets = Target::ALL.map(|target| (SeriesId::Target(target), target.response()));
    headset
        .into_iter()
        .chain(targets)
        .map(|(id, response)| Series { id, response })
        .collect()
}

fn embedded(text: &str) -> FrequencyResponse {
    FrequencyResponse::parse(text).expect("embedded response data parses")
}
