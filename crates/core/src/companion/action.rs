use std::rc::Rc;

use super::alerts::{Alert, AlertChange, AlertId};
use super::app_settings::PersistedChange;
use super::seen_apps::SeenApp;
use super::snapshot::Snapshot;
use crate::battery::sample::Sample;
use crate::eq::preset::{Preset, PresetId};
use crate::messages::audio::AudioCommand;
use crate::messages::device::Change;
use crate::time::Millis;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PresetChange {
    Save(Preset),
    Delete(PresetId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Persist(PersistedChange),
    StorePreset(PresetChange),
    RecordBattery(Sample),
    RecordSeenApp(SeenApp),
    SendDevice(Change),
    SendAudio(AudioCommand),
    Publish(Rc<Snapshot>),
    Notify(Alert),
    Withdraw(AlertId),
    WakeAt(Millis),
    ReadyToQuit,
}

impl From<AlertChange> for Action {
    fn from(change: AlertChange) -> Self {
        match change {
            AlertChange::Notify(alert) => Self::Notify(alert),
            AlertChange::Withdraw(id) => Self::Withdraw(id),
        }
    }
}
