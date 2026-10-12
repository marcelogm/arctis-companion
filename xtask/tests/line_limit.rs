mod support;

use support::Fixture;

#[test]
fn a_file_of_401_lines_warns_and_exits_zero() {
    let fixture = Fixture::init();
    fixture.write("notes/long.txt", &"line\n".repeat(401));
    let outcome = fixture.xtask(&["check-rules"]);
    assert!(outcome.succeeded(), "{}", outcome.stderr);
    assert!(
        outcome
            .stderr
            .contains("warning: notes/long.txt: 401 lines"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn a_file_of_400_lines_is_silent() {
    let fixture = Fixture::init();
    fixture.write("notes/long.txt", &"line\n".repeat(400));
    let outcome = fixture.xtask(&["check-rules"]);
    assert!(outcome.succeeded());
    assert!(!outcome.stderr.contains("warning"), "{}", outcome.stderr);
}

#[test]
fn the_staged_mode_warns_too() {
    let fixture = Fixture::init();
    fixture.write("notes/long.txt", &"line\n".repeat(401));
    let outcome = fixture.xtask(&["check-rules", "--staged"]);
    assert!(outcome.succeeded());
    assert!(outcome.stderr.contains("401 lines"), "{}", outcome.stderr);
}

#[test]
fn plans_are_exempt_from_the_line_limit() {
    let fixture = Fixture::init();
    fixture.write("docs/superpowers/plans/plan.md", &"line\n".repeat(900));
    let outcome = fixture.xtask(&["check-rules"]);
    assert!(outcome.succeeded(), "{}", outcome.stderr);
    assert!(!outcome.stderr.contains("warning"), "{}", outcome.stderr);
}
