use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::report::Report;

const CORE: &str = "arctis-core";
const ALLOWED: [&str; 2] = ["serde", "serde_json"];

pub fn run(root: &Path) -> Report {
    match metadata(root) {
        Ok(metadata) => check_metadata(&metadata),
        Err(message) => Report::failed(message),
    }
}

fn metadata(root: &Path) -> Result<Value, String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let output = Command::new(cargo)
        .args([
            "metadata",
            "--no-deps",
            "--offline",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(root.join("Cargo.toml"))
        .output()
        .map_err(|error| format!("cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|error| format!("cargo metadata: {error}"))
}

fn check_metadata(metadata: &Value) -> Report {
    let mut packages = metadata["packages"].as_array().into_iter().flatten();
    let Some(core) = packages.find(|package| package["name"] == CORE) else {
        return Report::failed(format!("package {CORE} not found"));
    };
    Report::from_errors(forbidden_dependencies(core).map(|name| {
        format!(
            "{CORE} may depend only on {} (found {name})",
            ALLOWED.join(" and ")
        )
    }))
}

fn forbidden_dependencies(core: &Value) -> impl Iterator<Item = String> + '_ {
    core["dependencies"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|dependency| dependency["kind"] != "dev")
        .filter_map(|dependency| dependency["name"].as_str())
        .filter(|name| !ALLOWED.contains(name))
        .map(str::to_owned)
}
