use crate::battery::sample::{Sample, UnixSeconds};
use crate::companion::seen_apps::{SeenApp, remember};
use crate::eq::preset::{Preset, PresetId};
use crate::ports::{BatteryRepository, PresetRepository, SeenAppsRepository, StoreError};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemoryPresets {
    presets: Vec<Preset>,
}

impl PresetRepository for MemoryPresets {
    fn load(&self) -> Result<Vec<Preset>, StoreError> {
        Ok(self.presets.clone())
    }

    fn save(&mut self, preset: &Preset) -> Result<(), StoreError> {
        self.presets.retain(|known| known.id() != preset.id());
        self.presets.push(preset.clone());
        Ok(())
    }

    fn delete(&mut self, id: &PresetId) -> Result<(), StoreError> {
        self.presets.retain(|known| known.id() != id);
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemoryBattery {
    samples: Vec<Sample>,
}

impl BatteryRepository for MemoryBattery {
    fn load(&self) -> Result<Vec<Sample>, StoreError> {
        Ok(self.samples.clone())
    }

    fn append(&mut self, sample: &Sample) -> Result<(), StoreError> {
        self.samples.push(*sample);
        Ok(())
    }

    fn retain_since(&mut self, cutoff: UnixSeconds) -> Result<(), StoreError> {
        self.samples.retain(|sample| sample.at >= cutoff);
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemorySeenApps {
    apps: Vec<SeenApp>,
}

impl SeenAppsRepository for MemorySeenApps {
    fn load(&self) -> Result<Vec<SeenApp>, StoreError> {
        Ok(self.apps.clone())
    }

    fn record(&mut self, app: &SeenApp) -> Result<(), StoreError> {
        self.apps = remember(&self.apps, app.clone());
        Ok(())
    }
}
