mod support;

use support::Fixture;

const TIME_RS: &str = "pub struct Millis;\n\nimpl Millis {\n    pub fn after(self) {}\n}\n";

fn outcome_for(document: &str, markdown: &str) -> support::Outcome {
    let fixture = Fixture::init();
    fixture.write("crates/core/src/time.rs", TIME_RS);
    fixture.write(document, markdown);
    fixture.xtask(&["check-doc-paths"])
}

#[test]
fn existing_path_passes() {
    let outcome = outcome_for(
        "docs/architecture/core.md",
        "See `crates/core/src/time.rs`.\n",
    );
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

#[test]
fn missing_path_fails_with_its_line() {
    let outcome = outcome_for(
        "docs/architecture/core.md",
        "Intro.\nSee `crates/core/src/gone.rs`.\n",
    );
    assert!(!outcome.succeeded());
    assert!(
        outcome.stderr.contains("docs/architecture/core.md:2"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn declared_symbol_passes() {
    assert!(
        outcome_for(
            "docs/architecture/core.md",
            "`crates/core/src/time.rs::Millis`\n"
        )
        .succeeded()
    );
}

#[test]
fn missing_symbol_fails() {
    assert!(
        !outcome_for(
            "docs/architecture/core.md",
            "`crates/core/src/time.rs::Instant`\n"
        )
        .succeeded()
    );
}

#[test]
fn method_reference_checks_its_last_segment() {
    assert!(
        outcome_for(
            "docs/architecture/core.md",
            "`crates/core/src/time.rs::Millis::after`\n"
        )
        .succeeded()
    );
}

#[test]
fn directory_reference_must_be_a_directory() {
    assert!(outcome_for("docs/architecture/core.md", "`crates/core/src/`\n").succeeded());
    assert!(!outcome_for("docs/architecture/core.md", "`crates/core/src/time.rs/`\n").succeeded());
}

#[test]
fn fenced_blocks_and_non_paths_are_ignored() {
    let markdown = "```text\n`gone/path.rs`\n```\nUse `core::session`, `org.gnome.Sdk//50`, `https://example.org/x` and `xtask/tests/*.rs`.\n";
    let outcome = outcome_for("docs/architecture/core.md", markdown);
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

#[test]
fn nested_architecture_pages_are_checked() {
    assert!(
        !outcome_for(
            "docs/architecture/hid/discovery.md",
            "`crates/hid/src/gone.rs`\n"
        )
        .succeeded()
    );
}

#[test]
fn the_glossary_is_checked() {
    assert!(!outcome_for("CONTEXT.md", "`crates/core/src/gone.rs`\n").succeeded());
}

#[test]
fn research_is_exempt() {
    assert!(outcome_for("docs/research/old.md", "`crates/core/src/gone.rs`\n").succeeded());
}

#[test]
fn adr_code_line_with_a_missing_path_fails() {
    let outcome = outcome_for(
        "docs/adr/0001-example.md",
        "# Example\n\n- Code: `crates/core/src/gone.rs`.\n",
    );
    assert!(!outcome.succeeded());
    assert!(
        outcome.stderr.contains("docs/adr/0001-example.md:3"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn adr_code_line_with_an_existing_symbol_passes() {
    let outcome = outcome_for(
        "docs/adr/0001-example.md",
        "- Code: `crates/core/src/time.rs::Millis`.\n",
    );
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

#[test]
fn adr_prose_is_not_checked() {
    let outcome = outcome_for(
        "docs/adr/0001-example.md",
        "Superseded `scripts/check-line-limit.sh`.\n",
    );
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

#[test]
fn an_unnumbered_page_is_not_an_adr() {
    let outcome = outcome_for("docs/adr/notes.md", "- Code: `crates/core/src/gone.rs`\n");
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}
