use std::rc::Rc;

use super::action::Action;
use super::adoption::AdoptionStatus;
use super::app_settings::{AppSettings, Persisted};
use super::battery_watch::BatteryWatch;
use super::catalog::Catalog;
use super::deadlines::{Deadlines, earliest};
use super::dongle::DongleState;
use super::engines::{Cause, Decision, Engines, Trigger};
use super::input::Input;
use super::per_path::{PATHS, PerPath};
use super::quitting::Quit;
use super::routing::AudioState;
use super::seen_apps::SeenApp;
use super::selection::{ActivePreset, Selection, candidates};
use super::snapshot::Snapshot;
use crate::eq::curve::{Curve, equivalent};
use crate::eq::preset::PresetId;
use crate::messages::audio::SoundPath;
use crate::messages::device::{Change, SyncPolicy};
use crate::settings::headset_settings::HeadsetSettings;
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
        self.update_ruled_sets(now);
        let before = self.targets();
        let cause = self.dispatch(input, now, &mut out);
        self.try_adopt(&mut out);
        self.update_ruled_sets(now);
        let causes = self.causes(&before, cause);
        self.advance(now, causes, &mut out);
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

    fn targets(&self) -> PerPath<ActivePreset> {
        PerPath::from_fn(|path| self.target(path))
    }

    fn causes(&self, before: &PerPath<ActivePreset>, cause: Cause) -> PerPath<Cause> {
        PerPath::from_fn(|path| self.cause_on(path, before.get(path), cause))
    }

    fn cause_on(&self, path: SoundPath, before: &ActivePreset, cause: Cause) -> Cause {
        if cause == Cause::User && self.target(path) != *before {
            Cause::User
        } else {
            Cause::Automatic
        }
    }

    fn advance(&mut self, now: Millis, causes: PerPath<Cause>, out: &mut Vec<Action>) {
        for path in PATHS {
            let cause = *causes.get(path);
            self.settle(path, Trigger { now, cause }, out);
        }
        if self.engines.editor.take_due(now) {
            self.write_active_headset_curve(out);
        }
        self.finish_quit(now, out);
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
        self.selection.get(path).active(&self.route_preset(path))
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

    pub(super) fn on_headset(&self, curve: &Curve) -> bool {
        self.dongle
            .observed
            .as_ref()
            .is_some_and(|observed| equivalent(&observed.equalizer, curve))
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
            self.selection.get_mut(path).follow(followed, now);
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
