use super::action::{Action, PresetChange};
use super::adoption::{AdoptionStatus, adopt};
use super::app_settings::PersistedChange;
use super::reducer::Companion;
use super::selection::ActivePreset;
use crate::battery::sample::Reading;
use crate::eq::preset::PresetId;
use crate::messages::audio::SoundPath;
use crate::messages::device::{DeviceUpdate, Phase};
use crate::settings::headset_settings::SettingChange;
use crate::time::Millis;

impl Companion {
    pub(super) fn device(&mut self, update: DeviceUpdate, now: Millis, out: &mut Vec<Action>) {
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

    pub(super) fn try_adopt(&mut self, out: &mut Vec<Action>) {
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

    fn phase_changed(&mut self, phase: Phase, now: Millis, out: &mut Vec<Action>) {
        self.dongle.set_phase(phase);
        if !self.dongle.is_present() {
            out.extend(Action::from_battery(self.battery.disconnect(now)));
        }
    }

    fn battery_status(&mut self, reading: Reading, now: Millis, out: &mut Vec<Action>) {
        let update = self
            .battery
            .observe(reading, now, self.settings.low_battery_threshold);
        out.extend(Action::from_battery(update));
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
