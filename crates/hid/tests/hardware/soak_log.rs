use std::time::Duration;

use arctis_core::messages::device::{DeviceUpdate, Phase};
use arctis_core::settings::headset_settings::HeadsetSettings;

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
