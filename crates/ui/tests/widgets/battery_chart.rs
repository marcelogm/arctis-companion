use adw::prelude::*;
use arctis_core::battery::series::{Window as ChartWindow, series};
use arctis_core::testkit::english::english;
use arctis_core::view::battery_chart::{BatteryChartModel, ChartPoint, ChartSpan, battery_chart};
use arctis_core::view::text::Text;
use arctis_ui::BatteryChart;
use gtk::{AccessibleProperty, AccessibleRole, gdk, gsk};

use crate::battery_history::{
    CHART_HEIGHT, CHART_WIDTH, NOW, PLOT_INSET, SHADE_ALPHA, count, drawn, stroke_bounds,
    synthetic_week,
};
use crate::support::{assert_close, has_accessible_text, init};

fn model(lines: Vec<Vec<ChartPoint>>, charging: Vec<ChartSpan>) -> BatteryChartModel {
    BatteryChartModel {
        window: ChartWindow::Day,
        lines,
        charging,
        description: Text::plain("No battery readings in the last day"),
    }
}

fn rendered(model: &BatteryChartModel) -> BatteryChart {
    init();
    let chart = BatteryChart::default();
    chart.render(model);
    chart
}

#[gtk::test]
fn the_chart_is_a_named_image() {
    init();
    let chart = BatteryChart::default();
    assert_eq!(chart.accessible_role(), AccessibleRole::Img);
    assert!(has_accessible_text(
        &chart,
        AccessibleProperty::Label,
        "Battery Level Chart"
    ));
}

#[gtk::test]
fn the_description_reaches_assistive_technology() {
    let drawn = model(Vec::new(), Vec::new());
    let chart = rendered(&drawn);
    assert!(has_accessible_text(
        &chart,
        AccessibleProperty::Description,
        &english(&drawn.description)
    ));
}

fn point(x: f64, y: f64) -> ChartPoint {
    ChartPoint { x, y }
}

fn level_line(from: f64, to: f64, y: f64) -> Vec<ChartPoint> {
    vec![point(from, y), point(to, y)]
}

#[gtk::test]
fn charging_spans_are_shaded_across_the_full_height() {
    let chart = rendered(&model(
        Vec::new(),
        vec![ChartSpan {
            from: 0.5,
            to: 0.75,
        }],
    ));
    let nodes = drawn(&chart, CHART_WIDTH, CHART_HEIGHT);
    assert_eq!(count(&nodes, gsk::RenderNodeType::FillNode), 1);
    let shade = nodes[0].bounds();
    assert_close(
        f64::from(shade.x()),
        PLOT_INSET + 0.5 * (CHART_WIDTH - 2.0 * PLOT_INSET),
    );
    assert_close(
        f64::from(shade.width()),
        0.25 * (CHART_WIDTH - 2.0 * PLOT_INSET),
    );
    assert_close(f64::from(shade.y()), 0.0);
    assert_close(f64::from(shade.height()), CHART_HEIGHT);
}

#[gtk::test]
fn gaps_break_the_line() {
    let lines = vec![level_line(0.0, 0.4, 0.8), level_line(0.6, 1.0, 0.7)];
    let nodes = drawn(
        &rendered(&model(lines, Vec::new())),
        CHART_WIDTH,
        CHART_HEIGHT,
    );
    let strokes = stroke_bounds(&nodes);
    assert_eq!(strokes.len(), 2);
    assert!(strokes[0].x() + strokes[0].width() < strokes[1].x());
}

#[gtk::test]
fn a_full_battery_is_drawn_at_the_top_inside_the_chart() {
    let lines = vec![vec![point(0.0, 1.0), point(0.5, 0.0), point(1.0, 0.5)]];
    let nodes = drawn(
        &rendered(&model(lines, Vec::new())),
        CHART_WIDTH,
        CHART_HEIGHT,
    );
    let line = stroke_bounds(&nodes)[0];
    assert_close(f64::from(line.x()), PLOT_INSET);
    assert_close(f64::from(line.y()), PLOT_INSET);
    assert_close(f64::from(line.width()), CHART_WIDTH - 2.0 * PLOT_INSET);
    assert_close(f64::from(line.height()), CHART_HEIGHT - 2.0 * PLOT_INSET);
}

