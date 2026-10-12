use crate::battery::sample::Reading;
use crate::companion::Snapshot;
use crate::companion::adoption::AdoptionStatus;
use crate::companion::alerts::BatteryWarning;
use crate::companion::app_settings::{AppRules, HeadsetValues, RoutePresets};
use crate::companion::per_path::PerPath;
use crate::companion::routing::{AudioService, Route};
use crate::companion::seen_apps::SeenApp;
use crate::companion::selection::ActivePreset;
use crate::companion::snapshot::{AudioSnapshot, BatterySnapshot, DongleSnapshot, PresetsSnapshot};
use crate::eq::library;
use crate::eq::preset::{Preset, PresetId};
use crate::eq::{Curve, Filter, FilterKind};
use crate::messages::audio::{DefaultSink, SinkKind};
use crate::messages::device::Phase;
use crate::protocol::firmware::{Firmware, Version};
use crate::protocol::model::{Model, ModelLookup};
use crate::protocol::status::{Charging, Link};
use crate::settings::values::Percent;
use crate::settings::values::{AutoPowerOff, MicVolume, MuteLight, Sidetone, WirelessMode};

#[must_use]
pub fn percent(value: u8) -> Percent {
    Percent::try_from(value).expect("percent")
}

#[must_use]
pub fn id(text: &str) -> PresetId {
    PresetId::new(text)
}

#[must_use]
pub fn nova_5() -> Model {
    let ModelLookup::Supported(model) = Model::lookup(0x2232) else {
        panic!("the Nova 5 is supported");
    };
    model
}

#[must_use]
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
        active: PerPath {
            dongle: ActivePreset::route(id("flat")),
            bluetooth: ActivePreset::route(id("flat")),
        },
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

#[must_use]
pub fn snapshot() -> Snapshot {
    Snapshot {
        dongle: dongle(),
        audio: audio(),
        presets: presets(),
        battery: battery(),
    }
}

#[must_use]
pub fn with(phase: Phase, route: Route) -> Snapshot {
    let mut s = snapshot();
    s.dongle.phase = phase;
    s.audio.route = route;
    s
}

#[must_use]
pub fn rules(entries: &[(&str, &str)]) -> AppRules {
    entries
        .iter()
        .map(|(app, preset)| ((*app).to_owned(), id(preset)))
        .collect()
}

#[must_use]
pub fn seen_app(name: &str, identifiers: &[&str]) -> SeenApp {
    SeenApp {
        key: identifiers
            .first()
            .map_or_else(String::new, |key| (*key).to_owned()),
        name: name.to_owned(),
        identifiers: identifiers
            .iter()
            .map(|identifier| (*identifier).to_owned())
            .collect(),
    }
}

#[must_use]
pub fn filter(kind: FilterKind, hz: u16, half_db: i8, milli_q: u16) -> Filter {
    Filter::from_parts(kind, hz, half_db, milli_q).expect("filter in the envelope")
}

#[must_use]
pub fn custom_preset(filters: &[Filter]) -> Preset {
    let curve = Curve::new(filters.to_vec()).expect("at most ten filters");
    Preset::custom(id("mine"), "Mine".to_owned(), curve)
}

#[must_use]
pub fn with_preset(preset: &Preset) -> Snapshot {
    let mut s = snapshot();
    s.presets.all.push(preset.clone());
    s
}
