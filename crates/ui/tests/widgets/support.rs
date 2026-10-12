use adw::prelude::*;
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

pub fn descendants(root: &impl IsA<gtk::Widget>) -> Vec<gtk::Widget> {
    let mut found = Vec::new();
    collect(root.upcast_ref(), &mut found);
    found
}

pub fn named<T: IsA<gtk::Widget>>(root: &impl IsA<gtk::Widget>, name: &str) -> T {
    descendants(root)
        .into_iter()
        .find(|widget| widget.widget_name() == name)
        .and_then(|widget| widget.downcast::<T>().ok())
        .unwrap_or_else(|| panic!("no widget named {name} of that type"))
}

pub fn first<T: IsA<gtk::Widget>>(root: &impl IsA<gtk::Widget>) -> T {
    descendants(root)
        .into_iter()
        .find_map(|widget| widget.downcast::<T>().ok())
        .expect("a widget of that type")
}

fn collect(widget: &gtk::Widget, found: &mut Vec<gtk::Widget>) {
    let mut child = widget.first_child();
    while let Some(current) = child {
        found.push(current.clone());
        collect(&current, found);
        child = current.next_sibling();
    }
}
