mod support;

use std::fs;
use std::process::Command;

use support::Fixture;

const TEXT_RS: &str = "pub fn title() -> String {\n    gettext(\"Arctis Companion\")\n}\n";
const XGETTEXT: [&str; 5] = [
    "--language=Rust",
    "--keyword=gettext",
    "--package-name=arctis-companion",
    "--output=po/arctis-companion.pot",
    "src/text.rs",
];
const MSGINIT: [&str; 4] = [
    "--no-translator",
    "--locale=pt_BR",
    "--input=po/arctis-companion.pot",
    "--output-file=po/pt_BR.po",
];

fn run_in(fixture: &Fixture, program: &str, args: &[&str]) {
    let status = Command::new(program)
        .args(args)
        .current_dir(fixture.path())
        .status()
        .expect("tool runs");
    assert!(status.success(), "{program} {args:?}");
}

fn generate_lockfile(fixture: &Fixture) {
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    fixture.write("src/lib.rs", "");
    run_in(fixture, env!("CARGO"), &["generate-lockfile", "--offline"]);
    fixture.git(&["add", "Cargo.lock"]);
}

fn generate_translation(fixture: &Fixture) {
    fixture.write("src/text.rs", TEXT_RS);
    fixture.write("po/POTFILES.in", "src/text.rs\n");
    fixture.write("po/LINGUAS", "pt_BR\n");
    run_in(fixture, "xgettext", &XGETTEXT);
    run_in(fixture, "msginit", &MSGINIT);
    fixture.git(&["add", "po"]);
}

#[test]
fn fresh_lockfile_and_translation_pass_check_rules() {
    let fixture = Fixture::init();
    generate_lockfile(&fixture);
    generate_translation(&fixture);
    let lockfile = fs::read_to_string(fixture.path().join("Cargo.lock")).expect("lockfile");
    assert!(lockfile.starts_with("# This file is automatically @generated"));
    let translation = fs::read(fixture.path().join("po/pt_BR.po")).expect("translation");
    assert!(
        translation.starts_with(b"# "),
        "msginit writes a comment header"
    );
    let outcome = fixture.xtask(&["check-rules"]);
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

const EXEMPT: [&str; 11] = [
    "Cargo.lock",
    "THIRD_PARTY_LICENSES",
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
fn every_exempt_entry_skips_the_line_limit() {
    let fixture = Fixture::init();
    for path in EXEMPT {
        fixture.write(path, &"line\n".repeat(900));
    }
    let outcome = fixture.xtask(&["check-rules"]);
    assert!(outcome.succeeded(), "{}", outcome.stderr);
    assert!(!outcome.stderr.contains("warning"), "{}", outcome.stderr);
}
