use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use arctis_core::eq::FilterKind;
use arctis_core::eq::preset::Preset;
use arctis_core::testkit::english::english;
use arctis_core::view::eq_graph::graph;
use arctis_core::view::preset_edits::BandEdit;
use arctis_ui::{EqGraph, GraphAction};
use gtk::glib::translate::IntoGlib;
use gtk::{AccessibleProperty, AccessibleRole, gdk, gsk};

use crate::support::{custom_preset, filter, has_accessible_text, init};

const WIDTH: i32 = 680;
const HEIGHT: i32 = 300;
const AXIS_LABELS: usize = 10 + 9;
const ZONE_LABELS: usize = 7;

type Actions = Rc<RefCell<Vec<GraphAction>>>;

#[track_caller]
fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}

fn two_bands() -> Preset {
    custom_preset(&[
        filter(FilterKind::Peaking, 250, 4, 1_000),
        filter(FilterKind::LowPass, 8_000, 0, 707),
    ])
}

fn shown(preset: &Preset, editable: bool, width: i32) -> (EqGraph, Actions) {
    init();
    let graph_widget = EqGraph::default();
    graph_widget.render(&graph(preset), editable);
    let actions = Actions::default();
    let log = Rc::clone(&actions);
    graph_widget.connect_action(move |action| log.borrow_mut().push(action));
    graph_widget.measure(gtk::Orientation::Horizontal, -1);
    graph_widget.allocate(width, HEIGHT, -1, None);
    (graph_widget, actions)
}

fn centre(graph_widget: &EqGraph, hz: f32, db: f32) -> (f32, f32) {
    let plot = graph_widget.plot();
    (plot.x_of(hz), plot.y_of(db))
}

fn nodes_of(node: &gsk::RenderNode, kind: gsk::RenderNodeType) -> usize {
    let own = usize::from(node.node_type() == kind);
    own + children(node)
        .iter()
        .map(|child| nodes_of(child, kind))
        .sum::<usize>()
}

fn children(node: &gsk::RenderNode) -> Vec<gsk::RenderNode> {
    if let Some(container) = node.downcast_ref::<gsk::ContainerNode>() {
        return (0..container.n_children())
            .map(|at| container.child(at))
            .collect();
    }
    if let Some(transform) = node.downcast_ref::<gsk::TransformNode>() {
        return vec![transform.child()];
    }
    node.downcast_ref::<gsk::ClipNode>()
        .map(|clip| vec![clip.child()])
        .unwrap_or_default()
}

fn drawn(graph_widget: &EqGraph, width: i32) -> gsk::RenderNode {
    let snapshot = gtk::Snapshot::new();
    graph_widget.draw(&snapshot, width, HEIGHT);
    snapshot.to_node().expect("something was drawn")
}

fn key(handle: &impl IsA<gtk::Widget>, key: gdk::Key) -> bool {
    let controller = handle
        .observe_controllers()
        .into_iter()
        .filter_map(Result::ok)
        .find_map(|object| object.downcast::<gtk::EventControllerKey>().ok())
        .expect("a key controller");
    controller.emit_by_name::<bool>(
        "key-pressed",
        &[&key.into_glib(), &0_u32, &gdk::ModifierType::empty()],
    )
}

#[gtk::test]
fn the_graph_is_a_described_group_of_one_named_handle_per_band() {
    let (graph_widget, _) = shown(&two_bands(), true, WIDTH);
    assert_eq!(graph_widget.accessible_role(), AccessibleRole::Group);
    let model = graph(&two_bands());
    assert!(has_accessible_text(
        &graph_widget,
        AccessibleProperty::Description,
        &english(&model.description)
    ));
    let handles = graph_widget.handles();
    assert_eq!(handles.len(), 2);
    assert!(handles.iter().all(WidgetExt::is_focusable));
    assert!(has_accessible_text(
        &handles[0],
        AccessibleProperty::Label,
        "Band 1, Peak, 250 Hz, +2.0 dB, Q 1.00"
    ));
}

#[gtk::test]
fn handles_sit_on_their_band_with_a_comfortable_hit_area() {
    let (graph_widget, _) = shown(&two_bands(), true, WIDTH);
    let handle = &graph_widget.handles()[0];
    let bounds = handle.compute_bounds(&graph_widget).expect("allocated");
    let (x, y) = centre(&graph_widget, 250.0, 2.0);
    near(
        f64::from(bounds.x() + bounds.width() / 2.0),
        f64::from(x),
        1e-3,
    );
    near(
        f64::from(bounds.y() + bounds.height() / 2.0),
        f64::from(y),
        1e-3,
    );
    assert!(
        bounds.width() > 23.9 && bounds.height() > 23.9,
        "{bounds:?}"
    );
}

#[gtk::test]
fn dragging_a_handle_previews_and_commits_on_release() {
    let (graph_widget, actions) = shown(&two_bands(), true, WIDTH);
    let (x, y) = centre(&graph_widget, 250.0, 2.0);
    graph_widget.press_at(x + 2.0, y - 1.0);
    let target = centre(&graph_widget, 500.0, 6.0);
    let offset = (target.0 - x, target.1 - y);
    graph_widget.drag_by(offset.0, offset.1);
    graph_widget.release_by(offset.0, offset.1);
    let log = actions.borrow();
    assert_eq!(log[0], GraphAction::Select(0));
    let (GraphAction::Preview(0, preview), GraphAction::Commit(0, commit)) = (log[1], log[2])
    else {
        panic!("expected a preview then a commit, got {log:?}");
    };
    assert_eq!(preview, commit);
    let BandEdit::Point { hz, db } = commit else {
        panic!("expected a point edit, got {commit:?}");
    };
    near(hz, 500.0, 1.0);
    near(db, 6.0, 0.01);
    assert_eq!(graph_widget.selected(), Some(0));
}

