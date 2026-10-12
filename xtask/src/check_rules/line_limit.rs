use std::path::Path;

const GUIDELINE: usize = 400;
const UNLIMITED_DIRS: [&str; 1] = ["docs/superpowers/plans/"];
const UNLIMITED_EXTENSIONS: [&str; 1] = ["svg"];

pub fn check(path: &Path, text: &str) -> Option<String> {
    let lines = text.lines().count();
    (lines > GUIDELINE && !is_unlimited(path)).then(|| {
        format!(
            "{}: {lines} lines, the guideline is about {GUIDELINE}; consider splitting it",
            path.display()
        )
    })
}

fn is_unlimited(path: &Path) -> bool {
    UNLIMITED_DIRS.iter().any(|dir| path.starts_with(dir))
        || path.extension().is_some_and(|extension| {
            UNLIMITED_EXTENSIONS
                .iter()
                .any(|unlimited| extension == *unlimited)
        })
}
