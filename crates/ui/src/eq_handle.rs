use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::view::eq_graph::Handle;
use gtk::accessible::Property;
use gtk::{gdk, glib, graphene, gsk, pango};

use crate::text::{resolve, translated};
use arctis_core::l10n::msgid;

pub const HANDLE_SIZE: i32 = 24;
const DOT_RADIUS: f32 = 9.0;
const RING_WIDTH: f32 = 2.0;
const FOCUS_RADIUS: f32 = 12.0;
const IDLE_FILL_ALPHA: f32 = 0.3;
const FOCUS_ALPHA: f32 = 0.5;
const OFF_ALPHA: f32 = 0.45;

mod imp {
    use std::cell::{Cell, RefCell};

    use adw::subclass::prelude::*;
    use gtk::glib;
    use gtk::prelude::*;

    #[derive(Default)]
    pub struct EqHandle {
        pub number: RefCell<String>,
        pub selected: Cell<bool>,
        pub enabled: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EqHandle {
        const NAME: &'static str = "ArctisEqHandle";
        type Type = super::EqHandle;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("eq-handle");
            klass.set_accessible_role(gtk::AccessibleRole::Slider);
        }
    }

    impl ObjectImpl for EqHandle {
        fn constructed(&self) {
            self.parent_constructed();
            let handle = self.obj();
            handle.set_focusable(true);
            handle.set_can_target(false);
            handle.set_size_request(super::HANDLE_SIZE, super::HANDLE_SIZE);
            handle.update_property(&[gtk::accessible::Property::Description(
                &super::keyboard_hint(),
            )]);
        }
    }

    impl WidgetImpl for EqHandle {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.obj().draw(snapshot);
        }
    }
}

glib::wrapper! {
    pub struct EqHandle(ObjectSubclass<imp::EqHandle>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for EqHandle {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl EqHandle {
    pub fn render(&self, handle: &Handle, selected: bool) {
        let imp = self.imp();
        imp.number.replace(handle.number.to_string());
        imp.enabled.set(handle.enabled);
        imp.selected.set(selected);
        self.update_property(&[Property::Label(&resolve(&handle.label))]);
        self.queue_draw();
    }

    pub fn set_selected(&self, selected: bool) {
        if self.imp().selected.replace(selected) != selected {
            self.queue_draw();
        }
    }

    #[must_use]
    pub fn is_selected(&self) -> bool {
        self.imp().selected.get()
    }

    fn draw(&self, snapshot: &gtk::Snapshot) {
        let imp = self.imp();
        let alpha = if imp.enabled.get() { 1.0 } else { OFF_ALPHA };
        let accent = with_alpha(&adw::StyleManager::default().accent_color_rgba(), alpha);
        let centre = half(self.width(), self.height());
        if self.has_visible_focus() {
            let ring = with_alpha(&accent, FOCUS_ALPHA * alpha);
            snapshot.append_stroke(&circle(centre, FOCUS_RADIUS), &stroke(), &ring);
        }
        let dot = circle(centre, DOT_RADIUS);
        let selected = imp.selected.get();
        let fill = if selected {
            accent
        } else {
            with_alpha(&accent, IDLE_FILL_ALPHA * alpha)
        };
        snapshot.append_fill(&dot, gsk::FillRule::Winding, &fill);
        snapshot.append_stroke(&dot, &stroke(), &accent);
        let ink = if selected {
            with_alpha(&gdk::RGBA::WHITE, alpha)
        } else {
            with_alpha(&self.color(), alpha)
        };
        self.draw_number(snapshot, centre, &ink);
    }

    fn draw_number(&self, snapshot: &gtk::Snapshot, centre: (f32, f32), ink: &gdk::RGBA) {
        let layout = self.create_pango_layout(Some(self.imp().number.borrow().as_str()));
        let attributes = pango::AttrList::new();
        attributes.insert(pango::AttrInt::new_weight(pango::Weight::Bold));
        attributes.insert(pango::AttrFloat::new_scale(pango::SCALE_SMALL));
        layout.set_attributes(Some(&attributes));
        let (width, height) = layout.pixel_size();
        snapshot.save();
        snapshot.translate(&graphene::Point::new(
            centre.0 - pixels(width) / 2.0,
            centre.1 - pixels(height) / 2.0,
        ));
        snapshot.append_layout(&layout, ink);
        snapshot.restore();
    }
}

fn keyboard_hint() -> String {
    translated(msgid(
        "Arrow keys change frequency and gain, Page Up and Page Down change Q",
    ))
}

fn stroke() -> gsk::Stroke {
    gsk::Stroke::new(RING_WIDTH)
}

fn circle(centre: (f32, f32), radius: f32) -> gsk::Path {
    let builder = gsk::PathBuilder::new();
    builder.add_circle(&graphene::Point::new(centre.0, centre.1), radius);
    builder.to_path()
}

fn half(width: i32, height: i32) -> (f32, f32) {
    (pixels(width) / 2.0, pixels(height) / 2.0)
}

pub fn pixels(size: i32) -> f32 {
    i16::try_from(size.max(0)).map_or(f32::from(i16::MAX), f32::from)
}

pub fn with_alpha(color: &gdk::RGBA, alpha: f32) -> gdk::RGBA {
    gdk::RGBA::new(
        color.red(),
        color.green(),
        color.blue(),
        color.alpha() * alpha,
    )
}
