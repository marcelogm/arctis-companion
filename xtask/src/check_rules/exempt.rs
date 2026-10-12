use std::path::Path;

const GENERATED_NAMES: [&str; 4] = [
    "Cargo.lock",
    "cargo-sources.json",
    "package-lock.json",
    "LINGUAS",
];
const GENERATED_EXTENSIONS: [&str; 2] = ["po", "pot"];
const VERBATIM_NAMES: [&str; 1] = ["LICENSE"];
const VERBATIM_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "gif", "webp", "ico"];

pub fn is_exempt(path: &Path) -> bool {
    is_generated(path) || is_verbatim(path)
}

fn is_generated(path: &Path) -> bool {
    let name = name_of(path);
    GENERATED_NAMES.contains(&name)
        || name.starts_with("POTFILES")
        || GENERATED_EXTENSIONS.contains(&extension_of(path))
}

fn is_verbatim(path: &Path) -> bool {
    VERBATIM_NAMES.contains(&name_of(path)) || VERBATIM_EXTENSIONS.contains(&extension_of(path))
}

fn name_of(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
}

fn extension_of(path: &Path) -> &str {
    path.extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
}
