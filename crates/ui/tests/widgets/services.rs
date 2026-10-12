use std::cell::Cell;
use std::rc::Rc;

use adw::prelude::*;
use arctis_core::view::text::Text;
use arctis_ui::details_dialog;

use crate::support::{click, fake_services, has_button, init, shows_text};

#[gtk::test]
fn a_toast_shows_its_text() {
    let (services, log, _) = fake_services();
    services.toast(&Text::plain("Imported “Mine”"));
    assert!(shows_text(&services.toasts, "Imported “Mine”"));
    assert!(log.borrow().is_empty());
}

#[gtk::test]
fn an_undo_toast_runs_its_undo_and_hands_back_its_handle() {
    let (services, _, _) = fake_services();
    let undone = Rc::new(Cell::new(false));
    let flag = Rc::clone(&undone);
    let toast = services.undo_toast(&Text::plain("Saved “Mine”"), move || flag.set(true));
    assert_eq!(toast.title().as_deref(), Some("Saved “Mine”"));
    click(&services.toasts, "Undo");
    assert!(undone.get());
}

#[gtk::test]
fn an_urgent_undo_toast_runs_its_undo() {
    let (services, _, _) = fake_services();
    let undone = Rc::new(Cell::new(false));
    let flag = Rc::clone(&undone);
    services.urgent_undo_toast(&Text::plain("Deleted “Mine”"), move || flag.set(true));
    assert!(shows_text(&services.toasts, "Deleted “Mine”"));
    click(&services.toasts, "Undo");
    assert!(undone.get());
}

#[gtk::test]
fn a_toast_with_details_offers_them() {
    let (services, _, _) = fake_services();
    services.toast_with_details(&Text::plain("Imported “Loud”"), vec![Text::plain("Gain")]);
    assert!(has_button(&services.toasts, "Details"));
}

#[gtk::test]
fn the_details_list_one_change_per_line() {
    init();
    let dialog = details_dialog(&[Text::plain("Filter 1: Gain"), Text::plain("Filter 2: Q")]);
    assert_eq!(dialog.body(), "Filter 1: Gain\nFilter 2: Q");
}
