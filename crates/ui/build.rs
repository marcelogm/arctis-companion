use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=resources");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    compile_blueprints(&out_dir);
    glib_build_tools::compile_resources(
        &[out_dir.as_path(), Path::new("resources")],
        "resources/resources.gresource.xml",
        "arctis-ui.gresource",
    );
}

fn compile_blueprints(out_dir: &Path) {
    let status = Command::new("blueprint-compiler")
        .arg("batch-compile")
        .arg(out_dir)
        .arg("resources")
        .args(blueprint_files())
        .status()
        .expect("blueprint-compiler is installed");
    assert!(status.success(), "blueprint-compiler failed");
}

fn blueprint_files() -> Vec<PathBuf> {
    let entries = fs::read_dir("resources").expect("resources directory exists");
    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "blp"))
        .collect();
    files.sort();
    files
}
