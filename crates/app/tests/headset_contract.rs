use std::fs;
use std::path::Path;

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

fn contract_xml() -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/dbus")
        .join(format!("{HEADSET1}.xml"));
    fs::read_to_string(file).expect("contract file")
}

#[test]
fn headset_contract_parses_and_declares_every_member() {
    let node = gtk::gio::DBusNodeInfo::for_xml(&contract_xml()).expect("valid introspection XML");
    let interface = node.lookup_interface(HEADSET1).expect("Headset1 interface");
    for property in HEADSET1_PROPERTIES {
        assert!(interface.lookup_property(property).is_some(), "{property}");
    }
    assert!(interface.lookup_method("ActivatePreset").is_some());
}
