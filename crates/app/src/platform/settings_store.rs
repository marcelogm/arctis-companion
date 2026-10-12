use arctis_core::companion::app_settings::AppSettings;
use gtk::gio;
use gtk::gio::prelude::*;

use super::settings_keys::{
    self as keys, AUTOMATIC_PRESETS, BLUETOOTH_SINK_SEEN, LOW_BATTERY_THRESHOLD,
};

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
