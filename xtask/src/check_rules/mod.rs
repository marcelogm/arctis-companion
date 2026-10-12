mod comments;
mod dbus_copy;
mod generated;
mod line_limit;
mod lints;
mod snapshot;
mod unsafe_api;

use std::path::Path;

use crate::report::Report;
use snapshot::Snapshot;

pub fn run(root: &Path, staged: bool) -> Report {
    let snapshot = if staged {
        Snapshot::Index(root)
    } else {
        Snapshot::WorkingTree(root)
    };
    match (snapshot.files_to_check(), snapshot.tracked_files()) {
        (Ok(files), Ok(tracked)) => check_tree(snapshot, &files, &tracked),
        (Err(message), _) | (_, Err(message)) => Report::failed(message),
    }
}

fn check_tree(snapshot: Snapshot, files: &[String], tracked: &[String]) -> Report {
    let per_file = files.iter().map(|file| check_file(snapshot, file));
    per_file
        .fold(Report::default(), Report::merged)
        .merged(lints::check(snapshot, files))
        .merged(dbus_copy::check(snapshot, tracked))
}

fn check_file(snapshot: Snapshot, file: &str) -> Report {
    let path = Path::new(file);
    if generated::is_generated(path) {
        return Report::default();
    }
    snapshot
        .read(file)
        .map_or_else(Report::default, |text| check_text(path, &text))
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
