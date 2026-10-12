use arctis_audio::changes::{Published, Settling};
use arctis_audio::eq_names::EQ_SINK_NAME;
use arctis_audio::graph::Graph;
use arctis_audio::keys::{CONFIGURED_AUDIO_SINK, DEFAULT_AUDIO_SINK};
use arctis_core::messages::audio::{AudioEvent, DefaultSink, SinkKind};

use crate::support::{BLUEZ, SPEAKERS, Scene, with_equalizer};

fn default_sink(terminal: SinkKind) -> AudioEvent {
    route(terminal, false)
}

fn route(terminal: SinkKind, via_equalizer: bool) -> AudioEvent {
    AudioEvent::DefaultSink(DefaultSink {
        terminal,
        via_equalizer,
        via_easy_effects: false,
    })
}

fn default_sinks(events: Vec<AudioEvent>) -> Vec<AudioEvent> {
    events
        .into_iter()
        .filter(|event| matches!(event, AudioEvent::DefaultSink(_)))
        .collect()
}

fn on_bluez() -> Graph {
    Scene::default().bluez(4).default_sink(BLUEZ).build()
}

fn equalizer_unlinked() -> Graph {
    Scene::default()
        .bluez(4)
        .equalizer(30, 31)
        .default_sink(EQ_SINK_NAME)
        .build()
}

fn equalizer_without_output() -> Graph {
    Scene::default()
        .bluez(4)
        .sink(30, EQ_SINK_NAME)
        .default_sink(EQ_SINK_NAME)
        .build()
}

fn equalizer_not_yet_bound() -> Graph {
    Scene::default()
        .bluez(4)
        .metadata(CONFIGURED_AUDIO_SINK, EQ_SINK_NAME)
        .metadata(DEFAULT_AUDIO_SINK, EQ_SINK_NAME)
        .build()
}

fn on_equalizer() -> Graph {
    with_equalizer().default_sink(EQ_SINK_NAME).build()
}

fn published_on(graph: &Graph) -> Published {
    let mut published = Published::default();
    published.changes(graph);
    published
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

#[test]
fn a_default_naming_an_absent_node_keeps_the_last_route() {
    let mut published = published_on(&on_bluez());
    assert_eq!(
        default_sinks(published.changes(&equalizer_not_yet_bound())),
        []
    );
    assert_eq!(
        default_sinks(published.changes(&on_equalizer())),
        [route(SinkKind::NovaBluetooth, true)]
    );
}

#[test]
fn an_insertion_publishes_only_the_route_through_the_equalizer() {
    let mut published = published_on(&on_bluez());
    published.settle(Settling::Insertion);
    let seen: Vec<AudioEvent> = [
        equalizer_not_yet_bound(),
        equalizer_unlinked(),
        on_equalizer(),
    ]
    .iter()
    .flat_map(|graph| default_sinks(published.changes(graph)))
    .collect();
    assert_eq!(seen, [route(SinkKind::NovaBluetooth, true)]);
}

#[test]
fn a_removal_publishes_only_the_restored_route() {
    let mut published = published_on(&on_equalizer());
    published.settle(Settling::Removal);
    let seen: Vec<AudioEvent> = [
        equalizer_unlinked(),
        equalizer_without_output(),
        equalizer_not_yet_bound(),
        on_bluez(),
    ]
    .iter()
    .flat_map(|graph| default_sinks(published.changes(graph)))
    .collect();
    assert_eq!(seen, [default_sink(SinkKind::NovaBluetooth)]);
}

#[test]
fn a_removal_settles_on_a_default_the_user_chose() {
    let mut published = published_on(&on_equalizer());
    published.settle(Settling::Removal);
    let user_moved = Scene::default().speakers(5).default_sink(SPEAKERS).build();
    assert_eq!(
        default_sinks(published.changes(&user_moved)),
        [default_sink(SinkKind::Other)]
    );
}

#[test]
fn streams_are_published_while_settling() {
    let mut published = published_on(&on_bluez());
    published.settle(Settling::Insertion);
    let playing = Scene::default()
        .bluez(4)
        .equalizer(30, 31)
        .default_sink(EQ_SINK_NAME)
        .stream(11, &[("application.id", "a")])
        .build();
    let events = published.changes(&playing);
    assert!(matches!(events.as_slice(), [AudioEvent::Streams(streams)] if streams.len() == 1));
}

#[test]
fn stopping_settling_publishes_whatever_the_graph_holds() {
    let mut published = published_on(&on_bluez());
    let ticket = published.settle(Settling::Insertion);
    assert_eq!(default_sinks(published.changes(&equalizer_unlinked())), []);
    published.stop_settling(ticket);
    assert_eq!(
        default_sinks(published.changes(&equalizer_without_output())),
        [route(SinkKind::Other, true)]
    );
}

#[test]
fn an_earlier_settling_deadline_does_not_end_a_later_one() {
    let mut published = published_on(&on_bluez());
    let first = published.settle(Settling::Insertion);
    published.settle(Settling::Removal);
    published.stop_settling(first);
    assert_eq!(default_sinks(published.changes(&equalizer_unlinked())), []);
}
