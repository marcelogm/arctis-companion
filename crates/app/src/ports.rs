use arctis_core::companion::app_settings::PersistedChange;

pub trait SettingsStore {
    fn persist(&self, change: &PersistedChange);
}
