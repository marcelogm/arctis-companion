use std::cell::Cell;
use std::rc::Rc;

use adw::prelude::*;
use arctis_app::platform::background::BackgroundView;
use arctis_app::ports::SnapshotView;
use arctis_app::window_slot::WindowSlot;
use arctis_core::testkit::snapshots::snapshot;
use arctis_core::view::preferences::Background;
use arctis_ui::Window;
use gtk::gio;
use gtk::glib;

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

fn closing_decision(keep_running: &Rc<Cell<bool>>) -> Rc<dyn Fn() -> glib::Propagation> {
    let decision = Rc::clone(keep_running);
    Rc::new(move || {
        if decision.get() {
            glib::Propagation::Proceed
        } else {
            glib::Propagation::Stop
        }
    })
}

fn close_and_settle(window: &Window) {
    window.close();
    settle();
}

pub fn presents_one_window_with_the_latest_state(_bus: &Bus) {
    let application = application();
    let intents = Intents::default();
    let keep_running = Rc::new(Cell::new(false));
    let slot = WindowSlot::new(Rc::new(recorder(&intents)), closing_decision(&keep_running));
    slot.publish(&Rc::new(snapshot()));
    slot.show_background(Background::Denied);
    slot.present(&application);
    slot.present(&application);
    assert_eq!(application.windows().len(), 1, "one window");
    let window = slot.window().expect("a window");
    hides_then_reuses(&slot, &application, &window);
    keep_running.set(true);
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
