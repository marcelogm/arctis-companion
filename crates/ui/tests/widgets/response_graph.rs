use adw::prelude::*;
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::{id, snapshot};
use arctis_core::view::eq_editor::{GraphModel, eq_editor};
use arctis_ui::ResponseGraph;
use gtk::gsk;
use gtk::{AccessibleProperty, AccessibleRole};

use crate::support::{has_accessible_text, init};

const GRID_LINES: usize = 8;

fn graph_of(preset: &str) -> GraphModel {
    eq_editor(&snapshot(), &id(preset)).expect("editor").graph
}

fn strokes(node: &gsk::RenderNode) -> usize {
    if node.downcast_ref::<gsk::StrokeNode>().is_some() {
        return 1;
    }
    node.downcast_ref::<gsk::ContainerNode>()
        .map_or(0, |container| {
            (0..container.n_children())
                .map(|at| strokes(&container.child(at)))
                .sum()
        })
}

fn drawn_strokes(graph: &ResponseGraph) -> usize {
    let snapshot = gtk::Snapshot::new();
    graph.draw(&snapshot, 360, 160);
    strokes(&snapshot.to_node().expect("something was drawn"))
}

#[gtk::test]
fn the_graph_is_an_image_described_by_its_model() {
    init();
    let graph = ResponseGraph::default();
    let model = graph_of("bass-boost");
    graph.render(&model);
    assert_eq!(graph.accessible_role(), AccessibleRole::Img);
    assert!(has_accessible_text(
        &graph,
        AccessibleProperty::Description,
        &english(&model.description)
    ));
}

#[gtk::test]
fn a_graph_without_points_draws_only_its_grid() {
    init();
    assert_eq!(drawn_strokes(&ResponseGraph::default()), GRID_LINES);
}

#[gtk::test]
fn the_curve_is_drawn_over_the_grid() {
    init();
    let graph = ResponseGraph::default();
    graph.render(&graph_of("bass-boost"));
    assert_eq!(drawn_strokes(&graph), GRID_LINES + 1);
}

#[gtk::test]
fn a_zero_sized_graph_draws_without_panicking() {
    init();
    let graph = ResponseGraph::default();
    graph.render(&graph_of("bass-boost"));
    graph.draw(&gtk::Snapshot::new(), 0, 0);
}
