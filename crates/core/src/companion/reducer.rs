use std::rc::Rc;

use super::action::Action;
use super::action::PresetChange;
use super::adoption::AdoptionStatus;
use super::adoption::adopt;
use super::app_settings::PersistedChange;
use super::app_settings::{AppSettings, Persisted};
use super::battery_watch::BatteryWatch;
use super::catalog::Catalog;
use super::deadlines::{Deadlines, earliest};
use super::dongle::DongleState;
use super::engines::Engines;
use super::input::Input;
use super::routing::AudioState;
use super::seen_apps::SeenApp;
use super::selection::{ActivePreset, Selection};
use super::snapshot::Snapshot;
use crate::battery::sample::Reading;
use crate::eq::curve::Curve;
use crate::eq::preset::PresetId;
use crate::messages::audio::SoundPath;
use crate::messages::device::{Change, SyncPolicy};
use crate::messages::device::{DeviceUpdate, Phase};
use crate::settings::headset_settings::HeadsetSettings;
use crate::settings::headset_settings::SettingChange;
use crate::time::Millis;

#[derive(Debug)]
pub struct Companion {
    pub(super) settings: AppSettings,
    pub(super) catalog: Catalog,
    pub(super) seen_apps: Vec<SeenApp>,
    pub(super) dongle: DongleState,
    pub(super) audio: AudioState,
    pub(super) selection: Selection,
    pub(super) engines: Engines,
    pub(super) battery: BatteryWatch,
    deadlines: Deadlines,
    published: Option<Rc<Snapshot>>,
}

impl Companion {
    #[must_use]
    pub fn new(persisted: Persisted) -> Self {
        Self {
            settings: persisted.settings,
            catalog: Catalog::new(persisted.custom_presets),
            seen_apps: persisted.seen_apps,
            dongle: DongleState::new(),
            audio: AudioState::new(),
            selection: Selection::default(),
            engines: Engines::default(),
            battery: BatteryWatch::new(persisted.battery),
            deadlines: Deadlines::default(),
            published: None,
        }
    }

    pub fn handle(&mut self, input: Input, now: Millis) -> Vec<Action> {
        let mut out = Vec::new();
        self.dispatch(input, now, &mut out);
        self.advance(&mut out);
        self.finish(&mut out);
        out
    }

    fn dispatch(&mut self, input: Input, now: Millis, out: &mut Vec<Action>) {
        match input {
            Input::Device(update) => self.device(update, now, out),
            Input::Woke => self.deadlines.woke(),
            Input::Audio(_) | Input::Intent(_) | Input::SettingsChanged(_) | Input::Quitting => {}
        }
    }

    fn device(&mut self, update: DeviceUpdate, now: Millis, out: &mut Vec<Action>) {
        match update {
            DeviceUpdate::Phase(phase) => self.phase_changed(phase, now, out),
            DeviceUpdate::Identity {
                model,
                uniq,
                firmware,
            } => {
                self.dongle.identify(model, uniq.as_deref(), firmware);
                out.push(self.sync_policy());
            }
            DeviceUpdate::Observed(observed) => {
                let desired = self.desired();
                self.dongle.observe(observed, &desired);
            }
            DeviceUpdate::Status {
                battery,
                charging,
                link,
            } => {
                let reading = Reading {
                    percent: battery,
                    charging,
                    link,
                };
                self.battery_status(reading, now, out);
            }
            DeviceUpdate::HeadsetChanged(change) => self.headset_changed(&change, out),
            DeviceUpdate::WriteFailed(setting) => self.dongle.write_failed(setting),
        }
    }

    fn phase_changed(&mut self, phase: Phase, now: Millis, out: &mut Vec<Action>) {
        self.dongle.set_phase(phase);
        if !self.dongle.is_present() {
            out.extend(self.battery.disconnect(now).map(Action::RecordBattery));
        }
    }

    fn battery_status(&mut self, reading: Reading, now: Millis, out: &mut Vec<Action>) {
        let update = self
            .battery
            .observe(reading, now, self.settings.low_battery_threshold);
        out.extend(update.sample.map(Action::RecordBattery));
        out.extend(update.alerts.into_iter().map(Action::from));
    }

    pub(super) fn adoption(&self) -> AdoptionStatus {
        self.dongle
            .adoption(self.settings.adopted_identity.as_ref())
    }

    pub(super) fn desired(&self) -> HeadsetSettings {
        self.settings
            .headset
            .with_curve(self.active_curve(SoundPath::Dongle))
    }

    pub(super) fn active(&self, path: SoundPath) -> ActivePreset {
        self.engines
            .applied(path)
            .cloned()
            .unwrap_or_else(|| self.target(path))
    }

    pub(super) fn active_curve(&self, path: SoundPath) -> Curve {
        self.catalog
            .resolve(&self.active(path).preset)
            .curve()
            .clone()
    }

    pub(super) fn route_preset(&self, path: SoundPath) -> PresetId {
        self.catalog
            .resolve_id(self.settings.route_presets.get(path))
    }

    fn target(&self, path: SoundPath) -> ActivePreset {
        self.selection.path(path).active(&self.route_preset(path))
    }

    fn finish(&mut self, out: &mut Vec<Action>) {
        let next = earliest([self.selection.next_deadline(), self.engines.next_deadline()]);
        out.extend(self.deadlines.schedule(next));
        let snapshot = self.snapshot();
        if self.published.as_ref() != Some(&snapshot) {
            self.published = Some(Rc::clone(&snapshot));
            out.push(Action::Publish(snapshot));
        }
    }

    fn advance(&mut self, out: &mut Vec<Action>) {
        self.try_adopt(out);
    }

    pub(super) fn sync_policy(&self) -> Action {
        let policy = if self.adoption() == AdoptionStatus::Adopted {
            SyncPolicy::Enforce
        } else {
            SyncPolicy::Adopt
        };
        Action::SendDevice(Change::Desired {
            settings: self.desired(),
            policy,
        })
    }

    fn headset_changed(&mut self, change: &SettingChange, out: &mut Vec<Action>) {
        self.dongle.headset_changed(change);
        if self.adoption() != AdoptionStatus::Adopted {
            return;
        }
        self.settings.headset.apply(change);
        out.push(Action::Persist(PersistedChange::Headset(
            self.settings.headset,
        )));
    }

    fn try_adopt(&mut self, out: &mut Vec<Action>) {
        if self.adoption() != AdoptionStatus::NeverAdopted || !self.dongle.is_ready() {
            return;
        }
        let (Some(identity), Some(observed)) =
            (self.dongle.identity.clone(), self.dongle.observed.clone())
        else {
            return;
        };
        let adoption = adopt(&observed, &self.catalog);
        self.settings.headset = adoption.values;
        out.push(Action::Persist(PersistedChange::Headset(adoption.values)));
        if let Some(preset) = adoption.new_preset {
            self.catalog.save(preset.clone());
            out.push(Action::StorePreset(PresetChange::Save(preset)));
        }
        self.adopt_route_preset(adoption.preset, out);
        self.settings.adopted_identity = Some(identity.clone());
        out.push(Action::Persist(PersistedChange::AdoptedIdentity(identity)));
        out.push(self.sync_policy());
    }

    fn adopt_route_preset(&mut self, preset: PresetId, out: &mut Vec<Action>) {
        self.settings.route_presets.dongle = preset.clone();
        out.push(Action::Persist(PersistedChange::RoutePreset(
            SoundPath::Dongle,
            preset.clone(),
        )));
        self.engines
            .record(SoundPath::Dongle, ActivePreset::route(preset), None);
    }
}
