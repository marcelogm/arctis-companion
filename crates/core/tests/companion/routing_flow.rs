use arctis_core::companion::Action;
use arctis_core::companion::app_settings::PersistedChange;
use arctis_core::companion::routing::{AudioService, Route};
use arctis_core::messages::audio::{AudioCommand, AudioEvent, SinkKind, SoundPath};

use crate::support::{
    Harness, adopted, audio_commands, curve_of, default_sink, id, observed, persisted,
    persisted_changes, sink, stream,
};

#[test]
fn the_equalizer_replaces_the_bluez_default_with_the_bluetooth_curve() {
    let mut saved = persisted();
    saved.settings.route_presets.bluetooth = id("podcast");
    let mut h = Harness::new(saved);
    let actions = h.audio(default_sink(SinkKind::NovaBluetooth));
    assert_eq!(
        audio_commands(&actions),
        [AudioCommand::InsertEqualizer(curve_of("podcast"))]
    );
    assert_eq!(h.snapshot().audio.route, Route::Bluetooth);
}

#[test]
fn the_equalizer_is_inserted_once() {
    let mut h = Harness::new(persisted());
    let _ = h.audio(default_sink(SinkKind::NovaBluetooth));
    let behind = h.audio(AudioEvent::DefaultSink(sink(
        SinkKind::NovaBluetooth,
        true,
        false,
    )));
    assert!(audio_commands(&behind).is_empty());
    assert!(audio_commands(&h.audio(default_sink(SinkKind::NovaBluetooth))).is_empty());
}

#[test]
fn leaving_bluetooth_removes_the_equalizer() {
    let mut h = Harness::new(persisted());
    let _ = h.audio(default_sink(SinkKind::NovaBluetooth));
    let actions = h.audio(default_sink(SinkKind::NovaDongle));
    assert_eq!(audio_commands(&actions), [AudioCommand::RemoveEqualizer]);
    assert_eq!(h.snapshot().audio.route, Route::Dongle);
}

#[test]
fn easy_effects_keeps_the_equalizer_out() {
    let mut h = Harness::new(persisted());
    let actions = h.audio(AudioEvent::DefaultSink(sink(
        SinkKind::NovaBluetooth,
        false,
        true,
    )));
    assert!(audio_commands(&actions).is_empty());
    assert_eq!(h.snapshot().audio.route, Route::Bluetooth);
}

#[test]
fn easy_effects_appearing_takes_the_equalizer_out() {
    let mut h = Harness::new(persisted());
    let _ = h.audio(default_sink(SinkKind::NovaBluetooth));
    let actions = h.audio(AudioEvent::DefaultSink(sink(
        SinkKind::NovaBluetooth,
        true,
        true,
    )));
    assert_eq!(audio_commands(&actions), [AudioCommand::RemoveEqualizer]);
    assert_eq!(h.snapshot().audio.route, Route::Bluetooth);
}

#[test]
fn speakers_are_no_headset_route() {
    let mut h = Harness::new(persisted());
    assert!(audio_commands(&h.audio(default_sink(SinkKind::Other))).is_empty());
    assert_eq!(h.snapshot().audio.route, Route::None);
}

#[test]
fn pipewire_down_keeps_the_dongle_path() {
    let mut h = Harness::new(adopted());
    let _ = h.connect(observed());
    let _ = h.audio(default_sink(SinkKind::NovaDongle));
    let _ = h.audio(AudioEvent::Unavailable);
    let snapshot = h.snapshot();
    assert_eq!(snapshot.audio.route, Route::None);
    assert_eq!(snapshot.audio.service, AudioService::Unavailable);
    assert_eq!(snapshot.picked_path(), Ok(SoundPath::Dongle));
}

#[test]
fn pipewire_down_drops_the_equalizer_without_a_command() {
    let mut h = Harness::new(persisted());
    let _ = h.audio(default_sink(SinkKind::NovaBluetooth));
    assert!(audio_commands(&h.audio(AudioEvent::Unavailable)).is_empty());
    let back = h.audio(default_sink(SinkKind::NovaBluetooth));
    assert_eq!(
        audio_commands(&back),
        [AudioCommand::InsertEqualizer(curve_of("flat"))]
    );
}

#[test]
fn a_bluez_node_seen_once_is_remembered() {
    let mut h = Harness::new(persisted());
    let first = h.audio(AudioEvent::BluetoothNodeSeen);
    assert_eq!(
        persisted_changes(&first),
        [PersistedChange::BluetoothSinkSeen]
    );
    assert!(persisted_changes(&h.audio(AudioEvent::BluetoothNodeSeen)).is_empty());
    assert!(h.snapshot().audio.bluetooth_sink_seen);
}

#[test]
fn a_denied_manager_role_is_published() {
    let mut h = Harness::new(persisted());
    let _ = h.audio(AudioEvent::ManagerDenied);
    assert!(h.snapshot().audio.manager_denied);
}

#[test]
fn an_app_that_starts_playing_is_recorded_once() {
    let mut h = Harness::new(persisted());
    let streams = AudioEvent::Streams(vec![stream("firefox", None, true)]);
    let first = h.audio(streams.clone());
    assert_eq!(
        first
            .iter()
            .filter(|a| matches!(a, Action::RecordSeenApp(_)))
            .count(),
        1
    );
    let again = h.audio(streams);
    assert!(!again.iter().any(|a| matches!(a, Action::RecordSeenApp(_))));
    assert_eq!(h.snapshot().presets.seen_apps.len(), 1);
}
