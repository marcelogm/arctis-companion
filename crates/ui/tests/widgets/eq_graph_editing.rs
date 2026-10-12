use adw::prelude::*;
use arctis_core::eq::FilterKind;
use arctis_core::intent::Intent;
use arctis_core::testkit::snapshots::snapshot;
use arctis_ui::{EqEditorPage, EqGraph};

use crate::support::{
    Mounted, band_rows, filter, named, one_peak, saved, standalone_editor, with_preset,
};

fn allocated(editor: &Mounted<EqEditorPage>) -> EqGraph {
    editor.page.measure(gtk::Orientation::Horizontal, -1);
    editor.page.allocate(720, 900, -1, None);
    named::<EqGraph>(&editor.page, "eq-graph")
}

fn handle_at(graph: &EqGraph, hz: f32, db: f32) -> (f32, f32) {
    let plot = graph.plot();
    (plot.x_of(hz), plot.y_of(db))
}

fn saves(editor: &Mounted<EqEditorPage>) -> usize {
    editor
        .log
        .borrow()
        .iter()
        .filter(|intent| matches!(intent, Intent::SavePreset(_)))
        .count()
}

#[gtk::test]
fn dragging_a_handle_moves_the_row_live_and_saves_once_on_release() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    let graph = allocated(&editor);
    let (x, y) = handle_at(&graph, 250.0, 2.0);
    graph.press_at(x, y);
    let (to_x, to_y) = handle_at(&graph, 2_000.0, 6.0);
    graph.drag_by((to_x - x) / 2.0, (to_y - y) / 2.0);
    graph.drag_by(to_x - x, to_y - y);
    assert_eq!(saves(&editor), 0);
    let row = &band_rows(&editor.page)[0];
    assert!(row.title().starts_with("2000 Hz"), "{}", row.title());
    graph.release_by(to_x - x, to_y - y);
    assert_eq!(saves(&editor), 1);
    assert_eq!(
        saved(&editor.log).curve().filters()[0],
        filter(FilterKind::Peaking, 2_000, 12, 1_000)
    );
}

#[gtk::test]
fn picking_a_handle_expands_its_row_and_expanding_a_row_selects_its_handle() {
    let two = arctis_core::testkit::snapshots::custom_preset(&[
        filter(FilterKind::Peaking, 250, 4, 1_000),
        filter(FilterKind::Peaking, 4_000, -2, 1_000),
    ]);
    let editor = standalone_editor(with_preset(&two), "mine");
    let graph = allocated(&editor);
    let (x, y) = handle_at(&graph, 4_000.0, -1.0);
    graph.press_at(x, y);
    let rows = band_rows(&editor.page);
    assert!(rows[1].is_expanded() && !rows[0].is_expanded());
    rows[0].set_expanded(true);
    assert_eq!(graph.selected(), Some(0));
    assert!(editor.log.borrow().is_empty());
}

#[gtk::test]
fn double_clicking_the_graph_adds_a_band_and_selects_it() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    let graph = allocated(&editor);
    let (x, y) = handle_at(&graph, 5_000.0, -4.0);
    graph.double_click_at(x, y);
    let added = saved(&editor.log);
    assert_eq!(added.curve().filters().len(), 2);
    assert_eq!(added.curve().filters()[1].hz().get(), 5_000);
    assert_eq!(graph.selected(), Some(1));
    assert!(band_rows(&editor.page)[1].is_expanded());
}

#[gtk::test]
fn delete_on_a_handle_removes_its_band() {
    let editor = standalone_editor(with_preset(&one_peak()), "mine");
    let graph = allocated(&editor);
    graph.activate_action("graph.remove-band", None).ok();
    assert_eq!(saves(&editor), 0);
    let (x, y) = handle_at(&graph, 250.0, 2.0);
    graph.press_at(x, y);
    graph
        .activate_action("graph.remove-band", None)
        .expect("the action exists");
    assert!(saved(&editor.log).curve().filters().is_empty());
    assert!(band_rows(&editor.page).is_empty());
    assert!(graph.handles().is_empty());
}

#[gtk::test]
fn a_built_in_preset_shows_handles_that_do_not_move() {
    let editor = standalone_editor(snapshot(), "flat");
    let graph = allocated(&editor);
    assert_eq!(graph.handles().len(), 10);
    let (x, y) = handle_at(&graph, 1_000.0, 0.0);
    graph.press_at(x, y);
    graph.drag_by(20.0, -40.0);
    graph.release_by(20.0, -40.0);
    assert!(editor.log.borrow().is_empty());
}
