mod support;

use support::Fixture;

fn workspace_with_core(dependencies: &str) -> support::Outcome {
    let fixture = Fixture::init();
    fixture.write(
        "Cargo.toml",
        "[workspace]\nresolver = \"3\"\nmembers = [\"crates/core\"]\n",
    );
    let core = format!(
        "[package]\nname = \"arctis-core\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n{dependencies}"
    );
    fixture.write("crates/core/Cargo.toml", &core);
    fixture.write("crates/core/src/lib.rs", "");
    fixture.xtask(&["check-pure"])
}

#[test]
fn core_without_dependencies_passes() {
    let outcome = workspace_with_core("");
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

#[test]
fn serde_and_serde_json_pass() {
    let outcome = workspace_with_core("[dependencies]\nserde = \"1\"\nserde_json = \"1\"\n");
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

#[test]
fn any_other_normal_dependency_fails() {
    let outcome = workspace_with_core("[dependencies]\nrustix = \"1\"\n");
    assert!(!outcome.succeeded());
    assert!(outcome.stderr.contains("rustix"), "{}", outcome.stderr);
}

#[test]
fn dev_dependencies_are_free() {
    assert!(workspace_with_core("[dev-dependencies]\nproptest = \"1\"\n").succeeded());
}

#[test]
fn build_dependencies_are_checked() {
    assert!(!workspace_with_core("[build-dependencies]\ncc = \"1\"\n").succeeded());
}

#[test]
fn target_specific_dependencies_are_checked() {
    assert!(
        !workspace_with_core("[target.'cfg(unix)'.dependencies]\nlibc = \"0.2\"\n").succeeded()
    );
}

#[test]
fn missing_core_package_fails() {
    let fixture = Fixture::init();
    fixture.write(
        "Cargo.toml",
        "[workspace]\nresolver = \"3\"\nmembers = []\n",
    );
    assert!(!fixture.xtask(&["check-pure"]).succeeded());
}
