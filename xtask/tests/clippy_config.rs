mod support;

use std::fs;
use std::process::Command;

use support::{Fixture, repo_root};

const MEMBER_MANIFEST: &str = "[package]\nname = \"member\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lints]\nworkspace = true\n";
const UNDOCUMENTED_RESULT: &str = "pub struct Failure;\n\npub fn f(ok: bool) -> Result<(), Failure> {\n    if ok { Ok(()) } else { Err(Failure) }\n}\n";

fn repository_lints() -> toml::Value {
    let text = fs::read_to_string(repo_root().join("Cargo.toml")).expect("root manifest");
    let manifest: toml::Table = text.parse().expect("valid toml");
    manifest["workspace"]["lints"].clone()
}

fn workspace_manifest(lints: toml::Value) -> String {
    let mut workspace = toml::Table::new();
    workspace.insert("members".into(), toml::Value::Array(vec!["member".into()]));
    workspace.insert("resolver".into(), "3".into());
    workspace.insert("lints".into(), lints);
    let mut root = toml::Table::new();
    root.insert("workspace".into(), toml::Value::Table(workspace));
    toml::to_string(&root).expect("serialisable manifest")
}

fn clippy_passes(lints: toml::Value) -> bool {
    let fixture = Fixture::init();
    fixture.write("Cargo.toml", &workspace_manifest(lints));
    fixture.write(
        "clippy.toml",
        &fs::read_to_string(repo_root().join("clippy.toml")).expect("clippy.toml"),
    );
    fixture.write("member/Cargo.toml", MEMBER_MANIFEST);
    fixture.write("member/src/lib.rs", UNDOCUMENTED_RESULT);
    let output = Command::new(env!("CARGO"))
        .args(["clippy", "--quiet", "--offline", "--", "-D", "warnings"])
        .current_dir(fixture.path())
        .env("CARGO_TARGET_DIR", fixture.path().join("target"))
        .output()
        .expect("cargo clippy runs");
    output.status.success()
}

#[test]
fn undocumented_public_result_function_passes_clippy_with_denied_warnings() {
    assert!(clippy_passes(repository_lints()));
}

#[test]
fn without_the_errors_doc_allow_the_same_function_fails() {
    let mut lints = repository_lints();
    lints["clippy"]
        .as_table_mut()
        .expect("clippy table")
        .remove("missing_errors_doc");
    assert!(!clippy_passes(lints));
}
