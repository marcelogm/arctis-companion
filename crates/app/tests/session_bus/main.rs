mod bus;
mod fixtures;
mod headset;
mod introspection;

use std::env;
use std::path::Path;
use std::process::Command;

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
    let status = Command::new(env::current_exe().expect("this test binary"))
        .env(CHILD, "1")
        .env("GSETTINGS_SCHEMA_DIR", &schemas)
        .env("GSETTINGS_BACKEND", "memory")
        .env("XDG_DATA_HOME", root.path().join("data"))
        .status()
        .expect("the scenarios run");
    assert!(status.success(), "a session bus scenario failed");
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

fn run_scenarios() {
    let bus = Bus::start();
    for (name, scenario) in SCENARIOS {
        eprintln!("scenario: {name}");
        scenario(&bus);
    }
    bus.stop();
    eprintln!("{} scenarios passed", SCENARIOS.len());
}
