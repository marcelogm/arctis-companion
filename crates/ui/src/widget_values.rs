use adw::prelude::*;
use arctis_core::view::text::Text;

use crate::text::{optional, resolve};

#[must_use]
pub fn nearest_step(value: f64, max: u8) -> u8 {
    (0..=max)
        .min_by(|a, b| distance(*a, value).total_cmp(&distance(*b, value)))
        .unwrap_or(0)
}

#[must_use]
pub fn index(position: u32) -> Option<usize> {
    (position != gtk::INVALID_LIST_POSITION)
        .then(|| usize::try_from(position).ok())
        .flatten()
}

#[must_use]
pub fn chosen<T: Clone>(choices: &[T], position: u32) -> Option<T> {
    choices.get(index(position)?).cloned()
}

#[must_use]
pub fn position(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(gtk::INVALID_LIST_POSITION)
}

pub fn show_if<W: IsA<gtk::Widget>, M>(
    widget: &W,
    model: Option<M>,
    fill: impl FnOnce(&W, M),
) -> bool {
    widget.set_visible(model.is_some());
    let shown = model.is_some();
    if let Some(model) = model {
        fill(widget, model);
    }
    shown
}

pub fn show_choices(row: &adw::ComboRow, items: &[String], selected: usize) {
    if !shows(row, items) {
        let names: Vec<&str> = items.iter().map(String::as_str).collect();
        row.set_model(Some(&gtk::StringList::new(&names)));
    }
    row.set_selected(position(selected));
}

pub fn show_combo_row<'a>(
    row: &adw::ComboRow,
    (title, subtitle): (&Text, Option<&Text>),
    items: impl Iterator<Item = &'a Text>,
    selected: usize,
) {
    row.set_title(&resolve(title));
    row.set_subtitle(&optional(subtitle));
    let items: Vec<String> = items.map(resolve).collect();
    show_choices(row, &items, selected);
}

pub fn show_label(label: &gtk::Label, text: Option<&Text>) {
    show_if(label, text, |label, text| label.set_label(&resolve(text)));
}

fn distance(step: u8, value: f64) -> f64 {
    (f64::from(step) - value).abs()
}

fn shows(row: &adw::ComboRow, items: &[String]) -> bool {
    row.model()
        .and_downcast::<gtk::StringList>()
        .is_some_and(|list| same_items(&list, items))
}

fn same_items(list: &gtk::StringList, items: &[String]) -> bool {
    position(items.len()) == list.n_items()
        && (0..list.n_items())
            .zip(items)
            .all(|(at, item)| list.string(at).as_deref() == Some(item.as_str()))
}
