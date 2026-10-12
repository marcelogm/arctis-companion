use std::path::Path;

use crate::report::Report;

pub fn run(_root: &Path, _staged: bool) -> Report {
    Report::default()
}
