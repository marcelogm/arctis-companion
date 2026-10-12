use arctis_audio::graph::{CONFIGURED_AUDIO_SINK, Graph};
use arctis_audio::outputs::{
    EQ_SINK_NAME, InsertStep, Outputs, RemoveStep, default_sink, has_nova_bluetooth,
    insertion_target, sink_kind,
};
use arctis_core::messages::audio::{DefaultSink, SinkKind};

use crate::support::{BLUEZ, SPEAKERS, Scene, USB};

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

fn with_equalizer() -> Scene {
    Scene::default().bluez(4).equalizer(30, 31).link(31, 4)
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
        (
            Scene::default().default_sink("vanished"),
            sink(SinkKind::Other, false, false),
        ),
    ];
    for (index, (scene, expected)) in cases.into_iter().enumerate() {
        assert_eq!(default_sink(&scene.build()), expected, "case {index}");
    }
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
            sink(SinkKind::Other, false, false),
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
    assert_eq!(default_sink(&graph).terminal, SinkKind::NovaBluetooth);
}

#[test]
fn a_nova_bluetooth_sink_anywhere_counts_as_seen() {
    assert!(has_nova_bluetooth(&Scene::default().bluez(4).build()));
    assert!(!has_nova_bluetooth(&Scene::default().speakers(5).build()));
}

#[test]
fn only_a_default_that_is_the_bluez_node_itself_is_replaced() {
    let cases: [(Graph, Option<&str>); 4] = [
        (
            Scene::default().bluez(4).default_sink(BLUEZ).build(),
            Some(BLUEZ),
        ),
        (with_equalizer().default_sink(EQ_SINK_NAME).build(), None),
        (
            Scene::default()
                .usb(7, 3, "0x2232")
                .default_sink(USB)
                .build(),
            None,
        ),
        (
            Scene::default()
                .bluez(4)
                .sink(40, "ee_sie_sink")
                .node(41, &[("node.name", "ee_soe_output_level")])
                .link(40, 41)
                .link(41, 4)
                .default_sink("ee_sie_sink")
                .build(),
            None,
        ),
    ];
    for (index, (graph, expected)) in cases.into_iter().enumerate() {
        assert_eq!(
            insertion_target(&graph).as_deref(),
            expected,
            "case {index}"
        );
    }
}

#[test]
fn inserting_creates_once_then_retargets() {
    let graph = Scene::default().bluez(4).default_sink(BLUEZ).build();
    let mut outputs = Outputs::default();
    assert_eq!(
        outputs.insert(&graph),
        InsertStep::Create {
            replaced: BLUEZ.to_owned()
        }
    );
    assert!(outputs.is_inserted());
    assert_eq!(outputs.insert(&graph), InsertStep::Retarget);
}

#[test]
fn inserting_over_another_default_is_skipped() {
    let graph = Scene::default().speakers(5).default_sink(SPEAKERS).build();
    let mut outputs = Outputs::default();
    assert_eq!(outputs.insert(&graph), InsertStep::Skip);
    assert!(!outputs.is_inserted());
}

fn inserted() -> Outputs {
    let mut outputs = Outputs::default();
    outputs.insert(&Scene::default().bluez(4).default_sink(BLUEZ).build());
    outputs
}

#[test]
fn removal_restores_the_bluez_node_while_our_sink_is_the_default() {
    let graph = with_equalizer().default_sink(EQ_SINK_NAME).build();
    let mut outputs = inserted();
    assert_eq!(
        outputs.remove(&graph),
        RemoveStep::Restore(BLUEZ.to_owned())
    );
    assert_eq!(outputs.remove(&graph), RemoveStep::NothingInserted);
}

#[test]
fn removal_leaves_a_default_the_user_changed_alone() {
    let graph = with_equalizer().speakers(5).default_sink(SPEAKERS).build();
    assert_eq!(inserted().remove(&graph), RemoveStep::LeaveDefault);
}

#[test]
fn removal_without_insertion_does_nothing() {
    let graph = with_equalizer().default_sink(EQ_SINK_NAME).build();
    assert_eq!(
        Outputs::default().remove(&graph),
        RemoveStep::NothingInserted
    );
}

#[test]
fn a_write_that_left_the_bluez_default_in_place_was_denied() {
    let unchanged = with_equalizer().default_sink(BLUEZ).build();
    let granted = with_equalizer().default_sink(EQ_SINK_NAME).build();
    let user_moved = with_equalizer().speakers(5).default_sink(SPEAKERS).build();
    assert!(inserted().manager_denied(&unchanged));
    assert!(!inserted().manager_denied(&granted));
    assert!(!inserted().manager_denied(&user_moved));
    assert!(!Outputs::default().manager_denied(&unchanged));
}
