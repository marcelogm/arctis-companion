use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::view::eq_editor::{
    GRID_DB, GRID_HZ, GraphModel, GraphPoint, x_fraction, y_fraction,
};
use gtk::accessible::Property;
use gtk::{gdk, glib, gsk};

use crate::text::resolve;

const GRID_ALPHA: f32 = 0.15;
const GRID_WIDTH: f32 = 1.0;
const CURVE_WIDTH: f32 = 2.0;

mod imp {
    use std::cell::RefCell;

    use adw::subclass::prelude::*;
    use arctis_core::view::eq_editor::GraphPoint;
    use gtk::glib;
    use gtk::prelude::WidgetExt;

    #[derive(Default)]
    pub struct ResponseGraph {
        pub points: RefCell<Vec<GraphPoint>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ResponseGraph {
        const NAME: &'static str = "ArctisResponseGraph";
        type Type = super::ResponseGraph;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("response-graph");
            klass.set_accessible_role(gtk::AccessibleRole::Img);
        }
    }

    impl ObjectImpl for ResponseGraph {}

    impl WidgetImpl for ResponseGraph {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let graph = self.obj();
            graph.draw(snapshot, graph.width(), graph.height());
        }
    }
}

glib::wrapper! {
    pub struct ResponseGraph(ObjectSubclass<imp::ResponseGraph>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for ResponseGraph {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl ResponseGraph {
    pub fn render(&self, model: &GraphModel) {
        self.imp().points.replace(model.points.clone());
        self.update_property(&[Property::Description(&resolve(&model.description))]);
        self.queue_draw();
    }

    pub fn draw(&self, snapshot: &gtk::Snapshot, width: i32, height: i32) {
        let area = (pixels(width), pixels(height));
        let grid = faded(&self.color());
        for line in grid_lines(area) {
            snapshot.append_stroke(&line, &gsk::Stroke::new(GRID_WIDTH), &grid);
        }
        if let Some(curve) = curve_path(&self.imp().points.borrow(), area) {
            let accent = adw::StyleManager::default().accent_color_rgba();
            snapshot.append_stroke(&curve, &gsk::Stroke::new(CURVE_WIDTH), &accent);
        }
    }
}

fn pixels(size: i32) -> f32 {
    i16::try_from(size.max(0)).map_or(f32::from(i16::MAX), f32::from)
}

fn faded(color: &gdk::RGBA) -> gdk::RGBA {
    gdk::RGBA::new(color.red(), color.green(), color.blue(), GRID_ALPHA)
}

fn grid_lines((width, height): (f32, f32)) -> Vec<gsk::Path> {
    let verticals = GRID_HZ
        .into_iter()
        .map(|hz| x_fraction(hz) * width)
        .map(|x| segment((x, 0.0), (x, height)));
    let horizontals = GRID_DB
        .into_iter()
        .map(|db| y_fraction(db) * height)
        .map(|y| segment((0.0, y), (width, y)));
    verticals.chain(horizontals).collect()
}

fn segment(from: (f32, f32), to: (f32, f32)) -> gsk::Path {
    let builder = gsk::PathBuilder::new();
    builder.move_to(from.0, from.1);
    builder.line_to(to.0, to.1);
    builder.to_path()
}

fn curve_path(points: &[GraphPoint], (width, height): (f32, f32)) -> Option<gsk::Path> {
    let place = |point: &GraphPoint| (x_fraction(point.hz) * width, y_fraction(point.db) * height);
    let (first, rest) = points.split_first()?;
    let builder = gsk::PathBuilder::new();
    let (x, y) = place(first);
    builder.move_to(x, y);
    for (x, y) in rest.iter().map(place) {
        builder.line_to(x, y);
    }
    Some(builder.to_path())
}
