mod support;

use support::Fixture;

const SOURCE: &str = "data/dbus/io.github.marcelogm.ArctisCompanion.Headset1.xml";
const COPY: &str = "extension/arctis-companion@marcelogm.github.io/dbus/io.github.marcelogm.ArctisCompanion.Headset1.xml";
const CONTRACT: &str =
    "<node>\n  <interface name=\"io.github.marcelogm.ArctisCompanion.Headset1\"/>\n</node>\n";

#[test]
fn identical_copy_passes() {
    let fixture = Fixture::init();
    fixture.write(SOURCE, CONTRACT).write(COPY, CONTRACT);
    let outcome = fixture.xtask(&["check-rules"]);
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

#[test]
fn diverging_copy_fails() {
    let fixture = Fixture::init();
    fixture.write(SOURCE, CONTRACT).write(COPY, "<node/>\n");
    let outcome = fixture.xtask(&["check-rules"]);
    assert!(!outcome.succeeded());
    assert!(outcome.stderr.contains(COPY), "{}", outcome.stderr);
}

#[test]
fn copy_without_a_source_fails() {
    let fixture = Fixture::init();
    fixture.write(COPY, CONTRACT);
    assert!(!fixture.xtask(&["check-rules"]).succeeded());
}
