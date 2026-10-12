use std::path::Path;

use crate::cli::MessageSource;
use crate::report::Report;

pub fn run(_root: &Path, _sources: &[MessageSource]) -> Report {
    Report::default()
}
