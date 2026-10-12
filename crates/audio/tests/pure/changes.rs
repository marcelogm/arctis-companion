use arctis_audio::changes::Published;
use arctis_core::messages::audio::{AudioEvent, DefaultSink, SinkKind};

use crate::support::{BLUEZ, SPEAKERS, Scene};

fn default_sink(terminal: SinkKind) -> AudioEvent {
    AudioEvent::DefaultSink(DefaultSink {
        terminal,
        via_equalizer: false,
        via_easy_effects: false,
    })
}

#[test]
fn the_first_look_publishes_everything() {
    let graph = Scene::default().bluez(4).default_sink(BLUEZ).build();
    assert_eq!(
        Published::default().changes(&graph),
        [
            AudioEvent::BluetoothNodeSeen,
            default_sink(SinkKind::NovaBluetooth),
            AudioEvent::Streams(Vec::new()),
        ]
    );
}

#[test]
fn an_unchanged_graph_publishes_nothing() {
    let graph = Scene::default().bluez(4).default_sink(BLUEZ).build();
    let mut published = Published::default();
    published.changes(&graph);
    assert_eq!(published.changes(&graph), []);
}

#[test]
fn only_what_changed_is_published() {
    let mut published = Published::default();
    published.changes(
        &Scene::default()
            .bluez(4)
            .speakers(5)
            .default_sink(BLUEZ)
            .build(),
    );
    let moved = Scene::default()
        .bluez(4)
        .speakers(5)
        .default_sink(SPEAKERS)
        .build();
    assert_eq!(published.changes(&moved), [default_sink(SinkKind::Other)]);
    let playing = Scene::default()
        .bluez(4)
        .speakers(5)
        .default_sink(SPEAKERS)
        .stream(11, &[("application.id", "a")])
        .build();
    let events = published.changes(&playing);
    assert!(matches!(events.as_slice(), [AudioEvent::Streams(streams)] if streams.len() == 1));
}

#[test]
fn the_bluetooth_node_is_announced_once() {
    let mut published = Published::default();
    published.changes(&Scene::default().bluez(4).build());
    published.changes(&Scene::default().build());
    let again = published.changes(&Scene::default().bluez(4).build());
    assert!(!again.contains(&AudioEvent::BluetoothNodeSeen));
}

#[test]
fn no_bluetooth_node_no_announcement() {
    let events = Published::default().changes(&Scene::default().speakers(5).build());
    assert!(!events.contains(&AudioEvent::BluetoothNodeSeen));
}
