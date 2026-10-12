use arctis_core::companion::app_settings::AppSettings;
use arctis_core::companion::app_settings::PersistedChange;
use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib;

use super::settings_keys::{
    self as keys, ADOPTED_IDENTITY, AUTOMATIC_PRESETS, BLUETOOTH_SINK_SEEN, LOW_BATTERY_THRESHOLD,
    route_key,
};
use crate::logging::warn;
use crate::ports::SettingsStore;

#[derive(Clone, Debug)]
pub struct GSettingsStore {
    settings: gio::Settings,
}

impl GSettingsStore {
    #[must_use]
    pub const fn new(settings: gio::Settings) -> Self {
        Self { settings }
    }

    #[must_use]
    pub fn load(&self) -> AppSettings {
        let settings = &self.settings;
        AppSettings {
            headset: keys::headset(settings),
            route_presets: keys::route_presets(settings),
            app_rules: keys::app_rules(settings),
            automatic_presets: settings.boolean(AUTOMATIC_PRESETS),
            adopted_identity: keys::adopted_identity(settings),
            bluetooth_sink_seen: settings.boolean(BLUETOOTH_SINK_SEEN),
            low_battery_threshold: keys::ranged(settings, LOW_BATTERY_THRESHOLD),
        }
    }
}

impl GSettingsStore {
    fn write(&self, change: &PersistedChange) -> Result<(), glib::BoolError> {
        let settings = &self.settings;
        match change {
            PersistedChange::Headset(values) => keys::write_headset(settings, *values),
            PersistedChange::RoutePreset(path, preset) => {
                settings.set_string(route_key(*path), preset.as_str())
            }
            PersistedChange::AppRule { identifier, preset } => {
                keys::write_app_rule(settings, identifier, preset.as_ref())
            }
            PersistedChange::AutomaticPresets(on) => settings.set_boolean(AUTOMATIC_PRESETS, *on),
            PersistedChange::AdoptedIdentity(identity) => {
                settings.set_string(ADOPTED_IDENTITY, &identity.to_setting())
            }
            PersistedChange::BluetoothSinkSeen => settings.set_boolean(BLUETOOTH_SINK_SEEN, true),
            PersistedChange::LowBatteryThreshold(threshold) => {
                settings.set_int(LOW_BATTERY_THRESHOLD, i32::from(threshold.get()))
            }
        }
    }
}

impl SettingsStore for GSettingsStore {
    fn persist(&self, change: &PersistedChange) {
        if let Err(error) = self.write(change) {
            warn(&format!("could not save {change:?}: {error}"));
        }
    }
}
