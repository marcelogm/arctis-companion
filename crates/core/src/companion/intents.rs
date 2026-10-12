use super::action::{Action, PresetChange};
use super::adoption::AdoptionStatus;
use super::app_settings::PersistedChange;
use super::reducer::Companion;
use super::selection::{ActivateError, ManualPick, picked_path};
use crate::eq::preset::{Preset, PresetId};
use crate::eq::reference::Overlays;
use crate::intent::Intent;
use crate::messages::audio::SoundPath;
use crate::settings::values::Percent;

impl Companion {
    pub(super) fn intent(&mut self, intent: Intent, out: &mut Vec<Action>) {
        match intent {
            setting @ (Intent::SetMicVolume(_)
            | Intent::SetSidetone(_)
            | Intent::SetMuteLight(_)
            | Intent::SetVolumeLimiter(_)
            | Intent::SetExtendedRange(_)
            | Intent::SetAutoPowerOff(_)) => self.change_setting(&setting, out),
            Intent::RetryWrite => self.retry_writes(out),
            Intent::SetRoutePreset(path, preset) => self.set_route_preset(path, preset, out),
            Intent::ActivatePreset(preset) => self.activate_preset(preset, out),
            Intent::UseRoutePreset(path) => self.use_route_preset(path),
            Intent::SetAutomaticPresets(on) => self.set_automatic_presets(on, out),
            Intent::SetAppRule { identifier, preset } => {
                self.set_app_rule(identifier, Some(preset), out);
            }
            Intent::RemoveAppRule(identifier) => self.set_app_rule(identifier, None, out),
            Intent::StartPreview(preset) => self.start_preview(preset),
            Intent::EndPreview => self.end_preview(),
            Intent::SetLowBatteryThreshold(threshold) => {
                self.set_low_battery_threshold(threshold, out);
            }
            Intent::SavePreset(preset) => self.save_preset(preset, out),
            Intent::DeletePreset(preset) => self.delete_preset(&preset, out),
            Intent::SetOverlays(overlays) => self.set_overlays(overlays, out),
            Intent::SetRunInBackground(_) => {}
        }
    }

    pub(super) fn picked_path(&self) -> Result<SoundPath, ActivateError> {
        picked_path(
            self.audio.route(),
            self.dongle.is_present(),
            self.adoption(),
        )
    }

    fn change_setting(&mut self, intent: &Intent, out: &mut Vec<Action>) {
        let Some(change) = intent.setting_change() else {
            return;
        };
        if self.adoption() != AdoptionStatus::Adopted {
            return;
        }
        self.settings.headset.apply(&change);
        out.push(Action::Persist(PersistedChange::Headset(
            self.settings.headset,
        )));
        out.push(self.sync_policy());
    }

    fn retry_writes(&self, out: &mut Vec<Action>) {
        if self.adoption() == AdoptionStatus::Adopted {
            out.push(self.sync_policy());
        }
    }

    fn set_route_preset(&mut self, path: SoundPath, preset: PresetId, out: &mut Vec<Action>) {
        if !self.catalog.contains(&preset) {
            return;
        }
        self.settings.route_presets.set(path, preset.clone());
        out.push(Action::Persist(PersistedChange::RoutePreset(
            path,
            preset.clone(),
        )));
        if !self.selection.get(path).ruled.is_empty() {
            self.pick(path, ManualPick::lasting(preset));
        }
    }

    fn activate_preset(&mut self, preset: PresetId, out: &mut Vec<Action>) {
        let Ok(path) = self.picked_path() else {
            return;
        };
        if self.selection.get(path).ruled.is_empty() {
            self.set_route_preset(path, preset, out);
        } else if self.catalog.contains(&preset) {
            self.pick(path, ManualPick::lasting(preset));
        }
    }

    fn use_route_preset(&mut self, path: SoundPath) {
        let preset = self.route_preset(path);
        self.pick(path, ManualPick::lasting(preset));
    }

    fn pick(&mut self, path: SoundPath, pick: ManualPick) {
        self.selection.get_mut(path).manual = Some(pick);
    }

    fn set_automatic_presets(&mut self, on: bool, out: &mut Vec<Action>) {
        self.settings.automatic_presets = on;
        out.push(Action::Persist(PersistedChange::AutomaticPresets(on)));
    }

    fn set_app_rule(
        &mut self,
        identifier: String,
        preset: Option<PresetId>,
        out: &mut Vec<Action>,
    ) {
        let replaced = self.store_app_rule(&identifier, preset.as_ref());
        let removals = replaced.into_iter().map(|stored| PersistedChange::AppRule {
            identifier: stored,
            preset: None,
        });
        out.extend(removals.map(Action::Persist));
        out.push(Action::Persist(PersistedChange::AppRule {
            identifier,
            preset,
        }));
    }

    fn store_app_rule(&mut self, identifier: &str, preset: Option<&PresetId>) -> Vec<String> {
        let Some(chosen) = preset else {
            self.settings.app_rules.remove(identifier);
            return Vec::new();
        };
        self.settings
            .app_rules
            .set(identifier.to_owned(), chosen.clone())
    }

    fn start_preview(&mut self, preset: PresetId) {
        if let Ok(path) = self.picked_path()
            && self.catalog.contains(&preset)
        {
            self.pick(path, ManualPick::preview(preset));
        }
    }

    fn end_preview(&mut self) {
        self.selection.end_previews();
    }

    fn set_low_battery_threshold(&mut self, threshold: Percent, out: &mut Vec<Action>) {
        self.settings.low_battery_threshold = threshold;
        out.push(Action::Persist(PersistedChange::LowBatteryThreshold(
            threshold,
        )));
        out.extend(
            self.battery
                .threshold_changed(threshold)
                .into_iter()
                .map(Action::from),
        );
    }

    fn set_overlays(&mut self, overlays: Overlays, out: &mut Vec<Action>) {
        self.settings.overlays = overlays;
        out.push(Action::Persist(PersistedChange::Overlays(overlays)));
    }

    fn save_preset(&mut self, preset: Preset, out: &mut Vec<Action>) {
        let id = preset.id().clone();
        if !self.catalog.save(preset.clone()) {
            return;
        }
        out.push(Action::StorePreset(PresetChange::Save(preset)));
        self.curve_changed(&id, out);
    }

    fn delete_preset(&mut self, id: &PresetId, out: &mut Vec<Action>) {
        if !self.catalog.delete(id) {
            return;
        }
        out.push(Action::StorePreset(PresetChange::Delete(id.clone())));
        self.curve_changed(id, out);
    }

    fn curve_changed(&mut self, id: &PresetId, out: &mut Vec<Action>) {
        if self.active(SoundPath::Dongle).preset == *id {
            self.engines.editor.request();
        }
        if self.active(SoundPath::Bluetooth).preset == *id {
            let curve = self.active_curve(SoundPath::Bluetooth);
            self.retarget_system_curve(curve, out);
        }
    }
}
