use std::cell::Cell;
use std::rc::Rc;

use adw::prelude::*;
use arctis_app::platform::background::BackgroundView;
use arctis_app::ports::SnapshotView;
use arctis_app::window_slot::{OnClose, WindowSlot};
use arctis_core::testkit::snapshots::snapshot;
use arctis_core::view::preferences::Background;
use arctis_ui::Window;
use gtk::gio;

use crate::bus::{Bus, settle};
use crate::fixtures::{Intents, recorder};

const TEST_APP_ID: &str = "io.github.marcelogm.ArctisCompanion.WindowTest";

fn application() -> adw::Application {
    adw::init().expect("libadwaita initialises");
    arctis_ui::register_resources().expect("bundled resources register");
    let application = adw::Application::builder()
        .application_id(TEST_APP_ID)
        .build();
    application
        .register(None::<&gio::Cancellable>)
        .expect("registered");
    application
}

fn closing_decision(decision: &Rc<Cell<OnClose>>) -> Rc<dyn Fn() -> OnClose> {
    let decision = Rc::clone(decision);
    Rc::new(move || decision.get())
}

fn descendants(root: &gtk::Widget) -> Vec<gtk::Widget> {
    let mut found = Vec::new();
    let mut pending = vec![root.clone()];
    while let Some(widget) = pending.pop() {
        if let Some(content) = widget
            .downcast_ref::<adw::Dialog>()
            .and_then(adw::Dialog::child)
        {
            pending.push(content);
        }
        let mut child = widget.first_child();
        while let Some(current) = child {
            child = current.next_sibling();
            pending.push(current.clone());
            found.push(current);
        }
    }
    found
}

fn named<T: IsA<gtk::Widget>>(root: &impl IsA<gtk::Widget>, name: &str) -> T {
    descendants(root.upcast_ref())
        .into_iter()
        .find(|widget| widget.widget_name() == name)
        .and_then(|widget| widget.downcast::<T>().ok())
        .unwrap_or_else(|| panic!("no widget named {name} of that type"))
}

fn assert_shows_the_replayed_state(window: &Window) {
    let stack = named::<gtk::Stack>(window, "main-stack");
    assert_ne!(
        stack.visible_child_name().as_deref(),
        Some("status"),
        "the connected snapshot shows the headset page"
    );
    let switch = named::<adw::SwitchRow>(&window.preferences_dialog(), "run-in-background");
    assert!(!switch.is_active(), "the denied background state");
    assert_eq!(
        switch.subtitle().as_deref(),
        Some("Allow it in Settings → Apps → Arctis Companion")
    );
}

fn close_and_settle(window: &Window) {
    window.close();
    settle();
}

pub fn presents_one_window_with_the_latest_state(_bus: &Bus) {
    let application = application();
    let intents = Intents::default();
    let decision = Rc::new(Cell::new(OnClose::HideUntilQuit));
    let slot = WindowSlot::new(recorder(&intents), closing_decision(&decision));
    slot.publish(&Rc::new(snapshot()));
    slot.show_background(Background::Denied);
    slot.present(&application);
    slot.present(&application);
    assert_eq!(application.windows().len(), 1, "one window");
    let window = slot.window().expect("a window");
    assert_shows_the_replayed_state(&window);
    hides_then_reuses(&slot, &application, &window);
    decision.set(OnClose::ForgetWindow);
    close_and_settle(&window);
    assert!(
        slot.window().is_none(),
        "closing with Run in Background on forgets the window"
    );
    assert!(intents.borrow().is_empty(), "rendering emits no intent");
}

fn hides_then_reuses(slot: &Rc<WindowSlot>, application: &adw::Application, window: &Window) {
    close_and_settle(window);
    assert!(!window.is_visible(), "quitting hides the window");
    assert!(slot.window().is_some(), "and keeps it until the quit ends");
    slot.present(application);
    assert!(
        window.is_visible(),
        "presenting again shows the kept window"
    );
    assert_eq!(application.windows().len(), 1, "still one window");
}
