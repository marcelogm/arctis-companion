mod support;

use std::fs;

use support::{Fixture, repo_root};

const GUARDED: [&str; 5] = [
    "missing_errors_doc",
    "missing_panics_doc",
    "missing_safety_doc",
    "undocumented_unsafe_blocks",
    "doc_markdown",
];

fn repository_manifest() -> String {
    fs::read_to_string(repo_root().join("Cargo.toml")).expect("root manifest")
}

fn check_rules_with(manifest: &str) -> support::Outcome {
    let fixture = Fixture::init();
    fixture.write("Cargo.toml", manifest);
    fixture.xtask(&["check-rules"])
}

#[test]
fn repository_manifest_passes() {
    let outcome = check_rules_with(&repository_manifest());
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

#[test]
fn removing_any_guarded_allow_fails() {
    for lint in GUARDED {
        let manifest = repository_manifest().replace(&format!("{lint} = \"allow\"\n"), "");
        let outcome = check_rules_with(&manifest);
        assert!(!outcome.succeeded(), "{lint}");
        assert!(outcome.stderr.contains(lint), "{}", outcome.stderr);
    }
}

#[test]
fn turning_an_allow_into_warn_fails() {
    let manifest =
        repository_manifest().replace("doc_markdown = \"allow\"", "doc_markdown = \"warn\"");
    assert!(!check_rules_with(&manifest).succeeded());
}

#[test]
fn staged_mode_reads_the_manifest_from_the_index() {
    let manifest = repository_manifest().replace("doc_markdown = \"allow\"\n", "");
    let fixture = Fixture::init();
    fixture
        .write("Cargo.toml", &manifest)
        .edit_unstaged("Cargo.toml", &repository_manifest());
    assert!(!fixture.xtask(&["check-rules", "--staged"]).succeeded());
}

const MEMBER: &str = "crates/core/Cargo.toml";
const PACKAGE: &str =
    "[package]\nname = \"arctis-core\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";

fn check_rules_with_member(member_manifest: &str) -> support::Outcome {
    let fixture = Fixture::init();
    fixture
        .write("Cargo.toml", &repository_manifest())
        .write(MEMBER, member_manifest);
    fixture.xtask(&["check-rules"])
}

#[test]
fn member_inheriting_the_workspace_lints_passes() {
    let outcome = check_rules_with_member(&format!("{PACKAGE}\n[lints]\nworkspace = true\n"));
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

#[test]
fn member_opting_out_of_the_workspace_lints_fails() {
    let outcome = check_rules_with_member(PACKAGE);
    assert!(!outcome.succeeded());
    assert!(outcome.stderr.contains(MEMBER), "{}", outcome.stderr);
}
