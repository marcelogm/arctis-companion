use adw::prelude::*;
use arctis_core::companion::routing::Route;
use arctis_core::messages::device::Phase;
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::{snapshot, with};
use arctis_ui::{HeadsetStatus, PermissionDialog};
use gtk::{AccessibleProperty, AccessibleRelation, AccessibleRole};

use crate::support::{
    descendants, first, headset_model, init, rendered_window, rich_snapshot, shown,
};

const INTERACTIVE: [AccessibleRole; 8] = [
    AccessibleRole::Button,
    AccessibleRole::ToggleButton,
    AccessibleRole::Switch,
    AccessibleRole::Slider,
    AccessibleRole::ComboBox,
    AccessibleRole::SpinButton,
    AccessibleRole::Checkbox,
    AccessibleRole::Link,
];

fn needs_name(widget: &gtk::Widget) -> bool {
    widget.is_focusable()
        && widget.can_focus()
        && shown(widget)
        && INTERACTIVE.contains(&widget.accessible_role())
}

fn labelled(widget: &impl IsA<gtk::Accessible>) -> bool {
    gtk::test_accessible_has_property(widget, AccessibleProperty::Label)
        || gtk::test_accessible_has_relation(widget, AccessibleRelation::LabelledBy)
}

fn named_by_menu_button(widget: &gtk::Widget) -> bool {
    widget
        .parent()
        .and_downcast::<gtk::MenuButton>()
        .is_some_and(|menu| labelled(&menu))
}

fn unnamed(root: &impl IsA<gtk::Widget>) -> Vec<String> {
    descendants(root)
        .into_iter()
        .filter(needs_name)
        .filter(|widget| !labelled(widget) && !named_by_menu_button(widget))
        .map(|widget| format!("{} {}", widget.type_().name(), widget.widget_name()))
        .collect()
}

#[gtk::test]
fn the_walk_sees_the_controls() {
    let (window, _) = rendered_window(rich_snapshot());
    let controls = descendants(&window).into_iter().filter(needs_name).count();
    assert!(controls >= 6, "only {controls} controls need a name");
}

#[gtk::test]
fn every_control_of_the_headset_page_has_a_name() {
    let (window, _) = rendered_window(rich_snapshot());
    assert_eq!(unnamed(&window), Vec::<String>::new());
}

#[gtk::test]
fn every_control_of_the_status_pages_has_a_name() {
    for phase in [Phase::Absent, Phase::PermissionDenied] {
        let (window, _) = rendered_window(with(phase, Route::Dongle));
        assert_eq!(unnamed(&window), Vec::<String>::new(), "{phase:?}");
    }
}

#[gtk::test]
fn every_control_of_the_banner_has_a_name() {
    let (window, _) = rendered_window(with(Phase::PermissionDenied, Route::Bluetooth));
    assert_eq!(unnamed(&window), Vec::<String>::new());
}

#[gtk::test]
fn every_control_of_the_dialogs_has_a_name() {
    init();
    let (window, _) = rendered_window(snapshot());
    assert_eq!(unnamed(&window.preferences_dialog()), Vec::<String>::new());
    assert_eq!(unnamed(&PermissionDialog::default()), Vec::<String>::new());
}

#[gtk::test]
fn every_shown_scale_has_value_text() {
    let (window, _) = rendered_window(rich_snapshot());
    let scales: Vec<gtk::Scale> = descendants(&window)
        .into_iter()
        .filter(shown)
        .filter_map(|widget| widget.downcast::<gtk::Scale>().ok())
        .collect();
    assert_eq!(scales.len(), 3);
    for scale in scales {
        assert!(gtk::test_accessible_has_property(
            &scale,
            AccessibleProperty::ValueText
        ));
    }
}

#[gtk::test]
fn the_header_has_a_description() {
    let s = rich_snapshot();
    let (window, _) = rendered_window(s.clone());
    let status = first::<HeadsetStatus>(&window);
    assert!(gtk::test_accessible_has_property(
        &status,
        AccessibleProperty::Description
    ));
    assert_eq!(
        status.description(),
        english(&headset_model(&s).header.description)
    );
}
