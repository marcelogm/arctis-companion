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
    confirmations: Vec<Duration>,
    pending: Option<Pending>,
    pub unrestored: bool,
}

#[derive(Debug)]
struct Pending {
    desired: HeadsetSettings,
    since: Duration,
}

impl SoakLog {
    pub fn record(&mut self, at: Duration, update: DeviceUpdate) {
        match update {
            DeviceUpdate::Phase(phase) => self.enter(at, phase),
            DeviceUpdate::Observed(settings) => self.observe(at, settings),
            DeviceUpdate::WriteFailed(_) => self.failed += 1,
            _ => {}
        }
    }

    fn enter(&mut self, at: Duration, phase: Phase) {
        if phase != Phase::Ready && self.pending.take().is_some() {
            self.interrupted += 1;
        }
        self.phases.push((at, phase));
    }

    fn observe(&mut self, at: Duration, settings: HeadsetSettings) {
        if self.original.is_none() {
            self.original = Some(settings.clone());
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.desired == settings)
        {
            self.confirm(at);
        }
        self.observed = Some(settings);
    }

    fn confirm(&mut self, at: Duration) {
        if let Some(pending) = self.pending.take() {
            self.confirmed += 1;
            self.confirmations.push(at.saturating_sub(pending.since));
        }
    }

    pub fn slowest_confirmation(&self) -> Option<Duration> {
        self.confirmations.iter().max().copied()
    }

    pub fn confirmations_within(&self, limit: Duration) -> bool {
        self.slowest_confirmation()
            .is_none_or(|slowest| slowest <= limit)
    }

    pub fn toggle_request(&mut self, at: Duration) -> Option<HeadsetSettings> {
        if !self.is_ready() || self.pending.is_some() {
            return None;
        }
        let desired = toggled(self.original.as_ref()?, self.observed.as_ref()?);
        Some(self.request(at, desired))
    }

    pub fn restore_request(&mut self, at: Duration) -> Option<HeadsetSettings> {
        let original = self.original.clone()?;
        if self.observed.as_ref() == Some(&original) {
            return None;
        }
        if !self.is_ready() {
            self.unrestored = true;
            return None;
        }
        Some(self.request(at, original))
    }

    fn request(&mut self, at: Duration, desired: HeadsetSettings) -> HeadsetSettings {
        self.requested += 1;
        self.pending = Some(Pending {
            desired: desired.clone(),
            since: at,
        });
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
