use std::rc::Rc;
use std::time::Duration;

use arctis_core::companion::Snapshot;
use arctis_core::companion::alerts::{Alert, AlertId};
use arctis_core::companion::app_settings::PersistedChange;
use arctis_core::messages::audio::AudioCommand;
use arctis_core::messages::device::Change;
use arctis_core::ports::{BatteryRepository, PresetRepository, SeenAppsRepository};
use arctis_core::time::Millis;

pub trait SettingsStore {
    fn persist(&self, change: &PersistedChange);
}

pub trait DevicePort {
    fn send(&self, change: Change);
}

pub trait AudioPort {
    fn send(&self, command: AudioCommand);
}

pub trait SnapshotView {
    fn publish(&self, snapshot: &Rc<Snapshot>);
}

pub trait Notifier {
    fn notify(&self, alert: &Alert);
    fn withdraw(&self, id: AlertId);
}

pub trait Alarm {
    fn schedule(&self, delay: Duration, wake: Box<dyn FnOnce()>);
}

pub trait Clock {
    fn now(&self) -> Millis;
}

pub trait Lifetime {
    fn ready_to_quit(&self);
}

pub struct Ports {
    pub settings: Box<dyn SettingsStore>,
    pub device: Box<dyn DevicePort>,
    pub audio: Box<dyn AudioPort>,
    pub notifier: Box<dyn Notifier>,
    pub alarm: Box<dyn Alarm>,
    pub clock: Box<dyn Clock>,
    pub lifetime: Box<dyn Lifetime>,
}

pub struct Stores {
    pub presets: Box<dyn PresetRepository>,
    pub battery: Box<dyn BatteryRepository>,
    pub seen_apps: Box<dyn SeenAppsRepository>,
}
