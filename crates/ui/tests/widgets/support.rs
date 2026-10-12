use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Once;

use arctis_core::intent::Intent;
use arctis_ui::intents::IntentSink;

static RESOURCES: Once = Once::new();

pub fn init() {
    adw::init().expect("libadwaita initialises");
    RESOURCES.call_once(|| arctis_ui::register_resources().expect("bundled resources register"));
}

pub type Log = Rc<RefCell<Vec<Intent>>>;

pub fn recording_sink() -> (IntentSink, Log) {
    let sink = IntentSink::default();
    let log = Log::default();
    let writer = Rc::clone(&log);
    sink.connect(move |intent| writer.borrow_mut().push(intent));
    (sink, log)
}
