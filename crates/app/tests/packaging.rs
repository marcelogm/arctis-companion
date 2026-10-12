use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

const MANIFEST: &str = "build-aux/io.github.marcelogm.ArctisCompanion.json";
const METAINFO: &str = "data/io.github.marcelogm.ArctisCompanion.metainfo.xml.in";
const RELEASE: &str = ".github/workflows/release.yml";

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn meson_version() -> String {
    let meson = repo_file("meson.build");
    let marker = "\n  version: '";
    let start = meson.find(marker).expect("project version") + marker.len();
    meson[start..]
        .split('\'')
        .next()
        .expect("closing quote")
        .to_owned()
}

#[test]
fn the_sandbox_grants_hidraw_pipewire_and_a_display() {
    let manifest: serde_json::Value =
        serde_json::from_str(&repo_file(MANIFEST)).expect("manifest is JSON");
    let granted: BTreeSet<&str> = manifest["finish-args"]
        .as_array()
        .expect("finish-args")
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect();
    let expected = BTreeSet::from([
        "--share=ipc",
        "--socket=wayland",
        "--socket=fallback-x11",
        "--device=all",
        "--filesystem=xdg-run/pipewire-0",
    ]);
    assert_eq!(granted, expected);
}

#[test]
fn cargo_and_the_metainfo_carry_the_meson_version() {
    let version = meson_version();
    assert!(repo_file("Cargo.toml").contains(&format!("\nversion = \"{version}\"")));
    assert!(repo_file(METAINFO).contains(&format!("<release version=\"{version}\"")));
}

#[test]
fn the_metainfo_names_the_known_gaps() {
    let metainfo = repo_file(METAINFO);
    for gap in [
        "Switching the headset between 2.4 GHz and Bluetooth audio",
        "The Bluetooth equalizer preset for audio from a phone",
    ] {
        assert!(metainfo.contains(gap), "{gap}");
    }
}

#[test]
fn the_metainfo_explains_the_device_permission() {
    let metainfo = repo_file(METAINFO);
    assert!(metainfo.contains("access to all devices"));
    assert!(metainfo.contains("70-arctis-companion.rules"));
    assert!(metainfo.contains("<translation type=\"gettext\">arctis-companion</translation>"));
}

#[test]
fn a_tag_publishes_the_bundle_the_extension_and_the_rule() {
    let workflow = repo_file(RELEASE);
    assert!(workflow.contains("tags: [\"v*\"]"));
    for asset in [
        "arctis-companion.flatpak",
        "arctis-companion@marcelogm.github.io.shell-extension.zip",
        "data/udev/70-arctis-companion.rules",
    ] {
        assert!(workflow.contains(asset), "{asset}");
    }
}
