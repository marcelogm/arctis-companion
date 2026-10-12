use std::rc::Rc;

use arctis_core::battery::sample::{Sample, UnixSeconds};
use arctis_core::companion::alerts::{Alert, AlertId};
use arctis_core::companion::app_settings::{AppSettings, PersistedChange};
use arctis_core::companion::seen_apps::SeenApp;
use arctis_core::companion::{Action, Companion, Input, PresetChange, Snapshot};
use arctis_core::intent::Intent;
use arctis_core::messages::audio::{AudioCommand, AudioEvent};
use arctis_core::messages::device::{Change, DeviceUpdate, Phase, SyncPolicy};
use arctis_core::time::Millis;
use arctis_core::view::text::Text;

use crate::support::{START, discharging, id, observed, persisted, settings};

const fn has_type<T>(_: &T) {}

fn input_shape(input: &Input) {
    match input {
        Input::Device(update) => has_type::<DeviceUpdate>(update),
        Input::Audio(event) => has_type::<AudioEvent>(event),
        Input::Intent(intent) => has_type::<Intent>(intent),
        Input::SettingsChanged(settings) => has_type::<AppSettings>(settings),
        Input::Woke | Input::Quitting => {}
    }
}

fn action_shape(action: &Action) {
    match action {
        Action::Persist(change) => has_type::<PersistedChange>(change),
        Action::StorePreset(change) => has_type::<PresetChange>(change),
        Action::RecordBattery(sample) => has_type::<Sample>(sample),
        Action::RecordSeenApp(app) => has_type::<SeenApp>(app),
        Action::SendDevice(change) => has_type::<Change>(change),
        Action::SendAudio(command) => has_type::<AudioCommand>(command),
        Action::Publish(snapshot) => has_type::<Rc<Snapshot>>(snapshot),
        Action::Notify(alert) => has_type::<Alert>(alert),
        Action::Withdraw(id) => has_type::<AlertId>(id),
        Action::WakeAt(at) => has_type::<Millis>(at),
        Action::ReadyToQuit => {}
    }
}

#[test]
fn every_input_of_the_contract_exists() {
    let inputs = [
        Input::Device(DeviceUpdate::Phase(Phase::Ready)),
        Input::Audio(AudioEvent::Available),
        Input::Intent(Intent::EndPreview),
        Input::Woke,
        Input::SettingsChanged(settings()),
        Input::Quitting,
    ];
    inputs.iter().for_each(input_shape);
    assert_eq!(inputs.len(), 6);
}

#[test]
fn every_action_of_the_contract_exists() {
    let alert = Alert {
        id: AlertId::LowBattery,
        title: Text::plain("Headset Battery Low"),
        body: Text::plain("Low"),
    };
    let seen = SeenApp {
        key: "firefox".to_owned(),
        name: "Firefox".to_owned(),
        identifiers: vec!["firefox".to_owned()],
    };
    let actions = [
        Action::Persist(PersistedChange::BluetoothSinkSeen),
        Action::StorePreset(PresetChange::Delete(id("mine"))),
        Action::RecordBattery(Sample {
            at: UnixSeconds(1),
            reading: discharging(50),
        }),
        Action::RecordSeenApp(seen),
        Action::SendDevice(Change::Desired {
            settings: observed(),
            policy: SyncPolicy::Adopt,
        }),
        Action::SendAudio(AudioCommand::RemoveEqualizer),
        Action::Publish(Companion::new(persisted()).snapshot()),
        Action::Notify(alert),
        Action::Withdraw(AlertId::CriticalBattery),
        Action::WakeAt(START),
        Action::ReadyToQuit,
    ];
    actions.iter().for_each(action_shape);
    assert_eq!(actions.len(), 11);
}
