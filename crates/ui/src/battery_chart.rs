use arctis_core::view::battery_chart::{BatteryChartModel, ChartPoint, ChartSpan};
use gtk::accessible::Property;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, gsk};

use crate::text::resolve;

const LINE_WIDTH: f32 = 2.0;
const SHADE_ALPHA: f32 = 0.25;
const INSET: f64 = 1.0;

mod imp {
    use std::cell::RefCell;

    use arctis_core::view::battery_chart::BatteryChartModel;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/marcelogm/ArctisCompanion/ui/battery_chart.ui")]
    pub struct BatteryChart {
        pub model: RefCell<Option<BatteryChartModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BatteryChart {
        const NAME: &'static str = "ArctisBatteryChart";
        type Type = super::BatteryChart;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_accessible_role(gtk::AccessibleRole::Img);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for BatteryChart {}
    impl WidgetImpl for BatteryChart {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let chart = self.obj();
            chart.draw(
                snapshot,
                f64::from(chart.width()),
                f64::from(chart.height()),
            );
        }
    }
}

glib::wrapper! {
    pub struct BatteryChart(ObjectSubclass<imp::BatteryChart>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for BatteryChart {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl BatteryChart {
    pub fn render(&self, model: &BatteryChartModel) {
        self.update_property(&[Property::Description(&resolve(&model.description))]);
        self.imp().model.replace(Some(model.clone()));
        self.queue_draw();
    }

    pub fn draw(&self, snapshot: &gtk::Snapshot, width: f64, height: f64) {
        let area = Area { width, height };
        if !area.is_drawable() {
            return;
        }
        if let Some(model) = self.imp().model.borrow().as_ref() {
            paint(snapshot, model, area, self.color());
        }
    }
}

#[derive(Clone, Copy)]
struct Area {
    width: f64,
    height: f64,
}

impl Area {
    fn is_drawable(self) -> bool {
        self.width > 2.0 * INSET && self.height > 2.0 * INSET
    }

    fn x(self, fraction: f64) -> f64 {
        INSET + fraction * (self.width - 2.0 * INSET)
    }

    fn y(self, fraction: f64) -> f64 {
        INSET + (1.0 - fraction) * (self.height - 2.0 * INSET)
    }
}

fn paint(snapshot: &gtk::Snapshot, model: &BatteryChartModel, area: Area, accent: gdk::RGBA) {
    for span in &model.charging {
        shade(snapshot, &shade_path(*span, area), accent);
    }
    for line in &model.lines {
        stroke(snapshot, &line_path(line, area), accent);
    }
}

fn shade_path(span: ChartSpan, area: Area) -> String {
    let (from, to) = (area.x(span.from), area.x(span.to));
    format!("M {from:.2} 0 H {to:.2} V {:.2} H {from:.2} Z", area.height)
}

fn line_path(points: &[ChartPoint], area: Area) -> String {
    let steps: Vec<String> = points
        .iter()
        .map(|point| format!("{:.2} {:.2}", area.x(point.x), area.y(point.y)))
        .collect();
    format!("M {}", steps.join(" L "))
}

fn shade(snapshot: &gtk::Snapshot, data: &str, accent: gdk::RGBA) {
    if let Ok(path) = gsk::Path::parse(data) {
        snapshot.append_fill(
            &path,
            gsk::FillRule::Winding,
            &accent.with_alpha(SHADE_ALPHA),
        );
    }
}

fn stroke(snapshot: &gtk::Snapshot, data: &str, accent: gdk::RGBA) {
    if let Ok(path) = gsk::Path::parse(data) {
        snapshot.append_stroke(&path, &gsk::Stroke::new(LINE_WIDTH), &accent);
    }
}
