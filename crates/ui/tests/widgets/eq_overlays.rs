use adw::prelude::*;
use arctis_core::eq::library;
use arctis_core::eq::reference::{Overlays, Target};
use arctis_core::intent::Intent;
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::snapshot;
use arctis_core::view::eq_graph::{LevelRange, graph, graph_with};
use arctis_core::view::eq_overlays::OVERLAID;
use arctis_ui::{EqEditorPage, EqGraph, EqLegend};
use gtk::{AccessibleProperty, AccessibleRole, gio, gsk};

use crate::support::{Mounted, descendants, has_accessible_text, init, named, standalone_editor};

const HEADSET: &str = "reference.headset";
const TARGET: &str = "reference.target";
const ITEMS: [&str; 4] = [
    "Headset Response",
    "None",
    "Harman (B&K 5128)",
    "Diffuse field (B&K 5128)",
];

fn editor() -> Mounted<EqEditorPage> {
    standalone_editor(snapshot(), "harman")
}

fn legend(editor: &Mounted<EqEditorPage>) -> EqLegend {
    named::<EqLegend>(&editor.page, "eq-legend")
}

fn graph_widget(editor: &Mounted<EqEditorPage>) -> EqGraph {
    named::<EqGraph>(&editor.page, "eq-graph")
}

fn activate(editor: &Mounted<EqEditorPage>, action: &str, target: Option<&str>) {
    let parameter = target.map(ToVariant::to_variant);
    WidgetExt::activate_action(&editor.page, action, parameter.as_ref())
        .expect("the action exists");
}

fn checked(editor: &Mounted<EqEditorPage>, label: &str) -> bool {
    let popover = named::<gtk::MenuButton>(&editor.page, "reference-menu")
        .popover()
        .expect("a popover");
    descendants(&popover)
        .into_iter()
        .filter(|widget| widget.type_().name() == "GtkModelButton")
        .find(|item| item.property::<Option<String>>("text").as_deref() == Some(label))
        .unwrap_or_else(|| panic!("no menu item {label}"))
        .property::<bool>("active")
}

fn checked_items(editor: &Mounted<EqEditorPage>) -> Vec<&'static str> {
    ITEMS
        .into_iter()
        .filter(|label| checked(editor, label))
        .collect()
}

fn chosen(editor: &Mounted<EqEditorPage>) -> Vec<Overlays> {
    editor
        .log
        .borrow()
        .iter()
        .filter_map(|intent| match intent {
            Intent::SetOverlays(overlays) => Some(*overlays),
            _ => None,
        })
        .collect()
}

fn children(node: &gsk::RenderNode) -> Vec<gsk::RenderNode> {
    if let Some(container) = node.downcast_ref::<gsk::ContainerNode>() {
        return (0..container.n_children())
            .map(|at| container.child(at))
            .collect();
    }
    node.downcast_ref::<gsk::ClipNode>()
        .map(|clip| vec![clip.child()])
        .unwrap_or_default()
}

fn strokes(node: &gsk::RenderNode) -> usize {
    let own = usize::from(node.node_type() == gsk::RenderNodeType::StrokeNode);
    own + children(node).iter().map(strokes).sum::<usize>()
}

fn series_strokes(node: &gsk::RenderNode) -> usize {
    if node.node_type() == gsk::RenderNodeType::ClipNode {
        return strokes(node);
    }
    children(node).iter().map(series_strokes).sum()
}

fn drawn_strokes(model: &arctis_core::view::eq_graph::GraphModel) -> usize {
    init();
    let graph_widget = EqGraph::default();
    graph_widget.render(model, false);
    let snapshot = gtk::Snapshot::new();
    graph_widget.draw(&snapshot, 680, 300);
    series_strokes(&snapshot.to_node().expect("something was drawn"))
}

#[gtk::test]
fn a_reference_menu_offers_the_headset_and_one_target() {
    let editor = editor();
    let button = named::<gtk::MenuButton>(&editor.page, "reference-menu");
    assert!(button.is_visible());
    assert_eq!(button.label().as_deref(), Some("Reference"));
    let menu = button.menu_model().expect("a menu");
    let section = |at: i32| {
        menu.item_link(at, gio::MENU_LINK_SECTION)
            .expect("a section")
    };
    let action_of = |model: &gio::MenuModel, at: i32| {
        model
            .item_attribute_value(at, gio::MENU_ATTRIBUTE_ACTION, None)
            .and_then(|value| value.str().map(str::to_owned))
            .expect("an action")
    };
    assert_eq!(action_of(&section(0), 0), HEADSET);
    let targets = section(1);
    let nicks: Vec<String> = (0..targets.n_items())
        .map(|at| {
            assert_eq!(action_of(&targets, at), TARGET);
            targets
                .item_attribute_value(at, gio::MENU_ATTRIBUTE_TARGET, None)
                .and_then(|value| value.str().map(str::to_owned))
                .expect("a target")
        })
        .collect();
    assert_eq!(nicks, ["none", "harman", "diffuse-field"]);
    let known: Vec<Option<Target>> = nicks
        .iter()
        .map(|nick| Target::from_setting(nick))
        .collect();
    assert_eq!(
        known,
        [None, Some(Target::Harman), Some(Target::DiffuseField)]
    );
}

