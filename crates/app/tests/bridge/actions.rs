use std::rc::Rc;

use arctis_core::companion::Input;
use arctis_core::companion::alerts::AlertId;
use arctis_core::companion::app_settings::PersistedChange;
use arctis_core::intent::Intent;
use arctis_core::messages::audio::{AudioCommand, AudioEvent, DefaultSink, SinkKind};
use arctis_core::messages::device::{Change, DeviceUpdate, Phase, SyncPolicy};
use arctis_core::protocol::firmware::Firmware;
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::testkit::snapshots::{id, nova_5, percent};

use crate::fakes::{Recorder, Seen};
use crate::rig::{Rig, curve, firefox, mine, persisted, published};

fn status(level: u8, charging: Charging) -> Input {
    Input::Device(DeviceUpdate::Status {
        battery: percent(level),
        charging,
        link: Link::Connected,
    })
}

#[test]
fn a_persisted_change_reaches_the_settings_store() {
    let rig = Rig::new(persisted());
    let seen = rig.send(Input::Intent(Intent::SetAutomaticPresets(false)));
    assert!(seen.contains(&Seen::Persisted(PersistedChange::AutomaticPresets(false))));
}

#[test]
fn a_saved_preset_reaches_the_preset_store() {
    let rig = Rig::new(persisted());
    let seen = rig.send(Input::Intent(Intent::SavePreset(mine())));
    assert!(seen.contains(&Seen::SavedPreset(id("mine"))));
}

#[test]
fn a_deleted_preset_reaches_the_preset_store() {
    let mut saved = persisted();
    saved.custom_presets = vec![mine()];
    let rig = Rig::new(saved);
    let seen = rig.send(Input::Intent(Intent::DeletePreset(id("mine"))));
    assert!(seen.contains(&Seen::DeletedPreset(id("mine"))));
}

#[test]
fn a_low_reading_is_recorded_and_notified() {
    let rig = Rig::new(persisted());
    let seen = rig.send(status(10, Charging::Discharging));
    assert!(
        seen.iter().any(
            |item| matches!(item, Seen::Battery(sample) if sample.reading.percent.get() == 10)
        )
    );
    assert!(seen.contains(&Seen::Notified(AlertId::LowBattery)));
}

#[test]
fn charging_withdraws_the_notification() {
    let rig = Rig::new(persisted());
    let _ = rig.send(status(10, Charging::Discharging));
    let seen = rig.send(status(10, Charging::Charging));
    assert!(seen.contains(&Seen::Withdrawn(AlertId::LowBattery)));
}

#[test]
fn an_app_seen_playing_is_recorded() {
    let rig = Rig::new(persisted());
    let seen = rig.send(Input::Audio(AudioEvent::Streams(vec![firefox(None)])));
    assert!(seen.contains(&Seen::AppRecorded("org.mozilla.firefox".to_owned())));
}

#[test]
fn an_identity_reaches_the_device_thread() {
    let rig = Rig::new(persisted());
    let seen = rig.send(Input::Device(DeviceUpdate::Identity {
        model: nova_5(),
        uniq: None,
        firmware: Firmware {
            receiver: None,
            headset: None,
        },
    }));
    assert!(seen.iter().any(|item| matches!(
        item,
        Seen::Device(Change::Desired {
            policy: SyncPolicy::Adopt,
            ..
        })
    )));
}

#[test]
fn a_bluetooth_default_reaches_the_audio_thread() {
    let rig = Rig::new(persisted());
    let sink = DefaultSink {
        terminal: SinkKind::NovaBluetooth,
        via_equalizer: false,
        via_easy_effects: false,
    };
    let seen = rig.send(Input::Audio(AudioEvent::DefaultSink(sink)));
    assert!(seen.contains(&Seen::Audio(AudioCommand::InsertEqualizer(curve("flat")))));
}

#[test]
fn every_changed_snapshot_is_published() {
    let rig = Rig::new(persisted());
    let seen = rig.send(Input::Device(DeviceUpdate::Phase(Phase::Settling)));
    let snapshot = published(&seen).expect("a published snapshot");
    assert_eq!(snapshot.dongle.phase, Phase::Settling);
}

#[test]
fn attaching_a_view_publishes_the_current_snapshot() {
    let rig = Rig::new(persisted());
    rig.bridge
        .attach(Rc::new(Recorder(Rc::clone(&rig.journal))));
    let seen = rig.take();
    assert_eq!(published(&seen), Some(rig.bridge.snapshot()));
}

#[test]
fn quitting_with_nothing_to_restore_is_ready_at_once() {
    let rig = Rig::new(persisted());
    assert!(rig.send(Input::Quitting).contains(&Seen::ReadyToQuit));
}
