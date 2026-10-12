use std::rc::Rc;

use adw::prelude::*;
use arctis_core::testkit::snapshots::snapshot;
use arctis_ui::{BatteryPage, Window};

use crate::battery_history::{NOW, synthetic_week, with_history};
use crate::support::{named, open_page, rendered_window};

fn battery_page_of(window: &Window) -> BatteryPage {
    named::<adw::NavigationView>(window, "navigation")
        .find_page("battery")
        .and_downcast::<BatteryPage>()
        .expect("the battery page")
}

fn battery_content(window: &Window) -> Option<String> {
    named::<gtk::Stack>(&battery_page_of(window), "battery-stack")
        .visible_child_name()
        .map(String::from)
}

#[gtk::test]
fn show_page_battery_pushes_the_battery_page() {
    let (window, _) = rendered_window(snapshot());
    open_page(&window, "battery");
    let navigation = named::<adw::NavigationView>(&window, "navigation");
    let tag = navigation.visible_page().and_then(|page| page.tag());
    assert_eq!(tag.as_deref(), Some("battery"));
}

#[gtk::test]
fn each_render_reaches_the_battery_page() {
    let (window, _) = rendered_window(snapshot());
    assert_eq!(battery_content(&window).as_deref(), Some("empty"));
    open_page(&window, "battery");
    window.render(&Rc::new(with_history(synthetic_week())));
    assert_eq!(battery_content(&window).as_deref(), Some("chart"));
}

#[gtk::test]
fn a_hidden_battery_page_is_not_redrawn() {
    let (window, _) = rendered_window(snapshot());
    window.render(&Rc::new(with_history(synthetic_week())));
    assert_eq!(battery_content(&window).as_deref(), Some("empty"));
}

#[gtk::test]
fn the_battery_clock_runs_only_while_the_page_is_shown() {
    let (window, _) = rendered_window(snapshot());
    assert!(!window.is_refreshing_battery());
    open_page(&window, "battery");
    assert!(window.is_refreshing_battery());
    named::<adw::NavigationView>(&window, "navigation").pop();
    assert!(!window.is_refreshing_battery());
}

#[gtk::test]
fn opening_the_battery_page_redraws_the_latest_snapshot() {
    let (window, _) = rendered_window(with_history(synthetic_week()));
    battery_page_of(&window).display_history(Rc::new(snapshot()), NOW);
    open_page(&window, "battery");
    assert_eq!(battery_content(&window).as_deref(), Some("chart"));
}
