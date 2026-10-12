use adw::prelude::*;
use arctis_core::companion::routing::Route;
use arctis_core::messages::device::Phase;
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::{snapshot, with};
use arctis_ui::{HeadsetStatus, PermissionDialog};
use gtk::{AccessibleProperty, AccessibleRelation, AccessibleRole};

use crate::add_rule_dialog::{rendered_dialog, spotify_seen};
use crate::app_presets_page::rendered_page;
use crate::support::{
    descendants, first, has_accessible_text, headset_model, init, rendered_window, rich_snapshot,
    ruled, shown,
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
    unnamed_among(descendants(root))
}

fn unnamed_among(widgets: Vec<gtk::Widget>) -> Vec<String> {
    widgets
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
    assert!(has_accessible_text(
        &status,
        AccessibleProperty::Description,
        &english(&headset_model(&s).header.description)
    ));
}

#[gtk::test]
fn every_control_of_the_app_presets_page_has_a_name() {
    for s in [
        snapshot(),
        ruled(&[("firefox", "footsteps"), ("mpv", "gone")]),
    ] {
        let (page, _, _) = rendered_page(&s);
        assert!(
            descendants(&page).iter().any(needs_name),
            "the walk sees the controls"
        );
        assert_eq!(unnamed(&page), Vec::<String>::new());
    }
}

#[gtk::test]
fn every_control_of_the_add_rule_dialog_has_a_name() {
    for s in [snapshot(), spotify_seen()] {
        let (dialog, _, _) = rendered_dialog(&s);
        let widgets = descendants(&dialog);
        assert!(widgets.iter().any(needs_name), "the walk sees the controls");
        assert_eq!(unnamed_among(widgets), Vec::<String>::new());
    }
}
