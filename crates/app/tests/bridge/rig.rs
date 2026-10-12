use std::rc::Rc;

use arctis_app::bridge::Bridge;
use arctis_app::ports::{Ports, Stores};
use arctis_core::companion::app_settings::Persisted;
use arctis_core::companion::{Companion, Input, Snapshot};
use arctis_core::eq::curve::Curve;
use arctis_core::eq::preset::Preset;
use arctis_core::messages::device::Change;
use arctis_core::ports::PresetRepository;
pub use arctis_core::testkit::companion::{curve, firefox, persisted, with_rule};
use arctis_core::testkit::snapshots::id;
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

    pub fn with_presets(persisted: Persisted, presets: Box<dyn PresetRepository>) -> Self {
        Self::assemble(persisted, presets, Journal::default())
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

pub fn mine() -> Preset {
    Preset::custom(id("mine"), "Mine".to_owned(), curve("bass-boost"))
}

pub fn published(seen: &[Seen]) -> Option<Rc<Snapshot>> {
    seen.iter().rev().find_map(|item| match item {
        Seen::Published(snapshot) => Some(Rc::clone(snapshot)),
        _ => None,
    })
}

pub fn device_curves(seen: &[Seen]) -> Vec<Curve> {
    seen.iter()
        .filter_map(|item| match item {
            Seen::Device(Change::Desired { settings, .. }) => Some(settings.equalizer.clone()),
            _ => None,
        })
        .collect()
}
