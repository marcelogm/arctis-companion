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
use super::engines::{Cause, Decision, Trigger};
use super::input::Input;
use super::quitting::Quit;
use super::routing::AudioState;
use super::routing::{Route, needs_equalizer};
use super::seen_apps::SeenApp;
use super::seen_apps::{newly_playing, remember};
use super::selection::{ActivePreset, Selection};
use super::selection::{PATHS, candidates};
use super::snapshot::Snapshot;
use crate::battery::sample::Reading;
use crate::eq::curve::Curve;
use crate::eq::curve::equivalent;
use crate::eq::preset::PresetId;
use crate::messages::audio::SoundPath;
use crate::messages::audio::{AudioCommand, AudioEvent, StreamInfo};
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
    pub(super) quit: Quit,
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
            quit: Quit::Running,
            deadlines: Deadlines::default(),
            published: None,
        }
    }

    pub fn handle(&mut self, input: Input, now: Millis) -> Vec<Action> {
        let mut out = Vec::new();
        let cause = self.dispatch(input, now, &mut out);
        self.advance(Trigger { now, cause }, &mut out);
        self.finish(&mut out);
        out
    }

    fn dispatch(&mut self, input: Input, now: Millis, out: &mut Vec<Action>) -> Cause {
        match input {
            Input::Device(update) => self.device(update, now, out),
            Input::Audio(event) => self.audio_event(&event, out),
            Input::Woke => self.deadlines.woke(),
            Input::SettingsChanged(settings) => self.settings_changed(settings, out),
            Input::Intent(intent) => {
                self.intent(intent, out);
                return Cause::User;
            }
            Input::Quitting => {
                self.start_quit(now, out);
                return Cause::User;
            }
        }
        Cause::Automatic
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
        if path == SoundPath::Dongle && self.quit != Quit::Running {
            return ActivePreset::route(self.route_preset(path));
        }
        self.selection.path(path).active(&self.route_preset(path))
    }

    fn finish(&mut self, out: &mut Vec<Action>) {
        let deadlines = [
            self.selection.next_deadline(),
            self.engines.next_deadline(),
            self.quit.deadline(),
        ];
        out.extend(self.deadlines.schedule(earliest(deadlines)));
        let snapshot = self.snapshot();
        if self.published.as_ref() != Some(&snapshot) {
            self.published = Some(Rc::clone(&snapshot));
            out.push(Action::Publish(snapshot));
        }
    }

    fn advance(&mut self, trigger: Trigger, out: &mut Vec<Action>) {
        self.try_adopt(out);
        self.update_ruled_sets(trigger.now);
        for path in PATHS {
            self.settle(path, trigger, out);
        }
        if self.engines.editor.take_due(trigger.now) {
            self.write_active_headset_curve(out);
        }
        self.finish_quit(trigger.now, out);
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

    fn audio_event(&mut self, event: &AudioEvent, out: &mut Vec<Action>) {
        if let AudioEvent::Streams(streams) = event {
            self.record_seen_apps(streams, out);
        }
        if *event == AudioEvent::BluetoothNodeSeen {
            self.bluetooth_seen(out);
        }
        let route = self.audio.route();
        self.audio.apply(event);
        if self.audio.route() != route {
            self.selection.end_manual_picks();
        }
        self.place_equalizer(out);
    }

    fn record_seen_apps(&mut self, streams: &[StreamInfo], out: &mut Vec<Action>) {
        for app in newly_playing(&self.audio.streams, streams) {
            self.seen_apps = remember(&self.seen_apps, app.clone());
            out.push(Action::RecordSeenApp(app));
        }
    }

    fn bluetooth_seen(&mut self, out: &mut Vec<Action>) {
        if !self.settings.bluetooth_sink_seen {
            self.settings.bluetooth_sink_seen = true;
            out.push(Action::Persist(PersistedChange::BluetoothSinkSeen));
        }
    }

    fn place_equalizer(&mut self, out: &mut Vec<Action>) {
        let wanted =
            self.audio.default_sink.is_some_and(needs_equalizer) && self.quit == Quit::Running;
        if wanted && self.audio.equalizer.is_none() {
            let curve = self.active_curve(SoundPath::Bluetooth);
            self.audio.equalizer = Some(curve.clone());
            out.push(Action::SendAudio(AudioCommand::InsertEqualizer(curve)));
        } else if self.audio.route() != Route::Bluetooth || self.audio.easy_effects_active() {
            self.remove_equalizer(out);
        }
    }

    pub(super) fn remove_equalizer(&mut self, out: &mut Vec<Action>) {
        if self.audio.equalizer.take().is_some() {
            out.push(Action::SendAudio(AudioCommand::RemoveEqualizer));
        }
    }

    fn update_ruled_sets(&mut self, now: Millis) {
        let automatic = self.settings.automatic_presets;
        for path in PATHS {
            let running = candidates(
                &self.audio.streams,
                &self.settings.app_rules,
                &self.catalog,
                path,
            );
            let followed = automatic.then_some(running.as_slice());
            self.selection.path_mut(path).follow(followed, now);
        }
    }

    fn settle(&mut self, path: SoundPath, trigger: Trigger, out: &mut Vec<Action>) {
        let target = self.target(path);
        match self.engines.decide(path, &target, trigger) {
            Decision::Keep | Decision::Hold => {}
            Decision::Relabel => self.engines.record(path, target, None),
            Decision::Apply => {
                let wrote = self.apply_curve(path, &target.preset, out);
                let spent = (wrote && trigger.cause == Cause::Automatic).then_some(trigger.now);
                self.engines.record(path, target, spent);
            }
        }
    }

    fn apply_curve(&mut self, path: SoundPath, preset: &PresetId, out: &mut Vec<Action>) -> bool {
        let curve = self.catalog.resolve(preset).curve().clone();
        match path {
            SoundPath::Dongle => self.write_headset_curve(curve, out),
            SoundPath::Bluetooth => self.retarget_system_curve(curve, out),
        }
    }

    fn write_headset_curve(&mut self, curve: Curve, out: &mut Vec<Action>) -> bool {
        if self.adoption() != AdoptionStatus::Adopted {
            return false;
        }
        let on_headset = self.on_headset(&curve);
        self.engines.editor.cancel();
        out.push(Action::SendDevice(Change::Desired {
            settings: self.settings.headset.with_curve(curve),
            policy: SyncPolicy::Enforce,
        }));
        !on_headset
    }

    pub(super) fn on_headset(&self, curve: &Curve) -> bool {
        self.dongle
            .observed
            .as_ref()
            .is_some_and(|observed| equivalent(&observed.equalizer, curve))
    }

    pub(super) fn retarget_system_curve(&mut self, curve: Curve, out: &mut Vec<Action>) -> bool {
        let running = self.audio.equalizer.as_ref();
        if running.is_none_or(|current| *current == curve) {
            return false;
        }
        self.audio.equalizer = Some(curve.clone());
        out.push(Action::SendAudio(AudioCommand::Retarget(curve)));
        true
    }

    fn write_active_headset_curve(&mut self, out: &mut Vec<Action>) {
        let curve = self.active_curve(SoundPath::Dongle);
        self.write_headset_curve(curve, out);
    }

    fn settings_changed(&mut self, settings: AppSettings, out: &mut Vec<Action>) {
        let headset_changed = settings.headset != self.settings.headset;
        let threshold_changed =
            settings.low_battery_threshold != self.settings.low_battery_threshold;
        self.settings = settings;
        if headset_changed && self.adoption() == AdoptionStatus::Adopted {
            out.push(self.sync_policy());
        }
        if threshold_changed {
            let alerts = self
                .battery
                .threshold_changed(self.settings.low_battery_threshold);
            out.extend(alerts.into_iter().map(Action::from));
        }
    }
}
