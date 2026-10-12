use arctis_core::intent::Intent;
use arctis_core::view::headset_page::ToastModel;
use arctis_core::view::text::Text;
use arctis_ui::ToastFeed;
use gtk::prelude::ObjectExt;

use crate::support::{init, recording_sink};

fn failure(serial: u32) -> ToastModel {
    ToastModel {
        serial,
        title: Text::plain("Could Not Change Sidetone"),
        retry: Intent::RetryWrite,
    }
}

#[gtk::test]
fn a_new_failure_makes_a_toast_with_retry() {
    init();
    let (sink, _) = recording_sink();
    let toast = ToastFeed::default()
        .next(Some(&failure(1)), &sink)
        .expect("a toast");
    assert_eq!(toast.title().as_deref(), Some("Could Not Change Sidetone"));
    assert_eq!(toast.button_label().as_deref(), Some("Retry"));
}

#[gtk::test]
fn the_same_failure_is_shown_once() {
    init();
    let (sink, _) = recording_sink();
    let feed = ToastFeed::default();
    assert!(feed.next(Some(&failure(1)), &sink).is_some());
    assert!(feed.next(Some(&failure(1)), &sink).is_none());
    assert!(feed.next(None, &sink).is_none());
    assert!(feed.next(Some(&failure(2)), &sink).is_some());
}

#[gtk::test]
fn retry_emits_the_retry_intent() {
    init();
    let (sink, log) = recording_sink();
    let toast = ToastFeed::default()
        .next(Some(&failure(1)), &sink)
        .expect("a toast");
    toast.emit_by_name::<()>("button-clicked", &[]);
    assert_eq!(*log.borrow(), vec![Intent::RetryWrite]);
}
