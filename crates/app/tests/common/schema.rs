use std::path::Path;
use std::process::Command;

use gtk::gio;
use tempfile::TempDir;

pub const APP_ID: &str = "io.github.marcelogm.ArctisCompanion";

pub fn compiled_dir() -> TempDir {
    let dir = tempfile::tempdir().expect("schema dir");
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let status = Command::new("glib-compile-schemas")
        .arg("--strict")
        .arg("--targetdir")
        .arg(dir.path())
        .arg(&data)
        .status()
        .expect("glib-compile-schemas runs");
    assert!(status.success(), "the schema compiles with --strict");
    dir
}

pub fn memory_pair(dir: &Path) -> (gio::Settings, gio::Settings) {
    let source =
        gio::SettingsSchemaSource::from_directory(dir, None, false).expect("schema source");
    let schema = source.lookup(APP_ID, false).expect("app schema");
    let backend = gio::memory_settings_backend_new();
    (
        gio::Settings::new_full(&schema, Some(&backend), None),
        gio::Settings::new_full(&schema, Some(&backend), None),
    )
}
