use std::rc::Rc;

use super::adoption::AdoptionStatus;
use super::alerts::BatteryWarning;
use super::app_settings::{AppRules, HeadsetValues, RoutePresets};
use super::catalog::{find_in, fresh_id, resolve_id_in, resolve_in};
use super::dongle::WriteFailure;
use super::per_path::PerPath;
use super::reducer::Companion;
use super::routing::{AudioService, Route, easy_effects_active};
use super::seen_apps::SeenApp;
use super::selection::ActivePreset;
use super::selection::{ActivateError, picked_path};
use crate::battery::sample::{Reading, Sample};
use crate::eq::preset::{Preset, PresetId};
use crate::eq::reference::Overlays;
use crate::messages::audio::{DefaultSink, SoundPath};
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
    pub active: PerPath<ActivePreset>,
    pub rules: AppRules,
    pub automatic: bool,
    pub seen_apps: Vec<SeenApp>,
    pub overlays: Overlays,
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

impl DongleSnapshot {
    #[must_use]
    pub const fn is_present(&self) -> bool {
        self.phase.is_present()
    }
}

impl AudioSnapshot {
    #[must_use]
    pub const fn easy_effects_active(&self) -> bool {
        easy_effects_active(self.default_sink)
    }
}

impl PresetsSnapshot {
    #[must_use]
    pub fn find(&self, id: &PresetId) -> Option<&Preset> {
        find_in(&self.all, id)
    }

    #[must_use]
    pub fn resolve(&self, id: &PresetId) -> Preset {
        resolve_in(&self.all, id)
    }

    #[must_use]
    pub fn route_preset(&self, path: SoundPath) -> PresetId {
        resolve_id_in(&self.all, self.route_presets.get(path))
    }
}

impl Snapshot {
    pub fn picked_path(&self) -> Result<SoundPath, ActivateError> {
        picked_path(
            self.audio.route,
            self.dongle.is_present(),
            self.dongle.adoption,
        )
    }

    #[must_use]
    pub fn fresh_preset_id(&self, name: &str) -> PresetId {
        fresh_id(&self.presets.all, name)
    }
}

impl Companion {
    #[must_use]
    pub fn snapshot(&self) -> Rc<Snapshot> {
        Rc::new(Snapshot {
            dongle: self.dongle_snapshot(),
            audio: self.audio_snapshot(),
            presets: self.presets_snapshot(),
            battery: self.battery_snapshot(),
        })
    }

    fn dongle_snapshot(&self) -> DongleSnapshot {
        let dongle = &self.dongle;
        DongleSnapshot {
            phase: dongle.phase,
            adoption: self.adoption(),
            model: dongle.model,
            firmware: dongle.firmware,
            desired: self.settings.headset,
            observed: dongle.observed.clone(),
            not_applied: dongle.not_applied.clone(),
            last_failure: dongle.last_failure,
        }
    }

    fn audio_snapshot(&self) -> AudioSnapshot {
        AudioSnapshot {
            route: self.audio.route(),
            service: self.audio.service,
            default_sink: self.audio.default_sink,
            manager_denied: self.audio.manager_denied,
            bluetooth_sink_seen: self.settings.bluetooth_sink_seen,
        }
    }

    fn presets_snapshot(&self) -> PresetsSnapshot {
        PresetsSnapshot {
            all: self.catalog.all().to_vec(),
            route_presets: self.settings.route_presets.clone(),
            active: PerPath::from_fn(|path| self.active(path)),
            rules: self.settings.app_rules.clone(),
            automatic: self.settings.automatic_presets,
            seen_apps: self.seen_apps.clone(),
            overlays: self.settings.overlays,
        }
    }

    fn battery_snapshot(&self) -> BatterySnapshot {
        BatterySnapshot {
            reading: self.battery.reading(),
            warning: self.battery.warning(),
            history: self.battery.history().to_vec(),
            low_threshold: self.settings.low_battery_threshold,
        }
    }
}
