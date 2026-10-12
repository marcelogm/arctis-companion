mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use support::{Fixture, git_succeeds};

fn assert_hook(path: &Path, command: &str) {
    let script = fs::read_to_string(path).expect("hook written");
    assert!(script.starts_with("#!/bin/sh\n"), "{script}");
    assert!(script.contains(command), "{script}");
    let mode = fs::metadata(path)
        .expect("hook metadata")
        .permissions()
        .mode();
    assert_ne!(mode & 0o111, 0, "hook must be executable");
}

#[test]
fn installs_both_hooks_and_drops_the_legacy_hooks_path() {
    let fixture = Fixture::init();
    fixture.git(&["config", "core.hooksPath", ".githooks"]);
    let outcome = fixture.xtask(&["install-hooks"]);
    assert!(outcome.succeeded(), "{}", outcome.stderr);
    let hooks = fixture.path().join(".git/hooks");
    assert_hook(
        &hooks.join("pre-commit"),
        "cargo xtask check-rules --staged",
    );
    assert_hook(
        &hooks.join("commit-msg"),
        "cargo xtask check-commits --message \"$1\"",
    );
    assert!(!git_succeeds(
        fixture.path(),
        &["config", "--get", "core.hooksPath"]
    ));
}

#[test]
fn installing_twice_succeeds() {
    let fixture = Fixture::init();
    assert!(fixture.xtask(&["install-hooks"]).succeeded());
    assert!(fixture.xtask(&["install-hooks"]).succeeded());
}

fn run_hook(fixture: &Fixture, hook: &str, args: &[&str]) -> Option<i32> {
    let script = fixture.path().join(".git/hooks").join(hook);
    let status = Command::new("sh")
        .arg(script)
        .args(args)
        .current_dir(fixture.path())
        .status();
    status.expect("hook runs").code()
}

#[test]
fn pre_commit_hook_skips_checkouts_without_xtask() {
    let fixture = Fixture::init();
    fixture.xtask(&["install-hooks"]);
    assert_eq!(run_hook(&fixture, "pre-commit", &[]), Some(0));
}

#[test]
fn commit_msg_hook_runs_the_legacy_guard_in_checkouts_without_xtask() {
    let fixture = Fixture::init();
    fixture.write(".githooks/commit-msg", "#!/bin/sh\nexit 7\n");
    fixture.xtask(&["install-hooks"]);
    assert_eq!(run_hook(&fixture, "commit-msg", &["message"]), Some(7));
}

#[test]
fn commit_msg_hook_passes_checkouts_without_any_guard() {
    let fixture = Fixture::init();
    fixture.xtask(&["install-hooks"]);
    assert_eq!(run_hook(&fixture, "commit-msg", &["message"]), Some(0));
}
