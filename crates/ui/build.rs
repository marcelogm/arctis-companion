use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const METAINFO: &str = "../../data/io.github.marcelogm.ArctisCompanion.metainfo.xml.in";
const PO_DIR: &str = "../../po";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=resources");
    println!("cargo::rerun-if-changed={METAINFO}");
    println!("cargo::rerun-if-changed={PO_DIR}");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    compile_blueprints(&out_dir)?;
    merge_metainfo(&out_dir)?;
    glib_build_tools::compile_resources(
        &[
            out_dir.as_path(),
            Path::new("resources"),
            Path::new("../../data"),
        ],
        "resources/resources.gresource.xml",
        "arctis-ui.gresource",
    );
    Ok(())
}

fn compile_blueprints(out_dir: &Path) -> Result<(), Box<dyn Error>> {
    let status = Command::new("blueprint-compiler")
        .arg("batch-compile")
        .arg(out_dir)
        .arg("resources")
        .args(blueprint_files()?)
        .status()
        .map_err(|error| format!("blueprint-compiler: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("blueprint-compiler failed".into())
    }
}

fn merge_metainfo(out_dir: &Path) -> Result<(), Box<dyn Error>> {
    let merged = out_dir.join("io.github.marcelogm.ArctisCompanion.metainfo.xml");
    let status = Command::new("msgfmt")
        .arg("--xml")
        .arg(format!("--template={METAINFO}"))
        .args(["-d", PO_DIR, "-o"])
        .arg(merged)
        .status()
        .map_err(|error| format!("msgfmt: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("msgfmt --xml failed on the metainfo".into())
    }
}

fn blueprint_files() -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files: Vec<PathBuf> = fs::read_dir("resources")?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "blp"))
        .collect();
    files.sort();
    Ok(files)
}
