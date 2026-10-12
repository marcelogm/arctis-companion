use adw::prelude::*;
use adw::subclass::prelude::*;
use arctis_core::view::eq_legend::{
    LegendEntry, SWATCH_GAP, SWATCH_WIDTH, natural_width, rows, slots,
};
use gtk::{glib, pango};

use super::draw::{draw_layout, rect, segment, size_of, small_layout};
use super::ink::{Ink, stroke};
use super::input::whole;
use crate::eq_handle::{pixels, with_alpha};
use crate::text::resolve;

const ROW_GAP: f32 = 4.0;
const LABEL_ALPHA: f32 = 0.8;
const FILL_HEIGHT: f32 = 6.0;

mod imp {
    use std::cell::RefCell;

    use adw::subclass::prelude::*;
    use arctis_core::view::eq_legend::LegendEntry;
    use gtk::glib;

    #[derive(Default)]
    pub struct EqLegend {
        pub entries: RefCell<Vec<LegendEntry>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EqLegend {
        const NAME: &'static str = "ArctisEqLegend";
        type Type = super::EqLegend;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("eq-legend");
            klass.set_accessible_role(gtk::AccessibleRole::Presentation);
        }
    }

    impl ObjectImpl for EqLegend {}

    impl WidgetImpl for EqLegend {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }

        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.obj().measured(orientation, for_size)
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.obj().draw(snapshot);
        }
    }
}

glib::wrapper! {
    pub struct EqLegend(ObjectSubclass<imp::EqLegend>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for EqLegend {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl EqLegend {
    pub fn render(&self, entries: &[LegendEntry]) {
        if self.imp().entries.borrow().as_slice() == entries {
            return;
        }
        self.imp().entries.replace(entries.to_vec());
        self.queue_resize();
    }

    #[must_use]
    pub fn labels(&self) -> Vec<String> {
        self.imp()
            .entries
            .borrow()
            .iter()
            .map(|entry| resolve(&entry.label))
            .collect()
    }

    #[must_use]
    pub fn rows_at(&self, width: i32) -> usize {
        let widths = widths(&self.layouts());
        slots(&widths, pixels(width)).map_or(0, |placed| rows(&placed))
    }

    fn layouts(&self) -> Vec<pango::Layout> {
        self.labels()
            .iter()
            .map(|label| small_layout(self, label))
            .collect()
    }

    fn measured(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
        let layouts = self.layouts();
        let widths = widths(&layouts);
        let natural = natural_width(&widths);
        if orientation == gtk::Orientation::Horizontal {
            return (0, whole(natural.ceil()), -1, -1);
        }
        let available = if for_size < 0 {
            natural
        } else {
            pixels(for_size)
        };
        let shown = slots(&widths, available).map_or(0, |placed| rows(&placed));
        let height = whole(block_height(shown, row_height(&layouts)).ceil());
        (height, height, -1, -1)
    }

    fn draw(&self, snapshot: &gtk::Snapshot) {
        let layouts = self.layouts();
        let Some(placed) = slots(&widths(&layouts), pixels(self.width())) else {
            return;
        };
        let ink = Ink::of(self);
        let row = row_height(&layouts);
        let entries = self.imp().entries.borrow();
        for ((entry, layout), slot) in entries.iter().zip(&layouts).zip(&placed) {
            let top = row_top(slot.row, row);
            draw_entry(snapshot, &ink, (entry, layout), (slot.x, top, row));
        }
    }
}

fn draw_entry(
    snapshot: &gtk::Snapshot,
    ink: &Ink,
    (entry, layout): (&LegendEntry, &pango::Layout),
    (x, top, row): (f32, f32, f32),
) {
    let middle = top + row / 2.0;
    let color = ink.line(entry.role, entry.weight);
    if entry.filled {
        let fill = ink.fill(entry.role);
        snapshot.append_color(&fill, &rect(x, middle, SWATCH_WIDTH, FILL_HEIGHT));
    }
    let line = segment((x, middle), (x + SWATCH_WIDTH, middle));
    snapshot.append_stroke(&line, &stroke(entry.style, entry.weight), &color);
    let label = with_alpha(&ink.fg, LABEL_ALPHA);
    draw_layout(
        snapshot,
        layout,
        (x + SWATCH_WIDTH + SWATCH_GAP, top),
        &label,
    );
}

fn widths(layouts: &[pango::Layout]) -> Vec<f32> {
    layouts.iter().map(|layout| size_of(layout).0).collect()
}

fn row_height(layouts: &[pango::Layout]) -> f32 {
    layouts
        .iter()
        .map(|layout| size_of(layout).1)
        .fold(0.0, f32::max)
}

fn row_top(index: usize, row: f32) -> f32 {
    f32::from(u8::try_from(index).unwrap_or(u8::MAX)) * (row + ROW_GAP)
}

fn block_height(rows: usize, row: f32) -> f32 {
    (row_top(rows, row) - ROW_GAP).max(0.0)
}
