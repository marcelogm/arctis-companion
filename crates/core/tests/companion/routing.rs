use arctis_core::companion::routing::{AudioService, AudioState, Route, needs_equalizer, route};
use arctis_core::messages::audio::{AudioEvent, SinkKind, SoundPath};

use crate::support::sink;

#[test]
fn the_route_follows_the_terminal_sink_of_the_default() {
    let cases = [
        (
            "bluez node",
            Some(sink(SinkKind::NovaBluetooth, false, false)),
            Route::Bluetooth,
        ),
        (
            "behind our EQ sink",
            Some(sink(SinkKind::NovaBluetooth, true, false)),
            Route::Bluetooth,
        ),
        (
            "USB sink",
            Some(sink(SinkKind::NovaDongle, false, false)),
            Route::Dongle,
        ),
        (
            "EasyEffects in between",
            Some(sink(SinkKind::NovaBluetooth, false, true)),
            Route::Bluetooth,
        ),
        (
            "speakers",
            Some(sink(SinkKind::Other, false, false)),
            Route::None,
        ),
        ("no default sink", None, Route::None),
    ];
    for (case, default_sink, expected) in cases {
        assert_eq!(route(default_sink), expected, "{case}");
    }
}

#[test]
fn the_equalizer_replaces_only_the_bluez_node_itself() {
    let cases = [
        (sink(SinkKind::NovaBluetooth, false, false), true),
        (sink(SinkKind::NovaBluetooth, true, false), false),
        (sink(SinkKind::NovaBluetooth, false, true), false),
        (sink(SinkKind::NovaDongle, false, false), false),
        (sink(SinkKind::Other, false, false), false),
    ];
    for (default_sink, expected) in cases {
        assert_eq!(needs_equalizer(default_sink), expected, "{default_sink:?}");
    }
}

#[test]
fn routes_name_their_paths() {
    assert_eq!(Route::Dongle.path(), Some(SoundPath::Dongle));
    assert_eq!(Route::Bluetooth.path(), Some(SoundPath::Bluetooth));
    assert_eq!(Route::None.path(), None);
}

#[test]
fn pipewire_down_forgets_the_default_sink() {
    let mut audio = AudioState::new();
    audio.apply(&AudioEvent::DefaultSink(sink(
        SinkKind::NovaDongle,
        false,
        false,
    )));
    audio.apply(&AudioEvent::ManagerDenied);
    audio.apply(&AudioEvent::Unavailable);
    assert_eq!(audio.route(), Route::None);
    assert_eq!(audio.service, AudioService::Unavailable);
    assert!(!audio.manager_denied);
}

#[test]
fn audio_events_update_the_state() {
    let mut audio = AudioState::new();
    assert_eq!(audio.service, AudioService::Unknown);
    audio.apply(&AudioEvent::Available);
    audio.apply(&AudioEvent::ManagerDenied);
    audio.apply(&AudioEvent::DefaultSink(sink(
        SinkKind::NovaBluetooth,
        false,
        true,
    )));
    assert_eq!(audio.service, AudioService::Available);
    assert!(audio.manager_denied);
    assert!(audio.easy_effects_active());
    assert_eq!(audio.route(), Route::Bluetooth);
}

#[test]
fn the_default_audio_state_is_a_fresh_one() {
    assert_eq!(AudioState::default(), AudioState::new());
}
