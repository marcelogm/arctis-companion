use adw::prelude::*;
use arctis_core::view::text::Text;
use arctis_ui::widget_values::{index, nearest_step, position, show_choices, show_label};

use crate::support::init;

fn items(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[test]
fn nearest_step_rounds_and_clamps() {
    assert_eq!(nearest_step(5.6, 10), 6);
    assert_eq!(nearest_step(5.4, 10), 5);
    assert_eq!(nearest_step(-3.0, 10), 0);
    assert_eq!(nearest_step(42.0, 15), 15);
}

#[test]
fn list_positions_convert_both_ways() {
    assert_eq!(index(3), Some(3));
    assert_eq!(index(gtk::INVALID_LIST_POSITION), None);
    assert_eq!(position(2), 2);
}

#[gtk::test]
fn show_choices_fills_and_selects() {
    init();
    let row = adw::ComboRow::new();
    show_choices(&row, &items(&["Never", "1 minute"]), 1);
    let list = row
        .model()
        .and_downcast::<gtk::StringList>()
        .expect("a string list");
    assert_eq!(list.string(1).as_deref(), Some("1 minute"));
    assert_eq!(row.selected(), 1);
}

#[gtk::test]
fn show_choices_keeps_the_model_when_items_are_unchanged() {
    init();
    let row = adw::ComboRow::new();
    show_choices(&row, &items(&["Flat", "Bass Boost"]), 0);
    let before = row.model();
    show_choices(&row, &items(&["Flat", "Bass Boost"]), 1);
    assert_eq!(row.model(), before);
    show_choices(&row, &items(&["Flat"]), 0);
    assert_ne!(row.model(), before);
}

#[gtk::test]
fn show_label_hides_a_missing_text() {
    init();
    let label = gtk::Label::new(Some("old"));
    show_label(&label, None);
    assert!(!label.get_visible());
    show_label(&label, Some(&Text::plain("Not applied")));
    assert!(label.get_visible());
    assert_eq!(label.label(), "Not applied");
}
