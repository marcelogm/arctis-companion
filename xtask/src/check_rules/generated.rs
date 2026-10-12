use std::path::Path;

const NAMES: [&str; 5] = [
    "Cargo.lock",
    "cargo-sources.json",
    "package-lock.json",
    "LINGUAS",
    "LICENSE",
];
const EXTENSIONS: [&str; 4] = ["po", "pot", "png", "svg"];

pub fn is_generated(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    NAMES.contains(&name) || name.starts_with("POTFILES") || EXTENSIONS.contains(&extension)
}
