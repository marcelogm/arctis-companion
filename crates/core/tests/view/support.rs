use arctis_core::battery::sample::Reading;
use arctis_core::companion::Snapshot;
use arctis_core::companion::adoption::AdoptionStatus;
use arctis_core::companion::alerts::BatteryWarning;
use arctis_core::companion::app_settings::{AppRules, HeadsetValues, RoutePresets};
use arctis_core::companion::routing::{AudioService, Route};
use arctis_core::companion::selection::ActivePreset;
use arctis_core::companion::snapshot::{
    AudioSnapshot, BatterySnapshot, DongleSnapshot, PresetsSnapshot,
};
use arctis_core::eq::library;
use arctis_core::eq::preset::{Preset, PresetId};
use arctis_core::eq::{Gain, Hertz};
use arctis_core::messages::audio::{DefaultSink, SinkKind};
use arctis_core::messages::device::Phase;
use arctis_core::protocol::firmware::{Firmware, Version};
use arctis_core::protocol::model::{Model, ModelLookup};
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::settings::values::Percent;
use arctis_core::settings::values::{AutoPowerOff, MicVolume, MuteLight, Sidetone, WirelessMode};

pub fn percent(value: u8) -> Percent {
    Percent::try_from(value).expect("percent")
}

pub fn gain(half_db: i8) -> Gain {
    Gain::from_half_db(half_db).expect("gain in the envelope")
}

pub fn hertz(hz: u16) -> Hertz {
    Hertz::new(hz).expect("frequency in the envelope")
}

pub fn id(text: &str) -> PresetId {
    PresetId::new(text)
}

pub fn custom(name: &str) -> Preset {
    Preset::custom(
        id(&name.to_lowercase()),
        name.to_owned(),
        library::flat().curve().clone(),
    )
}

pub fn rules(entries: &[(&str, &str)]) -> AppRules {
    entries
        .iter()
        .map(|(app, preset)| ((*app).to_owned(), id(preset)))
        .collect()
}

pub fn nova_5() -> Model {
    let ModelLookup::Supported(model) = Model::lookup(0x2232) else {
        panic!("the Nova 5 is supported");
    };
    model
}

pub fn values() -> HeadsetValues {
    HeadsetValues {
        mic_volume: MicVolume::try_from(7).expect("mic volume"),
        sidetone: Sidetone::try_from(3).expect("sidetone"),
        mute_light: MuteLight::Low,
        volume_limiter: false,
        wireless_mode: WirelessMode::ExtendedRange,
        auto_power_off: AutoPowerOff::try_from(15).expect("auto power off"),
    }
}

const fn version(major: u8, minor: u8) -> Version {
    Version {
        major,
        minor,
        patch: 0,
    }
}

fn dongle() -> DongleSnapshot {
    DongleSnapshot {
        phase: Phase::Ready,
        adoption: AdoptionStatus::Adopted,
        model: Some(nova_5()),
        firmware: Some(Firmware {
            receiver: Some(version(1, 16)),
            headset: Some(version(0, 9)),
        }),
        desired: values(),
        observed: Some(values().with_curve(library::flat().curve().clone())),
        not_applied: Vec::new(),
        last_failure: None,
    }
}

fn audio() -> AudioSnapshot {
    AudioSnapshot {
        route: Route::Dongle,
        service: AudioService::Available,
        default_sink: Some(DefaultSink {
            terminal: SinkKind::NovaDongle,
            via_equalizer: false,
            via_easy_effects: false,
        }),
        manager_denied: false,
        bluetooth_sink_seen: false,
    }
}

fn presets() -> PresetsSnapshot {
    PresetsSnapshot {
        all: library::library(),
        route_presets: RoutePresets {
            dongle: id("flat"),
            bluetooth: id("flat"),
        },
        dongle: ActivePreset::route(id("flat")),
        bluetooth: ActivePreset::route(id("flat")),
        rules: AppRules::default(),
        automatic: true,
        seen_apps: Vec::new(),
    }
}

fn battery() -> BatterySnapshot {
    BatterySnapshot {
        reading: Some(Reading {
            percent: percent(73),
            charging: Charging::Discharging,
            link: Link::Connected,
        }),
        warning: BatteryWarning::None,
        history: Vec::new(),
        low_threshold: percent(15),
    }
}

pub fn snapshot() -> Snapshot {
    Snapshot {
        dongle: dongle(),
        audio: audio(),
        presets: presets(),
        battery: battery(),
    }
}
