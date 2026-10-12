use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use adw::prelude::*;
use arctis_core::companion::routing::Route;
use arctis_core::eq::FilterKind;
use arctis_core::messages::device::Phase;
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::{id, snapshot, with};
use arctis_core::view::eq_editor::eq_editor;
use arctis_ui::{BatteryChart, EqGraph, HeadsetStatus, PermissionDialog};
use gtk::glib;
use gtk::{AccessibleProperty, AccessibleRelation, AccessibleRole};

use crate::add_rule_dialog::{rendered_dialog, spotify_seen};
use crate::app_presets_page::rendered_page;
use crate::battery_history::{shown_battery_page, synthetic_week, with_history};
use crate::support::{
    band_rows, custom_preset, descendants, filter, first, has_accessible_text, headset_model, init,
    open_editor, open_page, open_presets, rendered_window, rich_snapshot, ruled, shown,
    with_preset,
};

const INTERACTIVE: [AccessibleRole; 9] = [
    AccessibleRole::Button,
    AccessibleRole::ToggleButton,
    AccessibleRole::Switch,
    AccessibleRole::Slider,
    AccessibleRole::ComboBox,
    AccessibleRole::SpinButton,
    AccessibleRole::Checkbox,
    AccessibleRole::Radio,
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

fn criticals_during(run: impl FnOnce()) -> usize {
    let count = Arc::new(AtomicUsize::new(0));
    let handlers = ["Gtk", "GLib-GObject"].map(|domain| {
        let count = Arc::clone(&count);
        let handler = glib::log_set_handler(
            Some(domain),
            glib::LogLevels::LEVEL_CRITICAL,
            false,
            false,
            move |_, _, _| {
                count.fetch_add(1, Ordering::SeqCst);
            },
        );
        (domain, handler)
    });
    run();
    for (domain, handler) in handlers {
        glib::log_remove_handler(Some(domain), handler);
    }
    count.load(Ordering::SeqCst)
}

#[gtk::test]
fn every_control_of_the_dialogs_has_a_name() {
    init();
    let (window, _) = rendered_window(snapshot());
    assert_eq!(unnamed(&window.preferences_dialog()), Vec::<String>::new());
    assert_eq!(unnamed(&PermissionDialog::default()), Vec::<String>::new());
}

#[gtk::test]
fn walking_the_dialogs_logs_no_criticals() {
    let criticals = criticals_during(|| {
        unnamed(&PermissionDialog::default());
        unnamed(&rendered_dialog(&spotify_seen()).0);
    });
    assert_eq!(criticals, 0);
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

fn two_kinds_snapshot() -> arctis_core::companion::Snapshot {
    with_preset(&custom_preset(&[
        filter(FilterKind::Peaking, 250, 4, 1_000),
        filter(FilterKind::LowPass, 8_000, 0, 707),
    ]))
}

#[gtk::test]
fn every_control_of_the_presets_page_has_a_name() {
    let (window, _) = rendered_window(two_kinds_snapshot());
    open_presets(&window);
    assert_eq!(unnamed(&window), Vec::<String>::new());
}

#[gtk::test]
fn every_control_of_the_editor_has_a_name() {
    for preset in ["mine", "flat"] {
        let (window, _) = rendered_window(two_kinds_snapshot());
        let page = open_editor(&window, preset);
        for row in band_rows(&page) {
            row.set_expanded(true);
        }
        assert_eq!(unnamed(&window), Vec::<String>::new(), "{preset}");
    }
}

#[gtk::test]
fn the_eq_graph_has_a_description() {
    let s = two_kinds_snapshot();
    let (window, _) = rendered_window(s.clone());
    let page = open_editor(&window, "mine");
    let model = eq_editor(&s, &id("mine")).expect("editor");
    assert!(has_accessible_text(
        &first::<EqGraph>(&page),
        AccessibleProperty::Description,
        &english(&model.graph.description)
    ));
}

#[gtk::test]
fn every_control_of_the_battery_page_has_a_name() {
    let page = shown_battery_page(with_history(synthetic_week()));
    let controls = descendants(&page).into_iter().filter(needs_name).count();
    assert!(controls >= 2, "only {controls} controls need a name");
    assert_eq!(unnamed(&page), Vec::<String>::new());
}

#[gtk::test]
fn every_control_of_the_opened_battery_page_has_a_name() {
    let (window, _) = rendered_window(with_history(synthetic_week()));
    open_page(&window, "battery");
    assert_eq!(unnamed(&window), Vec::<String>::new());
}

#[gtk::test]
fn the_battery_chart_has_a_description() {
    let (window, _) = rendered_window(with_history(synthetic_week()));
    open_page(&window, "battery");
    assert!(gtk::test_accessible_has_property(
        &first::<BatteryChart>(&window),
        AccessibleProperty::Description
    ));
}
