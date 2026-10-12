use std::time::Duration;

use arctis_core::messages::device::{DeviceUpdate, Phase};
use arctis_core::settings::headset_settings::HeadsetSettings;
use arctis_core::settings::values::Sidetone;

#[derive(Debug, Default)]
pub struct SoakLog {
    pub phases: Vec<(Duration, Phase)>,
    pub original: Option<HeadsetSettings>,
    pub observed: Option<HeadsetSettings>,
    pub requested: usize,
    pub confirmed: usize,
    pub failed: usize,
    pub interrupted: usize,
    pub waiting_for: Option<HeadsetSettings>,
    pub unrestored: bool,
}

impl SoakLog {
    pub fn record(&mut self, at: Duration, update: DeviceUpdate) {
        match update {
            DeviceUpdate::Phase(phase) => self.enter(at, phase),
            DeviceUpdate::Observed(settings) => self.observe(settings),
            DeviceUpdate::WriteFailed(_) => self.failed += 1,
            _ => {}
        }
    }

    fn enter(&mut self, at: Duration, phase: Phase) {
        if phase != Phase::Ready && self.waiting_for.take().is_some() {
            self.interrupted += 1;
        }
        self.phases.push((at, phase));
    }

    fn observe(&mut self, settings: HeadsetSettings) {
        if self.original.is_none() {
            self.original = Some(settings.clone());
        }
        if self.waiting_for.as_ref() == Some(&settings) {
            self.waiting_for = None;
            self.confirmed += 1;
        }
        self.observed = Some(settings);
    }

    pub fn toggle_request(&mut self) -> Option<HeadsetSettings> {
        if !self.is_ready() || self.waiting_for.is_some() {
            return None;
        }
        let desired = toggled(self.original.as_ref()?, self.observed.as_ref()?);
        Some(self.request(desired))
    }

    pub fn restore_request(&mut self) -> Option<HeadsetSettings> {
        let original = self.original.clone()?;
        if self.observed.as_ref() == Some(&original) {
            return None;
        }
        if !self.is_ready() {
            self.unrestored = true;
            return None;
        }
        Some(self.request(original))
    }

    fn request(&mut self, desired: HeadsetSettings) -> HeadsetSettings {
        self.requested += 1;
        self.waiting_for = Some(desired.clone());
        desired
    }

    fn is_ready(&self) -> bool {
        self.phase() == Some(Phase::Ready)
    }

    pub fn phase(&self) -> Option<Phase> {
        self.phases.last().map(|(_, phase)| *phase)
    }

    pub fn disconnects(&self) -> Vec<(Duration, Phase)> {
        let connected_at = self
            .phases
            .iter()
            .position(|(_, phase)| matches!(phase, Phase::Ready | Phase::HeadsetOff));
        let Some(start) = connected_at else {
            return Vec::new();
        };
        self.phases[start..]
            .iter()
            .filter(|(_, phase)| !phase.is_present() || *phase == Phase::Rebinding)
            .copied()
            .collect()
    }
}

fn toggled(original: &HeadsetSettings, observed: &HeadsetSettings) -> HeadsetSettings {
    let level = original.sidetone.get();
    let other = if level == 0 { 1 } else { level - 1 };
    let next = if observed.sidetone.get() == level {
        other
    } else {
        level
    };
    HeadsetSettings {
        sidetone: Sidetone::try_from(next).expect("a sidetone level in range"),
        ..observed.clone()
    }
}
