use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::view::eq_graph::{GraphPoint, LevelRange, Series};
use arctis_core::view::eq_plot::{
    Emphasis, GUTTER_RIGHT, Plot, Tick, frequency_ticks, level_ticks, zone_label_centre,
    zone_label_rows, zones,
};
use gtk::{gdk, graphene, gsk, pango};

use super::EqGraph;
use super::ink::{Ink, stroke};
use crate::eq_handle::{pixels, with_alpha};
use crate::text::resolve;

const MINOR_ALPHA: f32 = 0.04;
const MAJOR_ALPHA: f32 = 0.12;
const ZERO_ALPHA: f32 = 0.4;
const ZONE_ALPHA: f32 = 0.035;
const LABEL_ALPHA: f32 = 0.6;
const ZONE_LABEL_ALPHA: f32 = 0.45;
const GRID_WIDTH: f32 = 1.0;
const LABEL_SPACE: f32 = 4.0;
const AXIS_LABEL_SPACE: f32 = 8.0;
const ZONE_LABEL_ROW: f32 = 14.0;

impl EqGraph {
    pub fn draw(&self, snapshot: &gtk::Snapshot, width: i32, height: i32) {
        let model = self.imp().model.borrow();
        let levels = model.as_ref().map(|model| model.levels);
        let plot = Plot::within(
            pixels(width),
            pixels(height),
            levels.unwrap_or(LevelRange::RELATIVE),
        );
        let ink = Ink::of(self);
        self.draw_zones(snapshot, &plot, &ink.fg);
        draw_grid(snapshot, &plot, &ink.fg);
        self.draw_axis_labels(snapshot, &plot, &ink.fg);
        snapshot.push_clip(&rect(plot.left, plot.top, plot.width, plot.height));
        for series in model.iter().flat_map(|model| &model.series) {
            draw_series(snapshot, &plot, series, &ink);
        }
        snapshot.pop();
    }

    fn draw_zones(&self, snapshot: &gtk::Snapshot, plot: &Plot, fg: &gdk::RGBA) {
        let zones = zones();
        let shade = with_alpha(fg, ZONE_ALPHA);
        for zone in zones.iter().skip(1).step_by(2) {
            let left = plot.x_of(zone.low_hz);
            let area = rect(left, plot.top, plot.x_of(zone.high_hz) - left, plot.height);
            snapshot.append_color(&shade, &area);
        }
        let layouts: Vec<pango::Layout> = zones
            .iter()
            .map(|zone| self.small_layout(&resolve(&zone.name)))
            .collect();
        let widths: Vec<f32> = layouts.iter().map(|layout| size_of(layout).0).collect();
        let Some(rows) = zone_label_rows(plot, &widths) else {
            return;
        };
        let ink = with_alpha(fg, ZONE_LABEL_ALPHA);
        for (at, (zone, layout)) in zones.iter().zip(&layouts).enumerate() {
            let width = widths[at];
            let row = f32::from(u8::from(at % rows == 1));
            let x = zone_label_centre(plot, zone, width) - width / 2.0;
            let y = plot.top + LABEL_SPACE + row * ZONE_LABEL_ROW;
            draw_layout(snapshot, layout, (x, y), &ink);
        }
    }

    fn draw_axis_labels(&self, snapshot: &gtk::Snapshot, plot: &Plot, fg: &gdk::RGBA) {
        let width = plot.right() + GUTTER_RIGHT;
        let ink = with_alpha(fg, LABEL_ALPHA);
        for tick in labelled(frequency_ticks()) {
            let (layout, label_width) = self.label(&tick);
            let centre = plot.x_of(tick.value);
            let x = (centre - label_width / 2.0).clamp(0.0, (width - label_width).max(0.0));
            draw_layout(
                snapshot,
                &layout,
                (x, plot.bottom() + AXIS_LABEL_SPACE),
                &ink,
            );
        }
        for tick in labelled(level_ticks(plot.levels, plot.height)) {
            let (layout, label_width) = self.label(&tick);
            let label_height = size_of(&layout).1;
            let x = plot.left - AXIS_LABEL_SPACE - label_width;
            let y = plot.y_of(tick.value) - label_height / 2.0;
            draw_layout(snapshot, &layout, (x, y), &ink);
        }
    }

    fn label(&self, tick: &Tick) -> (pango::Layout, f32) {
        let text = tick.label.as_ref().map(resolve).unwrap_or_default();
        let layout = self.small_layout(&text);
        let width = size_of(&layout).0;
        (layout, width)
    }

