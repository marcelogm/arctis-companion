use super::adoption::AdoptionStatus;
use super::alerts::BatteryWarning;
use super::app_settings::{AppRules, HeadsetValues, RoutePresets};
use super::dongle::WriteFailure;
use super::routing::{AudioService, Route};
use super::seen_apps::SeenApp;
use super::selection::ActivePreset;
use crate::battery::sample::{Reading, Sample};
use crate::eq::preset::Preset;
use crate::messages::audio::DefaultSink;
use crate::messages::device::Phase;
use crate::protocol::firmware::Firmware;
use crate::protocol::model::Model;
use crate::settings::headset_settings::{HeadsetSettings, Setting};
use crate::settings::values::Percent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DongleSnapshot {
    pub phase: Phase,
    pub adoption: AdoptionStatus,
    pub model: Option<Model>,
    pub firmware: Option<Firmware>,
    pub desired: HeadsetValues,
    pub observed: Option<HeadsetSettings>,
    pub not_applied: Vec<Setting>,
    pub last_failure: Option<WriteFailure>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioSnapshot {
    pub route: Route,
    pub service: AudioService,
    pub default_sink: Option<DefaultSink>,
    pub manager_denied: bool,
    pub bluetooth_sink_seen: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetsSnapshot {
    pub all: Vec<Preset>,
    pub route_presets: RoutePresets,
    pub dongle: ActivePreset,
    pub bluetooth: ActivePreset,
    pub rules: AppRules,
    pub automatic: bool,
    pub seen_apps: Vec<SeenApp>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatterySnapshot {
    pub reading: Option<Reading>,
    pub warning: BatteryWarning,
    pub history: Vec<Sample>,
    pub low_threshold: Percent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub dongle: DongleSnapshot,
    pub audio: AudioSnapshot,
    pub presets: PresetsSnapshot,
    pub battery: BatterySnapshot,
}
