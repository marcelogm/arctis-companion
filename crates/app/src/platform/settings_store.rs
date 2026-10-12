use std::cell::Cell;
use std::rc::Rc;

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

pub struct SettingsWatch {
    settings: gio::Settings,
    handler: Option<glib::SignalHandlerId>,
}

impl Drop for SettingsWatch {
    fn drop(&mut self) {
        if let Some(handler) = self.handler.take() {
            self.settings.disconnect(handler);
        }
    }
}

impl GSettingsStore {
    #[must_use]
    pub fn watch(&self, on_change: impl Fn(AppSettings) + 'static) -> SettingsWatch {
        let coalesced = Coalesced {
            pending: Rc::new(Cell::new(false)),
            store: self.clone(),
            on_change: Rc::new(on_change),
        };
        let handler = self
            .settings
            .connect_changed(None, move |_, _| coalesced.schedule());
        SettingsWatch {
            settings: self.settings.clone(),
            handler: Some(handler),
        }
    }
}

#[derive(Clone)]
struct Coalesced {
    pending: Rc<Cell<bool>>,
    store: GSettingsStore,
    on_change: Rc<dyn Fn(AppSettings)>,
}

impl Coalesced {
    fn schedule(&self) {
        if self.pending.replace(true) {
            return;
        }
        let coalesced = self.clone();
        glib::spawn_future_local(async move { coalesced.deliver() });
    }

    fn deliver(&self) {
        self.pending.set(false);
        (self.on_change)(self.store.load());
    }
}
