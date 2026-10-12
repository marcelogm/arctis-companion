use super::eq_graph::{ColorRole, LineStyle, Series, Weight};
use super::text::Text;
use crate::eq::reference::SeriesId;

pub const SWATCH_WIDTH: f32 = 20.0;
pub const SWATCH_GAP: f32 = 6.0;
pub const ENTRY_GAP: f32 = 16.0;
pub const MAX_ROWS: usize = 2;

#[derive(Clone, Debug, PartialEq)]
pub struct LegendEntry {
    pub label: Text,
    pub style: LineStyle,
    pub role: ColorRole,
    pub weight: Weight,
    pub filled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub x: f32,
    pub row: usize,
}

#[must_use]
pub fn legend(series: &[Series]) -> Vec<LegendEntry> {
    let mut shown: Vec<&Series> = series.iter().filter(|one| one.label.is_some()).collect();
    shown.sort_by_key(|one| rank(one.reference));
    shown.into_iter().filter_map(entry).collect()
}

#[must_use]
pub fn entry_width(label_width: f32) -> f32 {
    SWATCH_WIDTH + SWATCH_GAP + label_width
}

#[must_use]
pub fn natural_width(label_widths: &[f32]) -> f32 {
    let spaced: f32 = label_widths
        .iter()
        .map(|width| entry_width(*width) + ENTRY_GAP)
        .sum();
    (spaced - ENTRY_GAP).max(0.0)
}

#[must_use]
pub fn slots(label_widths: &[f32], available: f32) -> Option<Vec<Slot>> {
    let mut flow = Flow::default();
    let placed: Option<Vec<Slot>> = label_widths
        .iter()
        .map(|width| flow.place(entry_width(*width), available))
        .collect();
    placed.filter(|slots| slots.iter().all(|slot| slot.row < MAX_ROWS))
}

#[must_use]
pub fn rows(slots: &[Slot]) -> usize {
    slots.last().map_or(0, |slot| slot.row + 1)
}

#[derive(Default)]
struct Flow {
    x: f32,
    row: usize,
}

impl Flow {
    fn place(&mut self, width: f32, available: f32) -> Option<Slot> {
        if width > available {
            return None;
        }
        if self.x > 0.0 && self.x + width > available {
            self.x = 0.0;
            self.row += 1;
        }
        let slot = Slot {
            x: self.x,
            row: self.row,
        };
        self.x += width + ENTRY_GAP;
        Some(slot)
    }
}

const fn rank(reference: Option<SeriesId>) -> u8 {
    match reference {
        Some(SeriesId::Headset) => 0,
        Some(SeriesId::HeadsetWithEq) => 1,
        Some(SeriesId::Target(_)) => 2,
        None => 3,
    }
}

fn entry(series: &Series) -> Option<LegendEntry> {
    Some(LegendEntry {
        label: series.label.clone()?,
        style: series.style,
        role: series.role,
        weight: series.weight,
        filled: series.filled,
    })
}
