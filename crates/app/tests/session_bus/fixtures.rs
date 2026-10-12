use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use arctis_app::alarm::{GlibAlarm, SystemClock};
use arctis_app::bridge::Bridge;
use arctis_app::ports::{AudioPort, DevicePort, Lifetime, Notifier, Ports, SettingsStore, Stores};
use arctis_app::threads::Threads;
use arctis_core::battery::sample::{Sample, UnixSeconds};
use arctis_core::companion::app_settings::{
    AppRules, AppSettings, Persisted, PersistedChange, RoutePresets,
};
use arctis_core::companion::seen_apps::SeenApp;
use arctis_core::companion::{Companion, Input};
use arctis_core::eq::preset::{Preset, PresetId};
use arctis_core::intent::Intent;
use arctis_core::messages::audio::AudioCommand;
use arctis_core::messages::audio::AudioEvent;
use arctis_core::messages::device::{Change, DeviceUpdate};
use arctis_core::ports::{BatteryRepository, PresetRepository, SeenAppsRepository, StoreError};
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::testkit::snapshots::{id, percent, values};

pub const PATH: &str = "/io/github/marcelogm/ArctisCompanion";
pub const INTERFACE: &str = "io.github.marcelogm.ArctisCompanion.Headset1";

pub fn contract_xml() -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/dbus")
        .join(format!("{INTERFACE}.xml"));
    fs::read_to_string(file).expect("contract file")
}

pub type Intents = Rc<RefCell<Vec<Intent>>>;

pub fn recorder(log: &Intents) -> impl Fn(Intent) + 'static {
    let log = Rc::clone(log);
    move |intent| log.borrow_mut().push(intent)
}

pub struct Quiet;

impl SettingsStore for Quiet {
    fn persist(&self, _change: &PersistedChange) {}
}

impl DevicePort for Quiet {
    fn send(&self, _change: Change) {}
}

impl AudioPort for Quiet {
    fn send(&self, _command: AudioCommand) {}
}

impl Lifetime for Quiet {
    fn ready_to_quit(&self) {}
}

impl PresetRepository for Quiet {
    fn load(&self) -> Result<Vec<Preset>, StoreError> {
        Ok(Vec::new())
    }

    fn save(&mut self, _preset: &Preset) -> Result<(), StoreError> {
        Ok(())
    }

    fn delete(&mut self, _id: &PresetId) -> Result<(), StoreError> {
        Ok(())
    }
}

impl BatteryRepository for Quiet {
    fn load(&self) -> Result<Vec<Sample>, StoreError> {
        Ok(Vec::new())
    }

    fn append(&mut self, _sample: &Sample) -> Result<(), StoreError> {
        Ok(())
    }

    fn retain_since(&mut self, _cutoff: UnixSeconds) -> Result<(), StoreError> {
        Ok(())
    }
}

impl SeenAppsRepository for Quiet {
    fn load(&self) -> Result<Vec<SeenApp>, StoreError> {
        Ok(Vec::new())
    }

    fn record(&mut self, _app: &SeenApp) -> Result<(), StoreError> {
        Ok(())
    }
}

pub fn persisted() -> Persisted {
    Persisted {
        settings: AppSettings {
            headset: values(),
            route_presets: RoutePresets {
                dongle: id("flat"),
                bluetooth: id("flat"),
            },
            app_rules: AppRules::default(),
            automatic_presets: true,
            adopted_identity: None,
            bluetooth_sink_seen: false,
            low_battery_threshold: percent(15),
        },
        custom_presets: Vec::new(),
        seen_apps: Vec::new(),
        battery: Vec::new(),
    }
}

pub fn quiet_bridge(notifier: Box<dyn Notifier>) -> Rc<Bridge> {
    let ports = Ports {
        settings: Box::new(Quiet),
        device: Box::new(Quiet),
        audio: Box::new(Quiet),
        notifier,
        alarm: Box::<GlibAlarm>::default(),
        clock: Box::new(SystemClock),
        lifetime: Box::new(Quiet),
    };
    let stores = Stores {
        presets: Box::new(Quiet),
        battery: Box::new(Quiet),
        seen_apps: Box::new(Quiet),
    };
    Bridge::new(Companion::new(persisted()), ports, stores)
}

pub fn status(level: u8, charging: Charging) -> Input {
    Input::Device(DeviceUpdate::Status {
        battery: percent(level),
        charging,
        link: Link::Connected,
    })
}

pub struct Silent<T> {
    _open: async_channel::Sender<T>,
}

impl DevicePort for Silent<DeviceUpdate> {
    fn send(&self, _change: Change) {}
}

impl AudioPort for Silent<AudioEvent> {
    fn send(&self, _command: AudioCommand) {}
}

pub fn silent_threads() -> Threads {
    let (device_open, device_updates) = async_channel::unbounded();
    let (audio_open, audio_events) = async_channel::unbounded();
    Threads {
        device: Box::new(Silent { _open: device_open }),
        device_updates,
        audio: Box::new(Silent { _open: audio_open }),
        audio_events,
    }
}
