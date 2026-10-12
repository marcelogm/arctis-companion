use super::text::{Arg, Text};
use crate::companion::app_settings::HeadsetValues;
use crate::companion::snapshot::DongleSnapshot;
use crate::l10n::{forms, msgid};
use crate::protocol::firmware::{Firmware, Version};
use crate::protocol::model::{Model, Support};
use crate::settings::headset_settings::Setting;
use crate::settings::values::{AutoPowerOff, MicVolume, MuteLight, Sidetone, WirelessMode};

pub const SIDETONE_MARKS: [(u8, &str); 4] = [
    (0, msgid("Off")),
    (3, msgid("Low")),
    (6, msgid("Medium")),
    (10, msgid("High")),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mark {
    pub value: u8,
    pub label: Text,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScaleModel {
    pub setting: Setting,
    pub title: Text,
    pub value: u8,
    pub max: u8,
    pub marks: Vec<Mark>,
    pub value_text: Text,
    pub subtitle: Option<Text>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwitchModel {
    pub setting: Setting,
    pub title: Text,
    pub subtitle: Option<Text>,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChoiceModel {
    pub setting: Setting,
    pub title: Text,
    pub subtitle: Option<Text>,
    pub items: Vec<Text>,
    pub selected: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceModel {
    pub model: Text,
    pub support: Option<Text>,
    pub firmware: Text,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadsetGroupsModel {
    pub sensitive: bool,
    pub mic_volume: Option<ScaleModel>,
    pub sidetone: Option<ScaleModel>,
    pub mute_light: Option<ScaleModel>,
    pub volume_limiter: Option<SwitchModel>,
    pub extended_range: Option<SwitchModel>,
    pub auto_power_off: Option<ChoiceModel>,
    pub device: DeviceModel,
}

#[must_use]
pub fn headset_groups(dongle: &DongleSnapshot) -> Option<HeadsetGroupsModel> {
    let model = dongle.model?;
    let rows = Rows { dongle, model };
    Some(HeadsetGroupsModel {
        sensitive: dongle.observed.is_some() || !dongle.phase.is_connecting(),
        mic_volume: rows.row(Setting::MicVolume, mic_volume_row),
        sidetone: rows.row(Setting::Sidetone, sidetone_row),
        mute_light: rows.row(Setting::MuteLight, mute_light_row),
        volume_limiter: rows.row(Setting::VolumeLimiter, volume_limiter_row),
        extended_range: rows.row(Setting::WirelessMode, extended_range_row),
        auto_power_off: rows.row(Setting::AutoPowerOff, auto_power_off_row),
        device: device_model(&model, dongle.firmware),
    })
}

#[must_use]
pub fn sidetone_text(value: u8) -> Text {
    SIDETONE_MARKS
        .iter()
        .find(|(mark, _)| *mark == value)
        .map_or_else(
            || step_of(value, Sidetone::MAX),
            |(_, label)| Text::plain(label),
        )
}

#[must_use]
pub fn auto_power_off_text(choice: AutoPowerOff) -> Text {
    if choice == AutoPowerOff::NEVER {
        return Text::plain(msgid("Never"));
    }
    Text::plural(
        forms("{n} minute", "{n} minutes"),
        u32::from(choice.minutes()),
        Vec::new(),
    )
}

struct Rows<'a> {
    dongle: &'a DongleSnapshot,
    model: Model,
}

impl Rows<'_> {
    fn row<T>(&self, setting: Setting, build: fn(HeadsetValues, Option<Text>) -> T) -> Option<T> {
        self.model
            .capabilities
            .supports(setting)
            .then(|| build(self.shown(setting), self.not_applied(setting)))
    }

    fn shown(&self, setting: Setting) -> HeadsetValues {
        self.dongle
            .observed
            .as_ref()
            .filter(|_| self.dongle.not_applied.contains(&setting))
            .map_or(self.dongle.desired, HeadsetValues::from_settings)
    }

    fn not_applied(&self, setting: Setting) -> Option<Text> {
        self.dongle
            .not_applied
            .contains(&setting)
            .then(|| Text::plain(msgid("Not applied")))
    }
}

fn step_of(value: u8, max: u8) -> Text {
    Text::format(
        msgid("{value} of {max}"),
        vec![
            ("value", Arg::Number(u32::from(value))),
            ("max", Arg::Number(u32::from(max))),
        ],
    )
}

fn mic_volume_row(values: HeadsetValues, subtitle: Option<Text>) -> ScaleModel {
    let value = values.mic_volume.get();
    ScaleModel {
        setting: Setting::MicVolume,
        title: Text::plain(msgid("Volume")),
        value,
        max: MicVolume::MAX,
        marks: Vec::new(),
        value_text: step_of(value, MicVolume::MAX),
        subtitle,
    }
}

fn sidetone_row(values: HeadsetValues, subtitle: Option<Text>) -> ScaleModel {
    let value = values.sidetone.get();
    let marks = SIDETONE_MARKS
        .iter()
        .map(|(mark, label)| Mark {
            value: *mark,
            label: Text::plain(label),
        })
        .collect();
    ScaleModel {
        setting: Setting::Sidetone,
        title: Text::plain(msgid("Sidetone")),
        value,
        max: Sidetone::MAX,
        marks,
        value_text: sidetone_text(value),
        subtitle,
    }
}

fn mute_light_row(values: HeadsetValues, subtitle: Option<Text>) -> ScaleModel {
    let marks = MuteLight::ALL
        .into_iter()
        .map(|light| Mark {
            value: light.step(),
            label: mute_light_label(light),
        })
        .collect();
    ScaleModel {
        setting: Setting::MuteLight,
        title: Text::plain(msgid("Mute Light Brightness")),
        value: values.mute_light.step(),
        max: MuteLight::max_step(),
        marks,
        value_text: mute_light_label(values.mute_light),
        subtitle,
    }
}

fn mute_light_label(light: MuteLight) -> Text {
    Text::plain(match light {
        MuteLight::Off => msgid("Off"),
        MuteLight::Low => msgid("Low"),
        MuteLight::Medium => msgid("Medium"),
        MuteLight::High => msgid("High"),
    })
}

fn volume_limiter_row(values: HeadsetValues, subtitle: Option<Text>) -> SwitchModel {
    SwitchModel {
        setting: Setting::VolumeLimiter,
        title: Text::plain(msgid("Volume Limiter")),
        subtitle,
        active: values.volume_limiter,
    }
}

fn extended_range_row(values: HeadsetValues, subtitle: Option<Text>) -> SwitchModel {
    SwitchModel {
        setting: Setting::WirelessMode,
        title: Text::plain(msgid("Extended Range")),
        subtitle: subtitle.or_else(|| {
            Some(Text::plain(msgid(
                "Increases range, lowers microphone quality",
            )))
        }),
        active: values.wireless_mode == WirelessMode::ExtendedRange,
    }
}

fn auto_power_off_row(values: HeadsetValues, subtitle: Option<Text>) -> ChoiceModel {
    let selected = AutoPowerOff::CHOICES
        .iter()
        .position(|choice| *choice == values.auto_power_off)
        .unwrap_or(0);
    ChoiceModel {
        setting: Setting::AutoPowerOff,
        title: Text::plain(msgid("Automatic Power Off")),
        subtitle,
        items: AutoPowerOff::CHOICES
            .into_iter()
            .map(auto_power_off_text)
            .collect(),
        selected,
    }
}

fn device_model(model: &Model, firmware: Option<Firmware>) -> DeviceModel {
    DeviceModel {
        model: Text::verbatim(model.name),
        support: (model.support == Support::Untested).then(|| Text::plain(msgid("Untested model"))),
        firmware: firmware_text(firmware),
    }
}

fn firmware_text(firmware: Option<Firmware>) -> Text {
    let (receiver, headset) =
        firmware.map_or((None, None), |slots| (slots.receiver, slots.headset));
    Text::format(
        msgid("Receiver {receiver}, headset {headset}"),
        vec![
            ("receiver", version_arg(receiver)),
            ("headset", version_arg(headset)),
        ],
    )
}

fn version_arg(version: Option<Version>) -> Arg {
    version.map_or_else(
        || Arg::Msgid(msgid("Unknown").to_owned()),
        |known| Arg::Str(known.to_string()),
    )
}
