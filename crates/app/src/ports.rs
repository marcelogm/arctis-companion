use arctis_core::companion::app_settings::PersistedChange;
use arctis_core::ports::{BatteryRepository, PresetRepository, SeenAppsRepository};

pub trait SettingsStore {
    fn persist(&self, change: &PersistedChange);
}

pub struct Stores {
    pub presets: Box<dyn PresetRepository>,
    pub battery: Box<dyn BatteryRepository>,
    pub seen_apps: Box<dyn SeenAppsRepository>,
}
