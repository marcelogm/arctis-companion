use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use arctis_app::ports::{
    Alarm, AudioPort, Clock, DevicePort, Lifetime, Notifier, SettingsStore, SnapshotView,
};
use arctis_core::battery::sample::{Sample, UnixSeconds};
use arctis_core::companion::Snapshot;
use arctis_core::companion::alerts::{Alert, AlertId};
use arctis_core::companion::app_settings::PersistedChange;
use arctis_core::companion::seen_apps::SeenApp;
use arctis_core::eq::preset::{Preset, PresetId};
use arctis_core::messages::audio::AudioCommand;
use arctis_core::messages::device::Change;
use arctis_core::ports::{BatteryRepository, PresetRepository, SeenAppsRepository, StoreError};
use arctis_core::time::Millis;

#[derive(Clone, Debug, PartialEq)]
pub enum Seen {
    Persisted(PersistedChange),
    SavedPreset(PresetId),
    DeletedPreset(PresetId),
    Battery(Sample),
    AppRecorded(String),
    Device(Change),
    Audio(AudioCommand),
    Published(Rc<Snapshot>),
    Notified(AlertId),
    Withdrawn(AlertId),
    Scheduled(Duration),
    ReadyToQuit,
}

pub type Journal = Rc<RefCell<Vec<Seen>>>;

#[derive(Clone, Default)]
pub struct Recorder(pub Journal);

impl Recorder {
    fn push(&self, seen: Seen) {
        self.0.borrow_mut().push(seen);
    }
}

impl SettingsStore for Recorder {
    fn persist(&self, change: &PersistedChange) {
        self.push(Seen::Persisted(change.clone()));
    }
}

impl DevicePort for Recorder {
    fn send(&self, change: Change) {
        self.push(Seen::Device(change));
    }
}

impl AudioPort for Recorder {
    fn send(&self, command: AudioCommand) {
        self.push(Seen::Audio(command));
    }
}

impl SnapshotView for Recorder {
    fn publish(&self, snapshot: &Rc<Snapshot>) {
        self.push(Seen::Published(Rc::clone(snapshot)));
    }
}

impl Notifier for Recorder {
    fn notify(&self, alert: &Alert) {
        self.push(Seen::Notified(alert.id));
    }

    fn withdraw(&self, id: AlertId) {
        self.push(Seen::Withdrawn(id));
    }
}

impl Lifetime for Recorder {
    fn ready_to_quit(&self) {
        self.push(Seen::ReadyToQuit);
    }
}

impl PresetRepository for Recorder {
    fn load(&self) -> Result<Vec<Preset>, StoreError> {
        Ok(Vec::new())
    }

    fn save(&mut self, preset: &Preset) -> Result<(), StoreError> {
        self.push(Seen::SavedPreset(preset.id().clone()));
        Ok(())
    }

    fn delete(&mut self, id: &PresetId) -> Result<(), StoreError> {
        self.push(Seen::DeletedPreset(id.clone()));
        Ok(())
    }
}

impl BatteryRepository for Recorder {
    fn load(&self) -> Result<Vec<Sample>, StoreError> {
        Ok(Vec::new())
    }

    fn append(&mut self, sample: &Sample) -> Result<(), StoreError> {
        self.push(Seen::Battery(*sample));
        Ok(())
    }

    fn retain_since(&mut self, _cutoff: UnixSeconds) -> Result<(), StoreError> {
        Ok(())
    }
}

impl SeenAppsRepository for Recorder {
    fn load(&self) -> Result<Vec<SeenApp>, StoreError> {
        Ok(Vec::new())
    }

    fn record(&mut self, app: &SeenApp) -> Result<(), StoreError> {
        self.push(Seen::AppRecorded(app.key.clone()));
        Ok(())
    }
}

type Wake = Box<dyn FnOnce()>;

#[derive(Clone)]
pub struct ManualAlarm {
    journal: Journal,
    pending: Rc<RefCell<Option<Wake>>>,
}

impl ManualAlarm {
    pub fn new(journal: Journal) -> Self {
        Self {
            journal,
            pending: Rc::default(),
        }
    }

    pub fn fire(&self) {
        let wake = self.pending.borrow_mut().take();
        if let Some(wake) = wake {
            wake();
        }
    }
}

impl Alarm for ManualAlarm {
    fn schedule(&self, delay: Duration, wake: Wake) {
        self.journal.borrow_mut().push(Seen::Scheduled(delay));
        self.pending.replace(Some(wake));
    }
}

#[derive(Clone)]
pub struct ManualClock(Rc<Cell<Millis>>);

impl ManualClock {
    pub fn starting_at(start: Millis) -> Self {
        Self(Rc::new(Cell::new(start)))
    }

    pub fn advance(&self, milliseconds: u64) {
        self.0.set(self.0.get().after(milliseconds));
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Millis {
        self.0.get()
    }
}
