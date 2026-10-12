mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use support::{Fixture, xtask_with_env};

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

fn local_hooks_path_is_set(fixture: &Fixture) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(fixture.path())
        .args(["config", "--local", "--get", "core.hooksPath"])
        .status()
        .expect("git runs")
        .success()
}

fn hook_path(fixture: &Fixture, hook: &str) -> std::path::PathBuf {
    fixture.path().join(".git/hooks").join(hook)
}

#[test]
fn installs_both_hooks_and_drops_the_legacy_hooks_path() {
    let fixture = Fixture::init();
    fixture.git(&["config", "core.hooksPath", ".githooks"]);
    let outcome = fixture.xtask(&["install-hooks"]);
    assert!(outcome.succeeded(), "{}", outcome.stderr);
    assert_hook(
        &hook_path(&fixture, "pre-commit"),
        "cargo xtask check-rules --staged",
    );
    assert_hook(
        &hook_path(&fixture, "commit-msg"),
        "cargo xtask check-commits --message \"$1\"",
    );
    assert!(!local_hooks_path_is_set(&fixture));
}

#[test]
fn installing_twice_succeeds() {
    let fixture = Fixture::init();
    assert!(fixture.xtask(&["install-hooks"]).succeeded());
    assert!(fixture.xtask(&["install-hooks"]).succeeded());
}

#[test]
fn a_hook_from_an_older_xtask_is_replaced() {
    let fixture = Fixture::init();
    fs::create_dir_all(fixture.path().join(".git/hooks")).expect("hooks dir");
    fs::write(
        hook_path(&fixture, "pre-commit"),
        "#!/bin/sh\nexec cargo xtask check-rules\n",
    )
    .expect("old hook");
    let outcome = fixture.xtask(&["install-hooks"]);
    assert!(outcome.succeeded(), "{}", outcome.stderr);
    assert_hook(
        &hook_path(&fixture, "pre-commit"),
        "cargo xtask check-rules --staged",
    );
}

#[test]
fn a_foreign_hook_is_kept_and_reported() {
    let fixture = Fixture::init();
    let foreign = "#!/bin/sh\nexit 0\n";
    fs::create_dir_all(fixture.path().join(".git/hooks")).expect("hooks dir");
    fs::write(hook_path(&fixture, "commit-msg"), foreign).expect("foreign hook");
    let outcome = fixture.xtask(&["install-hooks"]);
    assert!(!outcome.succeeded());
    assert!(outcome.stderr.contains("commit-msg"), "{}", outcome.stderr);
    let kept = fs::read_to_string(hook_path(&fixture, "commit-msg")).expect("hook");
    assert_eq!(kept, foreign);
}

#[test]
fn a_global_hooks_path_is_reported() {
    let fixture = Fixture::init();
    let global = fixture.path().join("global.gitconfig");
    fs::write(&global, "[core]\n\thooksPath = /elsewhere\n").expect("global config");
    let outcome = xtask_with_env(
        fixture.path(),
        &["install-hooks"],
        &[("GIT_CONFIG_GLOBAL", &global)],
    );
    assert!(!outcome.succeeded());
    assert!(outcome.stderr.contains("global"), "{}", outcome.stderr);
}

fn run_hook(fixture: &Fixture, hook: &str, args: &[&str]) -> Option<i32> {
    let status = Command::new("sh")
        .arg(hook_path(fixture, hook))
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