#[gtk::test]
fn a_press_away_from_the_handles_drags_nothing() {
    let (graph_widget, actions) = shown(&two_bands(), true, WIDTH);
    let (x, y) = centre(&graph_widget, 2_000.0, -8.0);
    graph_widget.press_at(x, y);
    graph_widget.drag_by(30.0, 30.0);
    graph_widget.release_by(30.0, 30.0);
    assert!(actions.borrow().is_empty());
}

#[gtk::test]
fn double_clicking_empty_space_adds_a_band_there() {
    let (graph_widget, actions) = shown(&two_bands(), true, WIDTH);
    let (x, y) = centre(&graph_widget, 1_000.0, -3.0);
    graph_widget.double_click_at(x, y);
    let (handle_x, handle_y) = centre(&graph_widget, 250.0, 2.0);
    graph_widget.double_click_at(handle_x, handle_y);
    let log = actions.borrow();
    assert_eq!(log.len(), 1);
    let GraphAction::Add { hz, db } = log[0] else {
        panic!("expected an added band, got {log:?}");
    };
    near(hz, 1_000.0, 1.0);
    near(db, -3.0, 0.01);
}

#[gtk::test]
fn scrolling_over_a_handle_changes_its_q() {
    let (graph_widget, actions) = shown(&two_bands(), true, WIDTH);
    let (x, y) = centre(&graph_widget, 250.0, 2.0);
    graph_widget.hover_at(x, y);
    assert!(graph_widget.scroll_by(1.0));
    graph_widget.hover_at(x, y + 60.0);
    assert!(!graph_widget.scroll_by(1.0));
    let log = actions.borrow();
    let [GraphAction::Commit(0, BandEdit::Q(q))] = log.as_slice() else {
        panic!("expected one Q edit, got {log:?}");
    };
    assert!(*q > 1.0);
}

#[gtk::test]
fn keys_on_a_focused_handle_nudge_or_remove_its_band() {
    let (graph_widget, actions) = shown(&two_bands(), true, WIDTH);
    let handle = graph_widget.handles()[0].clone();
    assert!(key(&handle, gdk::Key::Up));
    assert!(key(&handle, gdk::Key::Right));
    assert!(key(&handle, gdk::Key::Page_Up));
    assert!(key(&handle, gdk::Key::Delete));
    assert!(!key(&handle, gdk::Key::a));
    let log = actions.borrow();
    assert_eq!(log[0], GraphAction::Commit(0, BandEdit::Gain(2.5)));
    assert!(matches!(log[1], GraphAction::Commit(0, BandEdit::Frequency(hz)) if hz > 264.0));
    assert!(matches!(log[2], GraphAction::Commit(0, BandEdit::Q(q)) if q > 1.0));
    assert_eq!(log[3], GraphAction::Remove(0));
    let pass = graph_widget.handles()[1].clone();
    assert!(key(&pass, gdk::Key::Up));
    assert_eq!(log.len(), 4);
}

#[gtk::test]
fn read_only_handles_select_but_never_edit() {
    let (graph_widget, actions) = shown(&two_bands(), false, WIDTH);
    let (x, y) = centre(&graph_widget, 250.0, 2.0);
    graph_widget.press_at(x, y);
    graph_widget.drag_by(40.0, 40.0);
    graph_widget.release_by(40.0, 40.0);
    graph_widget.hover_at(x, y);
    assert!(!graph_widget.scroll_by(1.0));
    graph_widget.double_click_at(x + 200.0, y + 50.0);
    assert!(!key(&graph_widget.handles()[0], gdk::Key::Up));
    assert_eq!(*actions.borrow(), vec![GraphAction::Select(0)]);
}

#[gtk::test]
fn the_graph_labels_both_axes_and_the_reference_zones() {
    let (graph_widget, _) = shown(&two_bands(), true, WIDTH);
    let wide = drawn(&graph_widget, WIDTH);
    assert_eq!(
        nodes_of(&wide, gsk::RenderNodeType::TextNode),
        AXIS_LABELS + ZONE_LABELS
    );
    let narrow = drawn(&graph_widget, 200);
    assert_eq!(
        nodes_of(&narrow, gsk::RenderNodeType::TextNode),
        AXIS_LABELS
    );
}

#[gtk::test]
fn the_response_is_filled_and_stroked_over_the_grid() {
    let (graph_widget, _) = shown(&two_bands(), true, WIDTH);
    let node = drawn(&graph_widget, WIDTH);
    assert_eq!(nodes_of(&node, gsk::RenderNodeType::FillNode), 1);
    assert!(nodes_of(&node, gsk::RenderNodeType::StrokeNode) > 30);
}

#[gtk::test]
fn a_zero_sized_graph_draws_without_panicking() {
    let (graph_widget, _) = shown(&two_bands(), true, WIDTH);
    graph_widget.draw(&gtk::Snapshot::new(), 0, 0);
}
