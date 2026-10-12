use std::fs;
use std::path::Path;

use crate::cli::MessageSource;
use crate::commit_message::{attribution_problem, editor_message, subject_of, subject_problem};
use crate::git;
use crate::report::Report;

pub fn run(root: &Path, sources: &[MessageSource]) -> Report {
    if sources.is_empty() {
        return Report::failed("check-commits needs --message, --range, --pr-title or --pr-body");
    }
    sources
        .iter()
        .map(|source| check_source(root, source))
        .fold(Report::default(), Report::merged)
}

fn check_source(root: &Path, source: &MessageSource) -> Report {
    match source {
        MessageSource::File(path) => check_file(path),
        MessageSource::Range(range) => check_range(root, range),
        MessageSource::PrTitle(title) => check_message("PR title", title),
        MessageSource::PrBody(body) => check_text("PR body", body),
    }
}

fn check_file(path: &Path) -> Report {
    match fs::read_to_string(path) {
        Ok(raw) => check_message("commit message", &editor_message(&raw)),
        Err(error) => Report::failed(format!("{}: {error}", path.display())),
    }
}

fn check_range(root: &Path, range: &str) -> Report {
    match git::run(root, &["rev-list", range]) {
        Ok(list) => list
            .lines()
            .map(|sha| check_commit(root, sha))
            .fold(Report::default(), Report::merged),
        Err(message) => Report::failed(message),
    }
}

fn check_commit(root: &Path, sha: &str) -> Report {
    match git::run(root, &["log", "-1", "--format=%B", sha]) {
        Ok(message) => check_message(&format!("commit {sha}"), &message),
        Err(message) => Report::failed(message),
    }
}

fn check_message(label: &str, message: &str) -> Report {
    let mut report = check_text(label, message);
    if let Some(problem) = subject_problem(subject_of(message)) {
        report.error(format!("{label}: {problem}"));
    }
    report
}

fn check_text(label: &str, text: &str) -> Report {
    attribution_problem(text).map_or_else(Report::default, |problem| {
        Report::failed(format!("{label}: {problem}"))
    })
}
