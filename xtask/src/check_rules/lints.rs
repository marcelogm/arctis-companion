use std::fs;
use std::path::Path;

use crate::report::Report;

const GUARDED: [&str; 5] = [
    "missing_errors_doc",
    "missing_panics_doc",
    "missing_safety_doc",
    "undocumented_unsafe_blocks",
    "doc_markdown",
];

pub fn check(root: &Path, files: &[String]) -> Report {
    let members = files.iter().filter(|file| file.ends_with("/Cargo.toml"));
    members
        .map(|file| check_member(root, file))
        .fold(check_root(root), Report::merged)
}

fn check_root(root: &Path) -> Report {
    match read_manifest(&root.join("Cargo.toml")) {
        Some(Ok(manifest)) => check_workspace(&manifest),
        Some(Err(error)) => Report::failed(format!("Cargo.toml: {error}")),
        None => Report::default(),
    }
}

fn check_member(root: &Path, file: &str) -> Report {
    match read_manifest(&root.join(file)) {
        Some(Ok(manifest)) if inherits_lints(&manifest) => Report::default(),
        Some(Ok(_)) => Report::failed(format!(
            "{file}: add [lints] workspace = true so the lint policy applies"
        )),
        Some(Err(error)) => Report::failed(format!("{file}: {error}")),
        None => Report::default(),
    }
}

fn read_manifest(path: &Path) -> Option<Result<toml::Table, toml::de::Error>> {
    fs::read_to_string(path)
        .ok()
        .map(|text| text.parse::<toml::Table>())
}

fn inherits_lints(manifest: &toml::Table) -> bool {
    let workspace = manifest
        .get("lints")
        .and_then(|lints| lints.get("workspace"));
    !manifest.contains_key("package") || workspace.and_then(toml::Value::as_bool) == Some(true)
}

fn check_workspace(manifest: &toml::Table) -> Report {
    let Some(workspace) = manifest.get("workspace") else {
        return Report::default();
    };
    let clippy = workspace.get("lints").and_then(|lints| lints.get("clippy"));
    let missing = GUARDED.iter().filter(|lint| !is_allowed(clippy, lint));
    Report::errors(missing.map(|lint| {
        format!("Cargo.toml: [workspace.lints.clippy] must keep {lint} = \"allow\"; it would demand doc comments")
    }))
}

fn is_allowed(clippy: Option<&toml::Value>, lint: &str) -> bool {
    clippy.and_then(|table| table.get(lint)).and_then(level_of) == Some("allow")
}

fn level_of(value: &toml::Value) -> Option<&str> {
    value
        .as_str()
        .or_else(|| value.get("level").and_then(toml::Value::as_str))
}
