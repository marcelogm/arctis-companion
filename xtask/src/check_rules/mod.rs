mod comments;
mod dbus_copy;
mod generated;
mod line_limit;
mod lints;
mod unsafe_api;

use std::fs;
use std::path::Path;

use crate::git;
use crate::report::Report;

pub fn run(root: &Path, staged: bool) -> Report {
    let files = if staged {
        git::staged_files(root)
    } else {
        git::tracked_files(root)
    };
    match files {
        Ok(files) => check_tree(root, &files),
        Err(message) => Report::failed(message),
    }
}

fn check_tree(root: &Path, files: &[String]) -> Report {
    let per_file = files.iter().map(|file| check_file(root, file));
    per_file
        .fold(Report::default(), Report::merged)
        .merged(lints::check(root, files))
        .merged(dbus_copy::check(root, files))
}

fn check_file(root: &Path, file: &str) -> Report {
    let path = Path::new(file);
    if generated::is_generated(path) {
        return Report::default();
    }
    fs::read_to_string(root.join(path))
        .map_or_else(|_| Report::default(), |text| check_text(path, &text))
}

fn check_text(path: &Path, text: &str) -> Report {
    let mut report = Report::default();
    if let Some(warning) = line_limit::check(path, text) {
        report.warning(warning);
    }
    if let Some(error) = comments::check(path, text) {
        report.error(error);
    }
    if let Some(error) = unsafe_api::check(path, text) {
        report.error(error);
    }
    report
}
