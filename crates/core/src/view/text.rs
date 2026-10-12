use crate::battery::sample::UnixSeconds;
use crate::eq::filter::{Gain, Hertz};
use crate::eq::preset::{Preset, PresetKind};
use crate::settings::values::Percent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arg {
    Percent(Percent),
    Decibels(Gain),
    Hertz(Hertz),
    Number(u32),
    Time(UnixSeconds),
    Msgid(String),
    Str(String),
}

pub type Args = Vec<(&'static str, Arg)>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Text {
    Plain(String),
    Verbatim(String),
    Format {
        msgid: String,
        args: Args,
    },
    Plural {
        singular: String,
        plural: String,
        n: u32,
        args: Args,
    },
}

impl Text {
    #[must_use]
    pub fn plain(msgid: &str) -> Self {
        Self::Plain(msgid.to_owned())
    }

    #[must_use]
    pub fn verbatim(text: &str) -> Self {
        Self::Verbatim(text.to_owned())
    }

    #[must_use]
    pub fn format(msgid: &str, args: Args) -> Self {
        Self::Format {
            msgid: msgid.to_owned(),
            args,
        }
    }

    #[must_use]
    pub fn plural(forms: [&str; 2], n: u32, args: Args) -> Self {
        let [singular, plural] = forms;
        Self::Plural {
            singular: singular.to_owned(),
            plural: plural.to_owned(),
            n,
            args,
        }
    }
}

#[must_use]
pub fn preset_title(preset: &Preset) -> Text {
    match preset.kind() {
        PresetKind::BuiltIn => Text::plain(preset.name()),
        PresetKind::Custom => Text::verbatim(preset.name()),
    }
}

#[must_use]
pub fn preset_arg(preset: &Preset) -> Arg {
    match preset.kind() {
        PresetKind::BuiltIn => Arg::Msgid(preset.name().to_owned()),
        PresetKind::Custom => Arg::Str(preset.name().to_owned()),
    }
}

#[must_use]
pub fn saturating_count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
