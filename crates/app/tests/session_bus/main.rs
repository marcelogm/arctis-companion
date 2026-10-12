mod bus;
mod fixtures;
mod headset;
mod introspection;
mod notification_service;
mod notifications;
mod portal;
mod portal_service;
#[path = "../common/schema.rs"]
mod schema;
mod service;
mod window;

use std::env;
use std::panic;
use std::path::PathBuf;
use std::process::{self, Command};

use bus::Bus;

const CHILD: &str = "ARCTIS_SESSION_BUS_CHILD";

type Scenario = (&'static str, fn(&Bus));

macro_rules! scenarios {
    ($($scenario:path),* $(,)?) => {
        &[$((stringify!($scenario), $scenario)),*]
    };
}

const SCENARIOS: &[Scenario] = scenarios![
    headset::matches_the_contract,
    headset::follows_the_snapshot,
    headset::activate_preset_follows_the_picked_path,
    headset::automatic_presets_is_writable,
    headset::refuses_unknown_methods_and_mistyped_writes,
    notifications::battery_alerts_are_notified_then_withdrawn,
    portal::an_absent_portal_counts_as_denied,
    portal::asks_for_background_with_autostart,
    window::presents_one_window_with_the_latest_state,
    service::runs_windowless_until_a_second_launch,
];

fn main() {
    if env::var_os(service::SECOND_INSTANCE).is_some() {
        service::run_second_instance();
    } else if env::var_os(CHILD).is_some() {
        run_scenarios();
    } else {
        run_child();
    }
}

fn run_child() {
    let root = tempfile::tempdir().expect("scratch dir");
    let schemas = schema::compiled_dir();
    let mut child = Command::new(env::current_exe().expect("this test binary"));
    if let Some(display) = absolute_wayland_display() {
        child.env("WAYLAND_DISPLAY", display);
    }
    let status = child
        .env(CHILD, "1")
        .env("GTK_A11Y", "none")
        .env("GSETTINGS_SCHEMA_DIR", schemas.path())
        .env("GSETTINGS_BACKEND", "memory")
        .env("XDG_DATA_HOME", root.path().join("data"))
        .status()
        .expect("the scenarios run");
    assert!(status.success(), "a session bus scenario failed");
}

fn absolute_wayland_display() -> Option<PathBuf> {
    let display = PathBuf::from(env::var_os("WAYLAND_DISPLAY")?);
    let runtime = PathBuf::from(env::var_os("XDG_RUNTIME_DIR")?);
    Some(runtime.join(display))
}

fn exit_on_panic() {
    let report = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        report(info);
        process::exit(1);
    }));
}

fn run_scenarios() -> ! {
    exit_on_panic();
    let bus = Bus::start();
    for (name, scenario) in SCENARIOS {
        eprintln!("scenario: {name}");
        scenario(&bus);
    }
    bus.stop();
    eprintln!("{} scenarios passed", SCENARIOS.len());
    process::exit(0)
}
