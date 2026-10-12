use std::rc::Rc;

use adw::prelude::*;
use arctis_core::battery::sample::Reading;
use arctis_core::battery::series::Window as ChartWindow;
use arctis_core::companion::Snapshot;
use arctis_core::protocol::status::Link;
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::snapshot;
use arctis_core::view::battery_chart::{
    BatteryChartModel, BatteryContent, BatteryPageModel, battery_page,
};
use arctis_ui::text::resolve;
use arctis_ui::{BatteryChart, BatteryPage};
use gtk::{AccessibleProperty, gsk};

use crate::battery_history::{
    CHART_HEIGHT, CHART_WIDTH, NOW, count, drawn, shown_battery_page, synthetic_week, with_history,
};
use crate::support::{first, has_accessible_text, named};

fn model(s: &Snapshot, range: ChartWindow) -> BatteryPageModel {
    battery_page(s, range, NOW)
}

fn chart_model(s: &Snapshot, range: ChartWindow) -> BatteryChartModel {
    match model(s, range).content {
        BatteryContent::Chart(chart) => chart,
        BatteryContent::Empty(_) => panic!("a chart"),
    }
}

fn visible(page: &BatteryPage) -> Option<String> {
    named::<gtk::Stack>(page, "battery-stack")
        .visible_child_name()
        .map(String::from)
}

fn describes(page: &BatteryPage, chart: &BatteryChartModel) -> bool {
    has_accessible_text(
        &named::<BatteryChart>(page, "battery-chart"),
        AccessibleProperty::Description,
        &english(&chart.description),
    )
}

#[gtk::test]
fn the_page_answers_to_the_battery_tag() {
    let page = shown_battery_page(snapshot());
    assert_eq!(page.tag().as_deref(), Some("battery"));
}

#[gtk::test]
fn no_history_shows_the_empty_state() {
    let s = snapshot();
    let page = shown_battery_page(s.clone());
    assert_eq!(visible(&page).as_deref(), Some("empty"));
    let BatteryContent::Empty(empty) = model(&s, ChartWindow::Day).content else {
        panic!("empty state");
    };
    let status = first::<adw::StatusPage>(&page);
    assert_eq!(status.title(), english(&empty.title));
    assert_eq!(
        status.description().as_deref(),
        Some(english(&empty.description).as_str())
    );
    assert!(!named::<gtk::Button>(&page, "status-button").get_visible());
}

#[gtk::test]
fn history_shows_the_day_chart_first() {
    let s = with_history(synthetic_week());
    let page = shown_battery_page(s.clone());
    assert_eq!(visible(&page).as_deref(), Some("chart"));
    assert!(describes(&page, &chart_model(&s, ChartWindow::Day)));
}

#[gtk::test]
fn emptied_history_returns_to_the_empty_state() {
    let page = shown_battery_page(with_history(synthetic_week()));
    page.display_history(Rc::new(snapshot()), NOW);
    assert_eq!(visible(&page).as_deref(), Some("empty"));
}

fn choose(page: &BatteryPage, range: &str) {
    named::<adw::ToggleGroup>(page, "chart-range").set_active_name(Some(range));
}

fn assert_draws(page: &BatteryPage, expected: &BatteryChartModel) {
    let nodes = drawn(
        &named::<BatteryChart>(page, "battery-chart"),
        CHART_WIDTH,
        CHART_HEIGHT,
    );
    assert_eq!(
        count(&nodes, gsk::RenderNodeType::StrokeNode),
        expected.lines.len()
    );
    assert_eq!(
        count(&nodes, gsk::RenderNodeType::FillNode),
        expected.charging.len()
    );
}

#[gtk::test]
fn the_toggle_starts_on_day() {
    let page = shown_battery_page(with_history(synthetic_week()));
    assert_eq!(
        named::<adw::ToggleGroup>(&page, "chart-range")
            .active_name()
            .as_deref(),
        Some("day")
    );
}

#[gtk::test]
fn day_and_week_render_the_synthetic_week() {
    let s = with_history(synthetic_week());
    let page = shown_battery_page(s.clone());
    for (name, range) in [("week", ChartWindow::Week), ("day", ChartWindow::Day)] {
        choose(&page, name);
        let expected = chart_model(&s, range);
        assert!(describes(&page, &expected), "{name}");
        assert_draws(&page, &expected);
    }
}

#[gtk::test]
fn a_new_snapshot_keeps_the_chosen_range() {
    let s = with_history(synthetic_week());
    let page = shown_battery_page(s.clone());
    choose(&page, "week");
    page.display_history(Rc::new(s.clone()), NOW);
    assert!(describes(&page, &chart_model(&s, ChartWindow::Week)));
}

#[gtk::test]
fn the_rows_show_the_last_full_charge_and_the_level() {
    let s = with_history(synthetic_week());
    let page = shown_battery_page(s.clone());
    let expected = model(&s, ChartWindow::Day);
    let last_full = named::<adw::ActionRow>(&page, "last-full-charge").subtitle();
    assert_eq!(
        last_full.as_deref(),
        Some(resolve(&expected.last_full_charge).as_str())
    );
    let level = named::<adw::ActionRow>(&page, "current-level");
    assert!(level.get_visible());
    let text = expected.current_level.as_ref().map(english);
    assert_eq!(level.subtitle().as_deref(), text.as_deref());
}

#[gtk::test]
fn an_unknown_or_stale_level_hides_its_row() {
    let connected = snapshot().battery.reading;
    let headset_off = connected.map(|reading| Reading {
        link: Link::HeadsetOff,
        ..reading
    });
    for reading in [None, headset_off] {
        let mut s = with_history(synthetic_week());
        s.battery.reading = reading;
        assert_eq!(
            model(&s, ChartWindow::Day).current_level,
            None,
            "{reading:?}"
        );
        let page = shown_battery_page(s);
        assert!(
            !named::<adw::ActionRow>(&page, "current-level").get_visible(),
            "{reading:?}"
        );
    }
}
