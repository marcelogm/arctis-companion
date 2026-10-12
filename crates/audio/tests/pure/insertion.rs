use arctis_audio::eq_names::EQ_SINK_NAME;
use arctis_audio::graph::Graph;
use arctis_audio::insertion::{InsertStep, Insertion, RemoveStep, insertion_target};

use crate::support::{BLUEZ, SPEAKERS, Scene, USB, with_equalizer};

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
fn inserting_creates_once_committed_then_retargets() {
    let graph = Scene::default().bluez(4).default_sink(BLUEZ).build();
    let mut insertion = Insertion::default();
    assert_eq!(
        insertion.plan_insert(&graph),
        InsertStep::Create {
            replaced: BLUEZ.to_owned()
        }
    );
    insertion.commit(BLUEZ.to_owned());
    assert!(insertion.is_inserted());
    assert_eq!(insertion.plan_insert(&graph), InsertStep::Retarget);
}

#[test]
fn a_creation_that_failed_leaves_nothing_inserted() {
    let graph = Scene::default().bluez(4).default_sink(BLUEZ).build();
    let insertion = Insertion::default();
    let _planned = insertion.plan_insert(&graph);
    assert!(!insertion.is_inserted());
    assert!(matches!(
        insertion.plan_insert(&graph),
        InsertStep::Create { .. }
    ));
}

#[test]
fn inserting_over_another_default_is_skipped() {
    let graph = Scene::default().speakers(5).default_sink(SPEAKERS).build();
    let insertion = Insertion::default();
    assert_eq!(insertion.plan_insert(&graph), InsertStep::Skip);
    assert!(!insertion.is_inserted());
}

fn inserted() -> Insertion {
    let mut insertion = Insertion::default();
    insertion.commit(BLUEZ.to_owned());
    insertion
}

#[test]
fn removal_restores_the_bluez_node_while_our_sink_is_the_default() {
    let graph = with_equalizer().default_sink(EQ_SINK_NAME).build();
    let mut insertion = inserted();
    assert_eq!(
        insertion.remove(&graph),
        RemoveStep::Restore(BLUEZ.to_owned())
    );
    assert_eq!(insertion.remove(&graph), RemoveStep::NothingInserted);
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
        Insertion::default().remove(&graph),
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
    assert!(!Insertion::default().manager_denied(&unchanged));
}
