use std::cell::RefCell;
use std::ffi::CString;
use std::rc::Rc;
use std::sync::Once;

use adw::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::companion::selection::{ActivePreset, PresetReason};
use arctis_core::intent::Intent;
use arctis_core::testkit::snapshots::{id, snapshot};
use arctis_core::view::headset_page::{HeadsetModel, HeadsetPageModel, headset_page};
use arctis_ui::Window;
use arctis_ui::intents::IntentSink;
use gtk::glib;
use gtk::glib::translate::{IntoGlib, ToGlibPtr};

static RESOURCES: Once = Once::new();

pub fn init() {
    adw::init().expect("libadwaita initialises");
    RESOURCES.call_once(|| arctis_ui::register_resources().expect("bundled resources register"));
}

pub type Log = Rc<RefCell<Vec<Intent>>>;

pub fn recorder() -> (impl Fn(Intent) + 'static, Log) {
    let log = Log::default();
    let writer = Rc::clone(&log);
    (move |intent| writer.borrow_mut().push(intent), log)
}

pub fn recording_sink() -> (IntentSink, Log) {
    let sink = IntentSink::default();
    let (record, log) = recorder();
    sink.connect(record);
    (sink, log)
}

pub fn has_accessible_text(
    widget: &impl IsA<gtk::Accessible>,
    property: gtk::AccessibleProperty,
    expected: &str,
) -> bool {
    let expected = CString::new(expected).expect("text without NUL");
    let accessible: *mut gtk::ffi::GtkAccessible =
        widget.upcast_ref::<gtk::Accessible>().to_glib_none().0;
    let mismatch = unsafe {
        gtk::ffi::gtk_test_accessible_check_property(
            accessible,
            property.into_glib(),
            expected.as_ptr(),
        )
    };
    let matches = mismatch.is_null();
    if !matches {
        unsafe { glib::ffi::g_free(mismatch.cast()) };
    }
    matches
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
    if let Some(content) = widget
        .downcast_ref::<adw::Dialog>()
        .and_then(adw::Dialog::child)
    {
        found.push(content.clone());
        collect(&content, found);
    }
    let mut child = widget.first_child();
    while let Some(current) = child {
        found.push(current.clone());
        collect(&current, found);
        child = current.next_sibling();
    }
}

pub fn headset_model(snapshot: &Snapshot) -> HeadsetModel {
    match headset_page(snapshot) {
        HeadsetPageModel::Headset(model) => *model,
        HeadsetPageModel::Status(kind, _) => panic!("expected the headset page, got {kind:?}"),
    }
}

pub fn combo_text(row: &adw::ComboRow) -> String {
    row.selected_item()
        .and_downcast::<gtk::StringObject>()
        .map(|item| item.string().to_string())
        .unwrap_or_default()
}

pub fn rendered_window(snapshot: Snapshot) -> (Window, Log) {
    init();
    let window: Window = glib::Object::new();
    let (record, log) = recorder();
    window.on_intent(record);
    window.render(&Rc::new(snapshot));
    (window, log)
}

pub fn shown(widget: &gtk::Widget) -> bool {
    (is_toplevel(widget) || widget.get_visible() && widget.is_child_visible())
        && widget.parent().is_none_or(|parent| shown(&parent))
}

fn is_toplevel(widget: &gtk::Widget) -> bool {
    widget.is::<gtk::Window>() || widget.is::<adw::Dialog>()
}

pub fn rich_snapshot() -> Snapshot {
    let mut s = snapshot();
    s.audio.bluetooth_sink_seen = true;
    s.audio.manager_denied = true;
    s.presets.active.dongle = ActivePreset {
        preset: id("footsteps"),
        reason: PresetReason::App("Firefox".to_owned()),
    };
    s
}

pub fn has_value_text(row: &arctis_ui::ScaleRow, text: &str) -> bool {
    has_accessible_text(
        &first::<gtk::Scale>(row),
        gtk::AccessibleProperty::ValueText,
        text,
    )
}

#[track_caller]
pub fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
}
