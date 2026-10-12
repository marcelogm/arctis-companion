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

const GENERATED: [&str; 10] = [
    "Cargo.lock",
    "build-aux/cargo-sources.json",
    "extension/package-lock.json",
    "LICENSE",
    "po/pt_BR.po",
    "po/arctis-companion.pot",
    "po/LINGUAS",
    "po/POTFILES.in",
    "data/icons/app.svg",
    "data/screenshots/main.png",
];

#[test]
fn every_entry_of_the_generated_list_is_exempt() {
    let fixture = Fixture::init();
    for path in GENERATED {
        fixture.write(path, &"line\n".repeat(900));
    }
    let outcome = fixture.xtask(&["check-rules"]);
    assert!(outcome.succeeded(), "{}", outcome.stderr);
    assert!(!outcome.stderr.contains("warning"), "{}", outcome.stderr);
}