    fn small_layout(&self, text: &str) -> pango::Layout {
        small_layout(self, text)
    }
}

pub(super) fn small_layout(widget: &impl IsA<gtk::Widget>, text: &str) -> pango::Layout {
    let layout = widget.create_pango_layout(Some(text));
    let attributes = pango::AttrList::new();
    attributes.insert(pango::AttrFloat::new_scale(pango::SCALE_SMALL));
    attributes.insert(pango::AttrFontFeatures::new("tnum=1"));
    layout.set_attributes(Some(&attributes));
    layout
}

fn labelled(ticks: Vec<Tick>) -> impl Iterator<Item = Tick> {
    ticks.into_iter().filter(|tick| tick.label.is_some())
}

fn draw_grid(snapshot: &gtk::Snapshot, plot: &Plot, fg: &gdk::RGBA) {
    for tick in frequency_ticks() {
        let x = plot.x_of(tick.value);
        let line = segment((x, plot.top), (x, plot.bottom()));
        stroke_line(snapshot, &line, &with_alpha(fg, alpha_of(tick.emphasis)));
    }
    for tick in level_ticks(plot.levels, plot.height) {
        let y = plot.y_of(tick.value);
        let line = segment((plot.left, y), (plot.right(), y));
        stroke_line(snapshot, &line, &with_alpha(fg, alpha_of(tick.emphasis)));
    }
}

const fn alpha_of(emphasis: Emphasis) -> f32 {
    match emphasis {
        Emphasis::Minor => MINOR_ALPHA,
        Emphasis::Major => MAJOR_ALPHA,
        Emphasis::Zero => ZERO_ALPHA,
    }
}

fn draw_series(snapshot: &gtk::Snapshot, plot: &Plot, series: &Series, ink: &Ink) {
    let color = ink.line(series.role, series.weight);
    let place = |point: &GraphPoint| (plot.x_of(point.hz), plot.y_of(point.db));
    let points: Vec<(f32, f32)> = series.points.iter().map(place).collect();
    if series.filled
        && let Some(area) = filled_path(&points, plot.y_of(0.0))
    {
        snapshot.append_fill(&area, gsk::FillRule::Winding, &ink.fill(series.role));
    }
    let Some(line) = line_path(&points) else {
        return;
    };
    snapshot.append_stroke(&line, &stroke(series.style, series.weight), &color);
}

fn line_path(points: &[(f32, f32)]) -> Option<gsk::Path> {
    let (first, rest) = points.split_first()?;
    let builder = gsk::PathBuilder::new();
    builder.move_to(first.0, first.1);
    for (x, y) in rest {
        builder.line_to(*x, *y);
    }
    Some(builder.to_path())
}

fn filled_path(points: &[(f32, f32)], base: f32) -> Option<gsk::Path> {
    let (first, last) = (points.first()?, points.last()?);
    let builder = gsk::PathBuilder::new();
    builder.move_to(first.0, base);
    for (x, y) in points {
        builder.line_to(*x, *y);
    }
    builder.line_to(last.0, base);
    builder.close();
    Some(builder.to_path())
}

fn stroke_line(snapshot: &gtk::Snapshot, line: &gsk::Path, color: &gdk::RGBA) {
    snapshot.append_stroke(line, &gsk::Stroke::new(GRID_WIDTH), color);
}

pub(super) fn segment(from: (f32, f32), to: (f32, f32)) -> gsk::Path {
    let builder = gsk::PathBuilder::new();
    builder.move_to(from.0, from.1);
    builder.line_to(to.0, to.1);
    builder.to_path()
}

pub(super) fn draw_layout(
    snapshot: &gtk::Snapshot,
    layout: &pango::Layout,
    at: (f32, f32),
    ink: &gdk::RGBA,
) {
    snapshot.save();
    snapshot.translate(&graphene::Point::new(at.0, at.1));
    snapshot.append_layout(layout, ink);
    snapshot.restore();
}

pub(super) fn size_of(layout: &pango::Layout) -> (f32, f32) {
    let (width, height) = layout.pixel_size();
    (pixels(width), pixels(height))
}

pub(super) fn rect(x: f32, y: f32, width: f32, height: f32) -> graphene::Rect {
    graphene::Rect::new(x, y, width.max(0.0), height.max(0.0))
}
