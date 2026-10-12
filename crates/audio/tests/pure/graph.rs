use arctis_audio::graph::{Defaults, Graph, Link};
use arctis_audio::keys::{CONFIGURED_AUDIO_SINK, DEFAULT_AUDIO_SINK};
use arctis_audio::metadata_value::{json_for_name, name_from_json};

use crate::support::{BLUEZ, SPEAKERS, Scene, props, quoted};

#[test]
fn props_answer_by_key() {
    let props = props(&[("node.name", "a"), ("media.class", "Audio/Sink")]);
    assert_eq!(props.get("node.name"), Some("a"));
    assert_eq!(props.get("node.nick"), None);
}

#[test]
fn a_node_keeps_its_running_state_when_its_props_change() {
    let mut graph = Scene::default().speakers(5).running(5).build();
    graph.put_node(5, props(&[("node.name", "renamed")]));
    let node = graph.node(5).expect("node 5");
    assert!(node.running);
    assert_eq!(node.name(), Some("renamed"));
}

#[test]
fn removing_a_node_or_a_link_forgets_it() {
    let mut graph = Scene::default().usb(7, 3, "0x2232").link(7, 8).build();
    for id in [7, 1000] {
        graph.remove(id);
    }
    assert_eq!(graph.nodes().count(), 0);
    assert_eq!(graph.inputs_fed_by(7).count(), 0);
}

#[test]
fn removing_a_device_or_a_client_forgets_it() {
    let mut graph = Scene::default()
        .usb(7, 3, "0x2232")
        .client(9, &[("application.name", "x")])
        .stream(11, &[("client.id", "9")])
        .build();
    graph.remove(3);
    graph.remove(9);
    let usb = graph.node(7).expect("usb sink");
    let stream = graph.node(11).expect("stream");
    assert_eq!(graph.device_of(usb), None);
    assert_eq!(graph.client_of(stream), None);
}

#[test]
fn removing_the_metadata_that_set_the_defaults_clears_them() {
    let mut graph = Scene::default().default_sink(BLUEZ).build();
    graph.take_defaults_from(40);
    graph.remove(41);
    assert_eq!(graph.defaults().current(), Some(BLUEZ));
    graph.remove(40);
    assert_eq!(graph.defaults(), &Defaults::default());
}

#[test]
fn metadata_values_name_a_node() {
    let cases = [
        (r#"{"name":"bluez_output.1"}"#, Some("bluez_output.1")),
        (r#"{ "name": "spaced" }"#, Some("spaced")),
        (r#"{"name":"with \"quote\""}"#, Some("with \"quote\"")),
        (r#"{"other":"x"}"#, None),
        (r#"{"name":"#, None),
        (r#"{"name":"unterminated"#, None),
    ];
    for (value, expected) in cases {
        assert_eq!(name_from_json(value).as_deref(), expected, "{value}");
    }
}

#[test]
fn written_names_read_back() {
    for name in [BLUEZ, "a\"b", "back\\slash"] {
        assert_eq!(name_from_json(&json_for_name(name)).as_deref(), Some(name));
    }
}

#[test]
fn defaults_follow_their_metadata_keys() {
    let mut graph = Graph::default();
    graph.set_default(DEFAULT_AUDIO_SINK, Some(&quoted(BLUEZ)));
    graph.set_default(CONFIGURED_AUDIO_SINK, Some(&quoted(SPEAKERS)));
    graph.set_default("default.audio.source", Some(&quoted("mic")));
    assert_eq!(
        graph.defaults(),
        &Defaults {
            audio_sink: Some(BLUEZ.to_owned()),
            configured_audio_sink: Some(SPEAKERS.to_owned()),
        }
    );
    graph.set_default(DEFAULT_AUDIO_SINK, None);
    assert_eq!(graph.defaults().audio_sink, None);
    graph.clear_defaults();
    assert_eq!(graph.defaults(), &Defaults::default());
}

#[test]
fn the_current_default_prefers_the_effective_sink() {
    let both = Defaults {
        audio_sink: Some(BLUEZ.to_owned()),
        configured_audio_sink: Some(SPEAKERS.to_owned()),
    };
    let configured_only = Defaults {
        audio_sink: None,
        configured_audio_sink: Some(SPEAKERS.to_owned()),
    };
    assert_eq!(both.current(), Some(BLUEZ));
    assert_eq!(configured_only.current(), Some(SPEAKERS));
    assert!(both.mentions(SPEAKERS));
    assert!(!configured_only.mentions(BLUEZ));
}

#[test]
fn devices_and_clients_are_found_through_node_props() {
    let graph = Scene::default()
        .usb(7, 3, "0x2232")
        .client(9, &[("application.name", "Player")])
        .stream(11, &[("client.id", "9")])
        .build();
    let usb = graph.node(7).expect("usb sink");
    let stream = graph.node(11).expect("stream");
    assert_eq!(
        graph
            .device_of(usb)
            .and_then(|device| device.get("device.product.id")),
        Some("0x2232")
    );
    assert_eq!(
        graph
            .client_of(stream)
            .and_then(|client| client.get("application.name")),
        Some("Player")
    );
    assert_eq!(graph.client_of(usb), None);
}

#[test]
fn links_are_followed_from_their_output_side() {
    let mut graph = Scene::default().link(1, 2).link(1, 3).link(4, 1).build();
    graph.put_link(
        2000,
        Link {
            output: 1,
            input: 2,
        },
    );
    let mut inputs: Vec<u32> = graph.inputs_fed_by(1).collect();
    inputs.sort_unstable();
    assert_eq!(inputs, [2, 2, 3]);
}
