use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

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
