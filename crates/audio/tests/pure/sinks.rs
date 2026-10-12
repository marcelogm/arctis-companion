use arctis_audio::eq_names::EQ_SINK_NAME;
use arctis_audio::keys::{CONFIGURED_AUDIO_SINK, DEFAULT_AUDIO_SINK};
use arctis_audio::sinks::{default_sink, has_nova_bluetooth, sink_kind};
use arctis_core::messages::audio::{DefaultSink, SinkKind};

use crate::support::{BLUEZ, SPEAKERS, Scene, USB, with_equalizer};

#[test]
fn sinks_are_classified_by_their_props_and_device() {
    let cases = [
        (Scene::default().bluez(1), SinkKind::NovaBluetooth),
        (Scene::default().usb(1, 3, "0x2232"), SinkKind::NovaDongle),
        (Scene::default().usb(1, 3, "0x2253"), SinkKind::NovaDongle),
        (Scene::default().usb(1, 3, "0x2233"), SinkKind::Other),
        (Scene::default().usb(1, 3, "0x9999"), SinkKind::Other),
        (Scene::default().sink(1, USB), SinkKind::Other),
        (Scene::default().speakers(1), SinkKind::Other),
        (
            Scene::default().node(
                1,
                &[
                    ("media.class", "Audio/Source"),
                    ("node.description", "Arctis Nova 5"),
                    ("device.api", "bluez5"),
                ],
            ),
            SinkKind::Other,
        ),
        (
            Scene::default().node(
                1,
                &[
                    ("media.class", "Audio/Sink"),
                    ("node.description", "Other Headphones"),
                    ("device.api", "bluez5"),
                ],
            ),
            SinkKind::Other,
        ),
    ];
    for (index, (scene, expected)) in cases.into_iter().enumerate() {
        assert_eq!(sink_kind(&scene.build(), 1), expected, "case {index}");
    }
}

#[test]
fn a_vendor_from_another_maker_is_not_the_dongle() {
    let mut graph = Scene::default().usb(1, 3, "0x2232").build();
    graph.put_device(
        3,
        crate::support::props(&[
            ("device.vendor.id", "0x046d"),
            ("device.product.id", "0x2232"),
        ]),
    );
    assert_eq!(sink_kind(&graph, 1), SinkKind::Other);
}

fn sink(terminal: SinkKind, via_equalizer: bool, via_easy_effects: bool) -> DefaultSink {
    DefaultSink {
        terminal,
        via_equalizer,
        via_easy_effects,
    }
}

#[test]
fn the_default_sink_is_described_by_its_terminal() {
    let cases = [
        (
            Scene::default().bluez(4).default_sink(BLUEZ),
            sink(SinkKind::NovaBluetooth, false, false),
        ),
        (
            with_equalizer().default_sink(EQ_SINK_NAME),
            sink(SinkKind::NovaBluetooth, true, false),
        ),
        (
            Scene::default().usb(7, 3, "0x2232").default_sink(USB),
            sink(SinkKind::NovaDongle, false, false),
        ),
        (
            Scene::default().speakers(5).default_sink(SPEAKERS),
            sink(SinkKind::Other, false, false),
        ),
        (
            Scene::default()
                .usb(7, 3, "0x2232")
                .sink(40, "easyeffects_sink")
                .node(41, &[("node.name", "ee_soe_output_level")])
                .link(40, 41)
                .link(41, 7)
                .default_sink("easyeffects_sink"),
            sink(SinkKind::NovaDongle, false, true),
        ),
        (
            Scene::default().bluez(4),
            sink(SinkKind::Other, false, false),
        ),
    ];
    for (index, (scene, expected)) in cases.into_iter().enumerate() {
        assert_eq!(default_sink(&scene.build()), Some(expected), "case {index}");
    }
}

#[test]
fn a_default_naming_a_node_not_in_the_graph_is_held_back() {
    let vanished = Scene::default().bluez(4).default_sink("vanished");
    let not_yet_bound = Scene::default()
        .bluez(4)
        .metadata(CONFIGURED_AUDIO_SINK, BLUEZ)
        .metadata(DEFAULT_AUDIO_SINK, EQ_SINK_NAME);
    for (index, scene) in [vanished, not_yet_bound].into_iter().enumerate() {
        assert_eq!(default_sink(&scene.build()), None, "case {index}");
    }
}

#[test]
fn a_configured_default_alone_naming_a_missing_node_is_other() {
    let graph = Scene::default()
        .speakers(5)
        .metadata(CONFIGURED_AUDIO_SINK, "unplugged")
        .build();
    assert_eq!(
        default_sink(&graph),
        Some(sink(SinkKind::Other, false, false))
    );
}

#[test]
fn a_default_that_no_longer_reaches_bluez_through_our_sink_is_other() {
    let user_moved_while_inserted = with_equalizer().speakers(5).default_sink(SPEAKERS);
    let bluez_vanished_while_inserted = Scene::default()
        .equalizer(30, 31)
        .default_sink(EQ_SINK_NAME);
    for (index, scene) in [user_moved_while_inserted, bluez_vanished_while_inserted]
        .into_iter()
        .enumerate()
    {
        assert_eq!(
            default_sink(&scene.build()),
            Some(sink(SinkKind::Other, false, false)),
            "case {index}"
        );
    }
}

#[test]
fn the_configured_default_stands_in_while_no_effective_default_exists() {
    let graph = Scene::default()
        .bluez(4)
        .metadata(CONFIGURED_AUDIO_SINK, BLUEZ)
        .build();
    assert_eq!(
        default_sink(&graph).map(|sink| sink.terminal),
        Some(SinkKind::NovaBluetooth)
    );
}

#[test]
fn a_nova_bluetooth_sink_anywhere_counts_as_seen() {
    assert!(has_nova_bluetooth(&Scene::default().bluez(4).build()));
    assert!(!has_nova_bluetooth(&Scene::default().speakers(5).build()));
}
