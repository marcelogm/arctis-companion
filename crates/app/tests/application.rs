use std::fs;
use std::path::{Path, PathBuf};

use adw::prelude::*;
use arctis_app::application::{self, APP_ID};

#[test]
fn application_uses_the_app_id() {
    assert_eq!(
        application::build().application_id().as_deref(),
        Some(APP_ID)
    );
}

const HEADSET1: &str = "io.github.marcelogm.ArctisCompanion.Headset1";
const HEADSET1_PROPERTIES: [&str; 11] = [
    "State",
    "Model",
    "BatteryLevel",
    "Charging",
    "BatteryWarning",
    "Route",
    "Presets",
    "ActivePreset",
    "ActivePresetReason",
    "AppRuleCount",
    "AutomaticPresets",
];

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

#[test]
fn every_data_file_is_named_after_the_app_id() {
    for suffix in [".desktop", ".metainfo.xml", ".gschema.xml", ".service.in"] {
        let file = data_dir().join(format!("{APP_ID}{suffix}"));
        assert!(file.is_file(), "{}", file.display());
    }
}

#[test]
fn headset_contract_parses_and_declares_every_member() {
    let xml =
        fs::read_to_string(data_dir().join(format!("dbus/{HEADSET1}.xml"))).expect("contract file");
    let node = gtk::gio::DBusNodeInfo::for_xml(&xml).expect("valid introspection XML");
    let interface = node.lookup_interface(HEADSET1).expect("Headset1 interface");
    for property in HEADSET1_PROPERTIES {
        assert!(interface.lookup_property(property).is_some(), "{property}");
    }
    assert!(interface.lookup_method("ActivatePreset").is_some());
}