#[gtk::test]
fn the_overlays_the_settings_ask_for_are_shown_and_checked() {
    let editor = editor();
    assert_eq!(
        checked_items(&editor),
        ["Headset Response", "Harman (B&K 5128)"]
    );
    assert_eq!(
        legend(&editor).labels(),
        [
            "Headset",
            "Headset with EQ",
            "Harman (B&K 5128)",
            "Equalizer"
        ]
    );
    assert_eq!(graph_widget(&editor).plot().levels, OVERLAID);
    let harman = library::find(&arctis_core::testkit::snapshots::id("harman")).expect("harman");
    let description = english(&graph_with(&harman, Overlays::default()).description);
    assert!(description.contains("Harman target"));
    assert!(has_accessible_text(
        &graph_widget(&editor),
        AccessibleProperty::Description,
        &description
    ));
    assert!(chosen(&editor).is_empty());
}

#[gtk::test]
fn unchecking_the_headset_response_hides_it_and_saves_the_choice() {
    let editor = editor();
    activate(&editor, HEADSET, None);
    let hidden = Overlays::default().with_headset(false);
    assert_eq!(chosen(&editor), [hidden]);
    assert_eq!(checked_items(&editor), ["Harman (B&K 5128)"]);
    assert_eq!(legend(&editor).labels(), ["Harman (B&K 5128)", "Equalizer"]);
}

#[gtk::test]
fn picking_another_target_swaps_the_dashed_line() {
    let editor = editor();
    activate(&editor, TARGET, Some("diffuse-field"));
    let diffuse = Overlays::default().with_target_setting("diffuse-field");
    assert_eq!(chosen(&editor), [diffuse]);
    assert_eq!(
        checked_items(&editor),
        ["Headset Response", "Diffuse field (B&K 5128)"]
    );
    assert!(
        legend(&editor)
            .labels()
            .contains(&"Diffuse field (B&K 5128)".to_owned())
    );
}

#[gtk::test]
fn turning_every_overlay_off_restores_the_plain_equalizer_graph() {
    let editor = editor();
    activate(&editor, TARGET, Some("none"));
    activate(&editor, HEADSET, None);
    assert_eq!(chosen(&editor).last(), Some(&Overlays::NONE));
    assert!(legend(&editor).labels().is_empty());
    assert_eq!(graph_widget(&editor).plot().levels, LevelRange::RELATIVE);
    activate(&editor, HEADSET, None);
    assert_eq!(graph_widget(&editor).plot().levels, OVERLAID);
}

#[gtk::test]
fn a_published_choice_updates_the_menu_without_saving_again() {
    let editor = editor();
    let mut published = snapshot();
    published.presets.overlays = Overlays::NONE.with_target_setting("diffuse-field");
    editor.page.render(&std::rc::Rc::new(published));
    assert_eq!(checked_items(&editor), ["Diffuse field (B&K 5128)"]);
    assert!(chosen(&editor).is_empty());
}

#[gtk::test]
fn overlays_are_drawn_as_three_more_lines() {
    let harman = library::find(&arctis_core::testkit::snapshots::id("harman")).expect("harman");
    let plain = drawn_strokes(&graph(&harman));
    let overlaid = drawn_strokes(&graph_with(&harman, Overlays::default()));
    assert_eq!((plain, overlaid), (1, 4));
}

#[gtk::test]
fn the_legend_is_decorative_and_wraps_or_hides_when_narrow() {
    let editor = editor();
    let legend = legend(&editor);
    assert_eq!(legend.accessible_role(), AccessibleRole::Presentation);
    assert_eq!(legend.rows_at(2_000), 1);
    assert_eq!(legend.rows_at(260), 2);
    assert_eq!(legend.rows_at(60), 0);
    let (narrow, _, _, _) = legend.measure(gtk::Orientation::Vertical, 260);
    let (wide, _, _, _) = legend.measure(gtk::Orientation::Vertical, 2_000);
    assert!(narrow > wide && wide > 0, "{narrow} {wide}");
    let (hidden, _, _, _) = legend.measure(gtk::Orientation::Vertical, 60);
    assert_eq!(hidden, 0);
}