#[gtk::test]
fn spans_are_drawn_behind_the_lines() {
    let chart = rendered(&model(
        vec![level_line(0.0, 1.0, 0.5)],
        vec![ChartSpan { from: 0.0, to: 0.5 }],
    ));
    let kinds: Vec<gsk::RenderNodeType> = drawn(&chart, CHART_WIDTH, CHART_HEIGHT)
        .iter()
        .map(gsk::RenderNode::node_type)
        .collect();
    assert_eq!(
        kinds,
        vec![
            gsk::RenderNodeType::FillNode,
            gsk::RenderNodeType::StrokeNode
        ]
    );
}

#[gtk::test]
fn nothing_is_drawn_without_a_model_or_without_room() {
    init();
    let chart = BatteryChart::default();
    assert!(drawn(&chart, CHART_WIDTH, CHART_HEIGHT).is_empty());
    chart.render(&model(vec![level_line(0.0, 1.0, 0.5)], Vec::new()));
    assert!(drawn(&chart, 0.0, 0.0).is_empty());
}

fn painted_color(node: &gsk::RenderNode) -> Option<gdk::RGBA> {
    let paint = node
        .downcast_ref::<gsk::FillNode>()
        .map(gsk::FillNode::child)
        .or_else(|| {
            node.downcast_ref::<gsk::StrokeNode>()
                .map(gsk::StrokeNode::child)
        })?;
    paint
        .downcast_ref::<gsk::ColorNode>()
        .map(gsk::ColorNode::color)
}

#[gtk::test]
fn the_chart_paints_in_the_accent_color() {
    let chart = rendered(&model(
        vec![level_line(0.0, 1.0, 0.5)],
        vec![ChartSpan { from: 0.0, to: 0.5 }],
    ));
    assert!(chart.has_css_class("accent"));
    let accent = chart.color();
    let colors: Vec<Option<gdk::RGBA>> = drawn(&chart, CHART_WIDTH, CHART_HEIGHT)
        .iter()
        .map(painted_color)
        .collect();
    assert_eq!(
        colors,
        vec![Some(accent.with_alpha(SHADE_ALPHA)), Some(accent)]
    );
}

fn week_chart(range: ChartWindow) -> BatteryChartModel {
    battery_chart(&series(&synthetic_week(), range, NOW), range)
}

#[gtk::test]
fn day_and_week_draw_a_synthetic_week() {
    for range in [ChartWindow::Day, ChartWindow::Week] {
        let expected = week_chart(range);
        assert!(expected.lines.len() >= 2, "{range:?} has a gap");
        assert!(!expected.charging.is_empty(), "{range:?} has a charge");
        let nodes = drawn(&rendered(&expected), CHART_WIDTH, CHART_HEIGHT);
        assert_eq!(
            count(&nodes, gsk::RenderNodeType::StrokeNode),
            expected.lines.len(),
            "{range:?}"
        );
        assert_eq!(
            count(&nodes, gsk::RenderNodeType::FillNode),
            expected.charging.len(),
            "{range:?}"
        );
    }
}

#[gtk::test]
fn the_week_shows_more_charges_than_the_day() {
    assert!(
        week_chart(ChartWindow::Week).charging.len() > week_chart(ChartWindow::Day).charging.len()
    );
}

#[gtk::test]
fn every_line_of_the_week_stays_inside_the_chart() {
    let nodes = drawn(
        &rendered(&week_chart(ChartWindow::Week)),
        CHART_WIDTH,
        CHART_HEIGHT,
    );
    for line in stroke_bounds(&nodes) {
        assert!(
            f64::from(line.x()) >= PLOT_INSET && f64::from(line.y()) >= PLOT_INSET,
            "{line:?}"
        );
        assert!(
            f64::from(line.x() + line.width()) <= CHART_WIDTH - PLOT_INSET,
            "{line:?}"
        );
        assert!(
            f64::from(line.y() + line.height()) <= CHART_HEIGHT - PLOT_INSET,
            "{line:?}"
        );
    }
}
