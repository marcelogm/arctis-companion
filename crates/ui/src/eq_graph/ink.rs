use adw::prelude::*;
use arctis_core::view::eq_graph::{ColorRole, LineStyle, Weight};
use gtk::{gdk, gsk};

use crate::eq_handle::with_alpha;

const FILL_ALPHA: f32 = 0.18;
const CURVE_WIDTH: f32 = 2.0;
const BOLD_WIDTH: f32 = 2.5;
const FAINT_WIDTH: f32 = 1.5;
const FAINT_ALPHA: f32 = 0.45;
const DASH: [f32; 2] = [6.0, 4.0];

pub struct Ink {
    pub fg: gdk::RGBA,
    accent: gdk::RGBA,
}

impl Ink {
    pub fn of(widget: &impl IsA<gtk::Widget>) -> Self {
        Self {
            fg: widget.color(),
            accent: adw::StyleManager::default().accent_color_rgba(),
        }
    }

    pub fn fill(&self, role: ColorRole) -> gdk::RGBA {
        with_alpha(&self.base(role), FILL_ALPHA)
    }

    pub fn line(&self, role: ColorRole, weight: Weight) -> gdk::RGBA {
        let color = self.base(role);
        match weight {
            Weight::Faint => with_alpha(&color, FAINT_ALPHA),
            Weight::Regular | Weight::Bold => color,
        }
    }

    const fn base(&self, role: ColorRole) -> gdk::RGBA {
        match role {
            ColorRole::Accent => self.accent,
            ColorRole::Foreground => self.fg,
        }
    }
}

pub fn stroke(style: LineStyle, weight: Weight) -> gsk::Stroke {
    let stroke = gsk::Stroke::new(match weight {
        Weight::Regular => CURVE_WIDTH,
        Weight::Faint => FAINT_WIDTH,
        Weight::Bold => BOLD_WIDTH,
    });
    if style == LineStyle::Dashed {
        stroke.set_dash(&DASH);
    }
    stroke
}
