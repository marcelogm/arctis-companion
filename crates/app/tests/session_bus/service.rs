use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use arctis_app::application::{self, APP_ID};
use gtk::gio;
use gtk::glib;

use crate::bus::Bus;
use crate::fixtures::{INTERFACE, PATH, silent_threads};

const SETTLE: Duration = Duration::from_millis(500);
const CALL_TIMEOUT_MS: i32 = 5_000;

async fn call(
    client: &gio::DBusConnection,
    interface: &str,
    method: &str,
    parameters: glib::Variant,
) -> glib::Variant {
    client
        .call_future(
            Some(APP_ID),
            PATH,
            interface,
            method,
            Some(&parameters),
            None,
            gio::DBusCallFlags::NONE,
            CALL_TIMEOUT_MS,
        )
        .await
        .expect("the application answers")
}

fn no_platform_data() -> HashMap<String, glib::Variant> {
    HashMap::new()
}

async fn probe(
    application: application::CompanionApplication,
    client: gio::DBusConnection,
    probed: Rc<Cell<bool>>,
) {
    glib::timeout_future(SETTLE).await;
    assert!(
        application.windows().is_empty(),
        "no window before activation"
    );
    let state = call(
        &client,
        "org.freedesktop.DBus.Properties",
        "Get",
        (INTERFACE, "State").to_variant(),
    )
    .await;
    let (state,) = state.get::<(glib::Variant,)>().expect("(v)");
    assert_eq!(state.get::<String>().as_deref(), Some("absent"));
    let _ = call(
        &client,
        "org.freedesktop.Application",
        "Activate",
        (no_platform_data(),).to_variant(),
    )
    .await;
    glib::timeout_future(SETTLE).await;
    assert_eq!(application.windows().len(), 1, "Activate opens the window");
    probed.set(true);
    let quit = ("quit", Vec::<glib::Variant>::new(), no_platform_data()).to_variant();
    client.call(
        Some(APP_ID),
        PATH,
        "org.freedesktop.Application",
        "ActivateAction",
        Some(&quit),
        None,
        gio::DBusCallFlags::NONE,
        CALL_TIMEOUT_MS,
        None::<&gio::Cancellable>,
        |_| {},
    );
}

fn asked_for_background_before() {
    let settings = gio::Settings::new(APP_ID);
    settings
        .set_boolean("background-requested", true)
        .expect("background-requested");
    gio::Settings::sync();
}

pub fn runs_windowless_as_a_service(bus: &Bus) {
    asked_for_background_before();
    let application = application::build(|| Ok(silent_threads()));
    let probed = Rc::new(Cell::new(false));
    glib::spawn_future_local(probe(application.clone(), bus.client(), Rc::clone(&probed)));
    let code = application.run_with_args(&["arctis-companion", "--gapplication-service"]);
    assert_eq!(code, glib::ExitCode::SUCCESS);
    assert!(probed.get(), "the probe ran inside the service");
}
