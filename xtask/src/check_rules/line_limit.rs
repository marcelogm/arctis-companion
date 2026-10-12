use std::path::Path;

const GUIDELINE: usize = 400;

pub fn check(path: &Path, text: &str) -> Option<String> {
    let lines = text.lines().count();
    (lines > GUIDELINE).then(|| {
        format!(
            "{}: {lines} lines, the guideline is about {GUIDELINE}; consider splitting it",
            path.display()
        )
    })
}
