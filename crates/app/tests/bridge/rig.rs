use std::rc::Rc;

use arctis_app::bridge::Bridge;
use arctis_app::ports::{Ports, Stores};
use arctis_core::companion::app_settings::{AppRules, AppSettings, Persisted, RoutePresets};
use arctis_core::companion::{Companion, Input, Snapshot};
use arctis_core::eq::curve::Curve;
use arctis_core::eq::library;
use arctis_core::eq::preset::{Preset, PresetId};
use arctis_core::messages::audio::{AppIdentifiers, SoundPath, StreamInfo};
use arctis_core::ports::PresetRepository;
use arctis_core::testkit::snapshots::{id, percent, values};
use arctis_core::time::Millis;

use crate::fakes::{Journal, ManualAlarm, ManualClock, Recorder, Seen};

pub const START: Millis = Millis(1_760_000_000_000);

pub struct Rig {
    pub bridge: Rc<Bridge>,
    pub journal: Journal,
    pub alarm: ManualAlarm,
    pub clock: ManualClock,
}

impl Rig {
    pub fn new(persisted: Persisted) -> Self {
        let journal = Journal::default();
        Self::assemble(persisted, Box::new(Recorder(Rc::clone(&journal))), journal)
    }

    fn assemble(
        persisted: Persisted,
        presets: Box<dyn PresetRepository>,
        journal: Journal,
    ) -> Self {
        let recorder = Recorder(Rc::clone(&journal));
        let alarm = ManualAlarm::new(Rc::clone(&journal));
        let clock = ManualClock::starting_at(START);
        let ports = Ports {
            settings: Box::new(recorder.clone()),
            device: Box::new(recorder.clone()),
            audio: Box::new(recorder.clone()),
            notifier: Box::new(recorder.clone()),
            alarm: Box::new(alarm.clone()),
            clock: Box::new(clock.clone()),
            lifetime: Box::new(recorder.clone()),
        };
        let stores = Stores {
            presets,
            battery: Box::new(recorder.clone()),
            seen_apps: Box::new(recorder.clone()),
        };
        let bridge = Bridge::new(Companion::new(persisted), ports, stores);
        bridge.attach(Rc::new(recorder));
        journal.borrow_mut().clear();
        Self {
            bridge,
            journal,
            alarm,
            clock,
        }
    }

    pub fn send(&self, input: Input) -> Vec<Seen> {
        self.bridge.submit(input);
        self.take()
    }

    pub fn take(&self) -> Vec<Seen> {
        self.journal.borrow_mut().drain(..).collect()
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

pub fn with_rule(app: &str, preset: &str) -> Persisted {
    let mut saved = persisted();
    saved.settings.app_rules.set(app.to_owned(), id(preset));
    saved
}

pub fn curve(preset: &str) -> Curve {
    library::find(&PresetId::new(preset))
        .expect("built-in preset")
        .curve()
        .clone()
}

pub fn mine() -> Preset {
    Preset::custom(id("mine"), "Mine".to_owned(), curve("bass-boost"))
}

pub fn firefox(path: Option<SoundPath>) -> StreamInfo {
    StreamInfo {
        ids: AppIdentifiers::new(Some("org.mozilla.firefox"), None, Some("firefox")),
        name: "Firefox".to_owned(),
        running: true,
        path,
    }
}

pub fn published(seen: &[Seen]) -> Option<Rc<Snapshot>> {
    seen.iter().rev().find_map(|item| match item {
        Seen::Published(snapshot) => Some(Rc::clone(snapshot)),
        _ => None,
    })
}
