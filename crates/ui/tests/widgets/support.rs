use std::cell::{Cell, RefCell};
use std::ffi::CString;
use std::rc::Rc;
use std::sync::Once;

use adw::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::companion::selection::{ActivePreset, PresetReason};
use arctis_core::eq::FilterKind;
use arctis_core::eq::preset::Preset;
use arctis_core::intent::Intent;
pub use arctis_core::testkit::snapshots::{custom_preset, filter, with_preset};
use arctis_core::testkit::snapshots::{id, rules, snapshot};
use arctis_core::view::headset_page::{HeadsetModel, HeadsetPageModel, headset_page};
use arctis_ui::intents::IntentSink;
use arctis_ui::preset_files::{Export, PickedText, PresetFiles};
use arctis_ui::{EqBandRow, EqEditorPage, Services, Window};
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

pub fn open_page(window: &Window, tag: &str) {
    WidgetExt::activate_action(window, "win.show-page", Some(&tag.to_variant()))
        .expect("the action exists");
}

pub fn shown(widget: &gtk::Widget) -> bool {
    (is_toplevel(widget) || widget.get_visible() && widget.is_child_visible())
        && (has_floating_parent(widget) || widget.parent().is_none_or(|parent| shown(&parent)))
}

fn has_floating_parent(widget: &gtk::Widget) -> bool {
    let parent = unsafe { gtk::ffi::gtk_widget_get_parent(widget.to_glib_none().0) };
    !parent.is_null() && unsafe { glib::gobject_ffi::g_object_is_floating(parent.cast()) } != 0
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

pub fn ruled(entries: &[(&str, &str)]) -> Snapshot {
    let mut s = snapshot();
    s.presets.rules = rules(entries);
    s
}

pub use arctis_core::testkit::snapshots::seen_app;
#[derive(Default)]
pub struct FakeFiles {
    pub picked: RefCell<Option<PickedText>>,
    pub save_as: RefCell<String>,
    pub saved: RefCell<Vec<String>>,
    pub refuse_writes: Cell<bool>,
}

impl PresetFiles for FakeFiles {
    fn open(&self, _parent: &gtk::Widget, done: Box<dyn FnOnce(PickedText)>) {
        if let Some(picked) = self.picked.take() {
            done(picked);
        }
    }

    fn save(&self, _parent: &gtk::Widget, export: Export) {
        let written = match (export.render)(&self.save_as.borrow()) {
            Some(text) => {
                self.saved.borrow_mut().push(text);
                !self.refuse_writes.get()
            }
            None => false,
        };
        (export.done)(written);
    }
}

pub struct Mounted<P> {
    pub page: P,
    pub log: Log,
    pub files: Rc<FakeFiles>,
    pub toasts: adw::ToastOverlay,
}

pub fn fake_services() -> (Services, Log, Rc<FakeFiles>) {
    init();
    let (sink, log) = recording_sink();
    let files = Rc::new(FakeFiles::default());
    let shared: Rc<dyn PresetFiles> = files.clone();
    let services = Services {
        sink,
        toasts: adw::ToastOverlay::new(),
        files: shared,
    };
    (services, log, files)
}

pub fn mount<P: Default>(attach: impl FnOnce(&P, Services)) -> Mounted<P> {
    let (services, log, files) = fake_services();
    let page = P::default();
    let toasts = services.toasts.clone();
    attach(&page, services);
    Mounted {
        page,
        log,
        files,
        toasts,
    }
}

pub fn standalone_editor(s: Snapshot, preset: &str) -> Mounted<EqEditorPage> {
    let mounted = mount(|page: &EqEditorPage, services| {
        page.set_services(services);
    });
    mounted.page.open(&Rc::new(s), &id(preset));
    mounted
}

pub fn one_peak() -> Preset {
    custom_preset(&[filter(FilterKind::Peaking, 250, 4, 1_000)])
}

pub fn shows_text(root: &impl IsA<gtk::Widget>, text: &str) -> bool {
    descendants(root)
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Label>().ok())
        .any(|label| label.label() == text)
}

pub fn click(root: &impl IsA<gtk::Widget>, label: &str) {
    descendants(root)
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Button>().ok())
        .find(|button| button.label().as_deref() == Some(label))
        .unwrap_or_else(|| panic!("no button labelled {label}"))
        .emit_clicked();
}

pub fn has_button(root: &impl IsA<gtk::Widget>, label: &str) -> bool {
    descendants(root)
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Button>().ok())
        .any(|button| button.label().as_deref() == Some(label))
}

pub fn saved(log: &Log) -> Preset {
    match log.borrow().last() {
        Some(Intent::SavePreset(preset)) => preset.clone(),
        other => panic!("expected SavePreset, got {other:?}"),
    }
}

pub fn band_rows(page: &EqEditorPage) -> Vec<EqBandRow> {
    descendants(page)
        .into_iter()
        .filter_map(|widget| widget.downcast::<EqBandRow>().ok())
        .collect()
}

pub fn step_up(row: &adw::SpinRow) {
    first::<gtk::SpinButton>(row).emit_by_name::<()>("change-value", &[&gtk::ScrollType::StepUp]);
}

pub fn drain() {
    while glib::MainContext::default().iteration(false) {}
}

pub fn dismiss_toast(root: &impl IsA<gtk::Widget>) {
    descendants(root)
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Button>().ok())
        .find(|button| button.icon_name().as_deref() == Some("window-close-symbolic"))
        .expect("a toast with a close button")
        .emit_clicked();
    drain();
}

pub fn open_editor(window: &Window, preset: &str) -> EqEditorPage {
    WidgetExt::activate_action(window, "win.show-preset", Some(&preset.to_variant()))
        .expect("the action exists");
    first::<EqEditorPage>(window)
}

pub fn open_presets(window: &Window) {
    open_page(window, "presets");
}

pub fn visible_tag(window: &Window) -> Option<String> {
    named::<adw::NavigationView>(window, "navigation")
        .visible_page()
        .and_then(|page| page.tag())
        .map(String::from)
}

pub fn count_text(root: &impl IsA<gtk::Widget>, text: &str) -> usize {
    descendants(root)
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Label>().ok())
        .filter(|label| label.label() == text)
        .count()
}
