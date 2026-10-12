use arctis_audio::eq_names::EQ_OUTPUT_NAME;
use arctis_audio::streams::{identifiers, streams};
use arctis_core::messages::audio::{AppIdentifiers, SoundPath, StreamInfo};

use crate::support::{FilterNodes, Scene, props};

#[test]
fn pulse_streams_carry_their_identifiers_on_the_node() {
    let node = props(&[
        ("pipewire.access.portal.app_id", "org.mozilla.firefox"),
        ("application.process.binary", "Firefox"),
    ]);
    assert_eq!(
        identifiers(&node, None),
        AppIdentifiers::new(Some("org.mozilla.firefox"), None, Some("firefox"))
    );
}

#[test]
fn native_streams_take_the_portal_app_id_from_their_client() {
    let node = props(&[
        ("application.id", "com.example.Native"),
        ("application.process.binary", "native"),
    ]);
    let client = props(&[(
        "pipewire.access.portal.app_id",
        "com.example.Native.Flatpak",
    )]);
    assert_eq!(
        identifiers(&node, Some(&client)).as_slice(),
        ["com.example.Native.Flatpak", "com.example.Native", "native"]
    );
}

#[test]
fn the_node_value_wins_over_the_client_value() {
    let node = props(&[("application.id", "from.node")]);
    let client = props(&[("application.id", "from.client")]);
    assert_eq!(identifiers(&node, Some(&client)).key(), Some("from.node"));
}

fn bluetooth_scene() -> Scene {
    Scene::default()
        .bluez(4)
        .equalizer(30, 31)
        .link(31, 4)
        .usb(7, 3, "0x2232")
        .speakers(5)
}

fn info(ids: AppIdentifiers, name: &str, running: bool, path: Option<SoundPath>) -> StreamInfo {
    StreamInfo {
        ids,
        name: name.to_owned(),
        running,
        path,
    }
}

#[test]
fn each_stream_reports_the_path_its_chain_ends_on() {
    let graph = bluetooth_scene()
        .stream(
            11,
            &[("application.id", "eq.app"), ("application.name", "Eq App")],
        )
        .link(11, 30)
        .running(11)
        .stream(12, &[("application.id", "usb.app")])
        .link(12, 7)
        .stream(13, &[("application.id", "speaker.app")])
        .link(13, 5)
        .stream(14, &[("application.id", "bluez.app")])
        .link(14, 4)
        .stream(15, &[("application.id", "idle.app")])
        .build();
    let id = |name| AppIdentifiers::new(None, Some(name), None);
    assert_eq!(
        streams(&graph),
        [
            info(id("eq.app"), "Eq App", true, Some(SoundPath::Bluetooth)),
            info(id("usb.app"), "usb.app", false, Some(SoundPath::Dongle)),
            info(id("speaker.app"), "speaker.app", false, None),
            info(id("bluez.app"), "bluez.app", false, None),
            info(id("idle.app"), "idle.app", false, None),
        ]
    );
}

#[test]
fn a_stream_behind_an_intermediate_filter_reaches_the_equalizer() {
    let graph = bluetooth_scene()
        .filter(&FilterNodes {
            sink: 20,
            output: 21,
            name: "chain_filter",
            output_name: "chain_filter_out",
            group: "loopback-1",
        })
        .link(21, 30)
        .stream(11, &[("application.process.binary", "player")])
        .link(11, 20)
        .build();
    let found = streams(&graph);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, Some(SoundPath::Bluetooth));
}

#[test]
fn display_names_fall_back_from_node_to_client_to_key() {
    let graph = Scene::default()
        .client(9, &[("application.name", "Client Name")])
        .stream(
            11,
            &[("application.id", "a"), ("application.name", "Node Name")],
        )
        .stream(12, &[("application.id", "b"), ("client.id", "9")])
        .stream(13, &[("application.id", "c")])
        .build();
    let names: Vec<String> = streams(&graph).into_iter().map(|info| info.name).collect();
    assert_eq!(names, ["Node Name", "Client Name", "c"]);
}

#[test]
fn filter_outputs_and_anonymous_streams_are_not_apps() {
    let graph = bluetooth_scene()
        .stream(11, &[("media.name", "no identifiers")])
        .stream(
            12,
            &[
                ("node.name", EQ_OUTPUT_NAME),
                ("application.process.binary", "arctis-companion"),
            ],
        )
        .build();
    assert_eq!(streams(&graph), []);
}
