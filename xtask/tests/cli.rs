mod support;

use support::{Fixture, repo_root, xtask_in};

#[test]
fn unknown_command_prints_usage_and_exits_with_two() {
    let outcome = xtask_in(&repo_root(), &["frobnicate"]);
    assert_eq!(outcome.code, Some(2));
    assert!(
        outcome.stderr.contains("usage: cargo xtask"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn unknown_flag_exits_with_two() {
    let outcome = xtask_in(&repo_root(), &["check-rules", "--loud"]);
    assert_eq!(outcome.code, Some(2));
}

#[test]
fn every_command_is_recognised() {
    let fixture = Fixture::init();
    for command in [
        "check-rules",
        "check-commits",
        "check-doc-paths",
        "check-pure",
        "install-hooks",
    ] {
        assert_ne!(fixture.xtask(&[command]).code, Some(2), "{command}");
    }
}

#[test]
fn flag_of_another_command_exits_with_two() {
    let fixture = Fixture::init();
    let outcome = fixture.xtask(&["check-rules", "--range", "HEAD"]);
    assert_eq!(outcome.code, Some(2));
    assert!(
        outcome
            .stderr
            .contains("unknown flag --range for check-rules"),
        "{}",
        outcome.stderr
    );
    assert_eq!(fixture.xtask(&["check-pure", "--staged"]).code, Some(2));
}
