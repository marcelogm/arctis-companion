use std::cell::Cell;
use std::collections::HashMap;
use std::env;
use std::process;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use adw::prelude::*;
use arctis_app::application::{self, APP_ID};
use arctis_app::dbus::headset_object::INTERFACE;
use arctis_app::threads::Threads;
use gtk::gio;
use gtk::glib;

use crate::bus::{Bus, Target, call_async};
use crate::fixtures::{PATH, silent_threads};

pub const SECOND_INSTANCE: &str = "ARCTIS_SECOND_INSTANCE";
const SETTLE: Duration = Duration::from_millis(500);
const SECOND_LAUNCH_PATIENCE: Duration = Duration::from_secs(10);
const STARTED_THREADS: i32 = 3;

static SECOND_STARTED_THREADS: AtomicBool = AtomicBool::new(false);

const fn application_at(interface: &str) -> Target<'_> {
    Target {
        name: APP_ID,
        path: PATH,
        interface,
    }
}

fn no_platform_data() -> HashMap<String, glib::Variant> {
    HashMap::new()
}

async fn headset_state(client: &gio::DBusConnection) -> Option<String> {
    let reply = call_async(
        client,
        &application_at("org.freedesktop.DBus.Properties"),
        "Get",
        Some(&(INTERFACE, "State").to_variant()),
    )
    .await
    .expect("the application answers");
    let (state,) = reply.get::<(glib::Variant,)>().expect("(v)");
    state.get::<String>()
}

fn record_thread_start() -> Threads {
    SECOND_STARTED_THREADS.store(true, Ordering::SeqCst);
    silent_threads()
}

pub fn run_second_instance() -> ! {
    let application = application::build(|| Ok(record_thread_start()));
    let code = application.run_with_args(&["arctis-companion"]);
    if SECOND_STARTED_THREADS.load(Ordering::SeqCst) {
        process::exit(STARTED_THREADS);
    }
    process::exit(i32::from(code.get()))
}

async fn launch_second_instance() -> i32 {
    let launcher = gio::SubprocessLauncher::new(gio::SubprocessFlags::NONE);
    launcher.setenv(SECOND_INSTANCE, "1", true);
    let program = env::current_exe().expect("this test binary");
    let second = launcher
        .spawn(&[program.as_os_str()])
        .expect("the second instance starts");
    let waited = glib::future_with_timeout(SECOND_LAUNCH_PATIENCE, second.wait_future()).await;
    if !matches!(waited, Ok(Ok(()))) {
        second.force_exit();
        panic!("the second instance did not return");
    }
    second.exit_status()
}

fn quit(client: &gio::DBusConnection) {
    let quit = ("quit", Vec::<glib::Variant>::new(), no_platform_data()).to_variant();
    let target = application_at("org.freedesktop.Application");
    client.call(
        Some(target.name),
        target.path,
        target.interface,
        "ActivateAction",
        Some(&quit),
        None,
        gio::DBusCallFlags::NONE,
        -1,
        None::<&gio::Cancellable>,
        |_| {},
    );
}

async fn assert_windowless(
    application: &application::CompanionApplication,
    client: &gio::DBusConnection,
) {
    glib::timeout_future(SETTLE).await;
    assert!(
        application.windows().is_empty(),
        "no window before activation"
    );
    assert_eq!(headset_state(client).await.as_deref(), Some("absent"));
}

async fn assert_a_second_launch_activates(application: &application::CompanionApplication) {
    assert_eq!(
        launch_second_instance().await,
        0,
        "a second launch returns without starting threads"
    );
    glib::timeout_future(SETTLE).await;
    assert_eq!(
        application.windows().len(),
        1,
        "a second launch opens the window"
    );
}

async fn probe(
    application: application::CompanionApplication,
    client: gio::DBusConnection,
    probed: Rc<Cell<bool>>,
) {
    assert_windowless(&application, &client).await;
    assert_a_second_launch_activates(&application).await;
    probed.set(true);
    quit(&client);
}

fn asked_for_background_before() {
    let settings = gio::Settings::new(APP_ID);
    settings
        .set_boolean("background-requested", true)
        .expect("background-requested");
    gio::Settings::sync();
}

pub fn runs_windowless_until_a_second_launch(bus: &Bus) {
    asked_for_background_before();
    let application = application::build(|| Ok(silent_threads()));
    let probed = Rc::new(Cell::new(false));
    glib::spawn_future_local(probe(application.clone(), bus.client(), Rc::clone(&probed)));
    let code = application.run_with_args(&["arctis-companion", "--gapplication-service"]);
    assert_eq!(code, glib::ExitCode::SUCCESS);
    assert!(probed.get(), "the probe ran inside the service");
}
