use std::error::Error;
use std::fmt;

use crate::battery::sample::{Sample, UnixSeconds};
use crate::companion::seen_apps::SeenApp;
use crate::eq::preset::{Preset, PresetId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreError(pub String);

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for StoreError {}

pub trait PresetRepository {
    fn load(&self) -> Result<Vec<Preset>, StoreError>;
    fn save(&mut self, preset: &Preset) -> Result<(), StoreError>;
    fn delete(&mut self, id: &PresetId) -> Result<(), StoreError>;
}

pub trait BatteryRepository {
    fn load(&self) -> Result<Vec<Sample>, StoreError>;
    fn append(&mut self, sample: &Sample) -> Result<(), StoreError>;
    fn retain_since(&mut self, cutoff: UnixSeconds) -> Result<(), StoreError>;
}

pub trait SeenAppsRepository {
    fn load(&self) -> Result<Vec<SeenApp>, StoreError>;
    fn record(&mut self, app: &SeenApp) -> Result<(), StoreError>;
}
