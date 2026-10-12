#![forbid(unsafe_code)]

mod check_commits;
mod check_doc_paths;
mod check_pure;
mod check_rules;
mod cli;
mod commit_message;
mod doc_refs;
mod git;
mod install_hooks;
mod report;
mod rust_tokens;
mod text;

use std::process::ExitCode;

use cli::Invocation;
use report::Report;

fn main() -> ExitCode {
    match cli::parse(std::env::args().skip(1)) {
        Ok(invocation) => run(invocation).finish(),
        Err(usage) => {
            eprintln!("{usage}");
            ExitCode::from(2)
        }
    }
}

fn run(invocation: Invocation) -> Report {
    match invocation {
        Invocation::CheckRules { root, staged } => check_rules::run(&root, staged),
        Invocation::CheckCommits { root, sources } => check_commits::run(&root, &sources),
        Invocation::CheckDocPaths { root } => check_doc_paths::run(&root),
        Invocation::CheckPure { root } => check_pure::run(&root),
        Invocation::InstallHooks { root } => install_hooks::run(&root),
    }
}
