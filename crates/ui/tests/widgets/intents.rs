use arctis_core::intent::Intent;
use arctis_ui::intents::IntentSink;

use crate::support::{recorder, recording_sink};

#[test]
fn an_emitted_intent_reaches_the_handler() {
    let (sink, log) = recording_sink();
    sink.emit(Intent::RetryWrite);
    assert_eq!(*log.borrow(), vec![Intent::RetryWrite]);
}

#[test]
fn intents_emitted_while_rendering_are_dropped() {
    let (sink, log) = recording_sink();
    sink.rendering(|| sink.emit(Intent::RetryWrite));
    assert!(log.borrow().is_empty());
}

#[test]
fn nested_renders_restore_the_outer_state() {
    let (sink, log) = recording_sink();
    sink.rendering(|| sink.rendering(|| ()));
    sink.emit(Intent::RetryWrite);
    assert_eq!(log.borrow().len(), 1);
}

#[test]
fn emit_some_skips_none() {
    let (sink, log) = recording_sink();
    sink.emit_some(None);
    sink.emit_some(Some(Intent::EndPreview));
    assert_eq!(*log.borrow(), vec![Intent::EndPreview]);
}

#[test]
fn a_handler_may_reconnect_while_handling() {
    let sink = IntentSink::default();
    let (record, log) = recorder();
    let inner = sink.clone();
    sink.connect(move |intent| {
        let (nested, _) = recorder();
        inner.connect(nested);
        inner.rendering(|| ());
        record(intent);
    });
    sink.emit(Intent::RetryWrite);
    sink.emit(Intent::EndPreview);
    assert_eq!(*log.borrow(), vec![Intent::RetryWrite]);
}

#[test]
fn clones_share_the_handler() {
    let (sink, log) = recording_sink();
    sink.clone().emit(Intent::RetryWrite);
    assert_eq!(log.borrow().len(), 1);
}
