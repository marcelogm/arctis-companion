mod bus;
mod fixtures;
mod headset;
mod introspection;
mod notification_service;
mod notifications;
mod portal;
mod portal_service;
mod service;
mod window;

use std::env;
use std::panic;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use bus::Bus;

const CHILD: &str = "ARCTIS_SESSION_BUS_CHILD";

type Scenario = (&'static str, fn(&Bus));

const SCENARIOS: &[Scenario] = &[
    (
        "Headset1 matches its contract",
        headset::matches_the_contract,
    ),
    (
        "properties follow the snapshot",
        headset::follows_the_snapshot,
    ),
    (
        "ActivatePreset follows the picked path",
        headset::activate_preset_follows_the_picked_path,
    ),
    (
        "AutomaticPresets is writable",
        headset::automatic_presets_is_writable,
    ),
    (
        "battery alerts are notified then withdrawn",
        notifications::battery_alerts_are_notified_then_withdrawn,
    ),
    (
        "the portal is absent",
        portal::an_absent_portal_counts_as_denied,
    ),
    (
        "the portal grants and denies",
        portal::asks_for_background_with_autostart,
    ),
    (
        "the window slot presents one window",
        window::presents_one_window_with_the_latest_state,
    ),
    (
        "runs windowless as a service",
        service::runs_windowless_as_a_service,
    ),
];

fn main() {
    if env::var_os(CHILD).is_some() {
        run_scenarios();
    } else {
        run_child();
    }
}

fn run_child() {
    let root = tempfile::tempdir().expect("scratch dir");
    let schemas = root.path().join("schemas");
    compile_schema(&schemas);
    let mut child = Command::new(env::current_exe().expect("this test binary"));
    if let Some(display) = absolute_wayland_display() {
        child.env("WAYLAND_DISPLAY", display);
    }
    let status = child
        .env(CHILD, "1")
        .env("GTK_A11Y", "none")
        .env("GSETTINGS_SCHEMA_DIR", &schemas)
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

fn compile_schema(target: &Path) {
    std::fs::create_dir_all(target).expect("schema dir");
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let status = Command::new("glib-compile-schemas")
        .arg("--strict")
        .arg("--targetdir")
        .arg(target)
        .arg(data)
        .status()
        .expect("glib-compile-schemas runs");
    assert!(status.success());
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
