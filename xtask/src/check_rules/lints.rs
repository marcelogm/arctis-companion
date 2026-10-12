use super::snapshot::Snapshot;
use crate::report::Report;

const ROOT_MANIFEST: &str = "Cargo.toml";
const GUARDED: [&str; 5] = [
    "missing_errors_doc",
    "missing_panics_doc",
    "missing_safety_doc",
    "undocumented_unsafe_blocks",
    "doc_markdown",
];

pub fn check(snapshot: Snapshot, files: &[String]) -> Report {
    let members = files.iter().filter(|file| file.ends_with("/Cargo.toml"));
    members
        .map(|file| with_manifest(snapshot, file, |manifest| check_member(file, manifest)))
        .fold(
            with_manifest(snapshot, ROOT_MANIFEST, check_workspace),
            Report::merged,
        )
}

fn with_manifest(
    snapshot: Snapshot,
    file: &str,
    policy: impl FnOnce(&toml::Table) -> Report,
) -> Report {
    let Some(text) = snapshot.read(file) else {
        return Report::default();
    };
    match text.parse::<toml::Table>() {
        Ok(manifest) => policy(&manifest),
        Err(error) => Report::failed(format!("{file}: {error}")),
    }
}

fn check_member(file: &str, manifest: &toml::Table) -> Report {
    if inherits_lints(manifest) {
        return Report::default();
    }
    Report::failed(format!(
        "{file}: add [lints] workspace = true so the lint policy applies"
    ))
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
    Report::from_errors(missing.map(|lint| {
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
