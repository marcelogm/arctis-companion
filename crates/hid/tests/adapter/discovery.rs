use arctis_hid::discovery::{self, Dongle};

use crate::sysfs::{COMMAND_NODE, EVENTS_NODE, FakeSysfs, NOVA_5, NodeSpec, PHYSICAL};

fn found(sysfs: &FakeSysfs) -> Option<Dongle> {
    discovery::first_dongle(&discovery::scan(&sysfs.root()))
}

fn add_model(sysfs: &FakeSysfs, pid: u16) {
    for interface in [3, 5] {
        let spec = NodeSpec {
            pid,
            ..NodeSpec::nova_5(interface)
        };
        sysfs.add(&format!("hidraw{interface}"), &spec);
    }
}

#[test]
fn a_nova_5_is_found_with_both_interfaces() {
    let sysfs = FakeSysfs::new();
    sysfs.add_nova_5();
    assert_eq!(
        found(&sysfs),
        Some(Dongle {
            pid: NOVA_5,
            uniq: None,
            physical: PHYSICAL.to_owned(),
            command: sysfs.dev_node(COMMAND_NODE),
            events: sysfs.dev_node(EVENTS_NODE),
        })
    );
}

#[test]
fn a_nova_5x_is_found() {
    let sysfs = FakeSysfs::new();
    add_model(&sysfs, 0x2253);
    assert_eq!(found(&sysfs).map(|dongle| dongle.pid), Some(0x2253));
}

#[test]
fn an_untested_model_is_found() {
    let sysfs = FakeSysfs::new();
    add_model(&sysfs, 0x2255);
    assert_eq!(found(&sysfs).map(|dongle| dongle.pid), Some(0x2255));
}

#[test]
fn a_bootloader_is_never_reported() {
    let sysfs = FakeSysfs::new();
    add_model(&sysfs, 0x2233);
    assert!(discovery::scan(&sysfs.root()).is_empty());
}

#[test]
fn an_unknown_steelseries_product_is_ignored() {
    let sysfs = FakeSysfs::new();
    add_model(&sysfs, 0x1836);
    assert!(discovery::scan(&sysfs.root()).is_empty());
}

#[test]
fn other_vendors_are_ignored() {
    let sysfs = FakeSysfs::new();
    let keyboard = NodeSpec {
        vendor: 0x3434,
        pid: 0x0A30,
        ..NodeSpec::nova_5(0)
    };
    sysfs.add("hidraw1", &keyboard);
    assert!(discovery::scan(&sysfs.root()).is_empty());
}

#[test]
fn the_first_dongle_by_physical_path_wins() {
    let sysfs = FakeSysfs::new();
    let later = "usb-0000:0e:00.3-1";
    let earlier = "usb-0000:0c:00.0-5.2.1";
    for (name, physical, interface) in [
        ("hidraw0", later, 3),
        ("hidraw1", later, 5),
        ("hidraw6", earlier, 3),
        ("hidraw8", earlier, 5),
    ] {
        sysfs.add(
            name,
            &NodeSpec {
                physical,
                ..NodeSpec::nova_5(interface)
            },
        );
    }
    assert_eq!(
        found(&sysfs).map(|dongle| dongle.physical),
        Some(earlier.to_owned())
    );
}

#[test]
fn a_dongle_without_its_events_interface_is_not_found() {
    let sysfs = FakeSysfs::new();
    sysfs.add(COMMAND_NODE, &NodeSpec::nova_5(3));
    sysfs.add("hidraw7", &NodeSpec::nova_5(4));
    assert_eq!(found(&sysfs), None);
}

#[test]
fn a_present_hid_uniq_is_kept() {
    let sysfs = FakeSysfs::new();
    for (name, interface) in [(COMMAND_NODE, 3), (EVENTS_NODE, 5)] {
        let spec = NodeSpec {
            uniq: "ABC123",
            ..NodeSpec::nova_5(interface)
        };
        sysfs.add(name, &spec);
    }
    assert_eq!(
        found(&sysfs).and_then(|dongle| dongle.uniq),
        Some("ABC123".to_owned())
    );
}

#[test]
fn an_empty_hid_uniq_is_none() {
    let sysfs = FakeSysfs::new();
    sysfs.add_nova_5();
    assert_eq!(found(&sysfs).and_then(|dongle| dongle.uniq), None);
}

#[test]
fn malformed_uevents_are_skipped() {
    let sysfs = FakeSysfs::new();
    sysfs.add_raw("hidraw2", "HID_ID=garbage\nHID_PHYS=usb-1/input3\n");
    sysfs.add_raw(
        "hidraw3",
        "HID_ID=0003:00001038:00002232\nHID_PHYS=no-interface\n",
    );
    sysfs.add_raw("hidraw4", "DRIVER=hid-generic\n");
    assert!(discovery::scan(&sysfs.root()).is_empty());
}

#[test]
fn a_missing_class_directory_finds_nothing() {
    let sysfs = FakeSysfs::new();
    let root = arctis_hid::SysfsRoot::new(sysfs.dev_node("nowhere"), sysfs.dev_node(""));
    assert!(discovery::scan(&root).is_empty());
}

#[test]
fn the_command_node_is_found_by_physical_path() {
    let sysfs = FakeSysfs::new();
    sysfs.add_nova_5();
    let nodes = discovery::scan(&sysfs.root());
    assert_eq!(
        discovery::command_node(&nodes, PHYSICAL),
        Some(sysfs.dev_node(COMMAND_NODE))
    );
    assert_eq!(discovery::command_node(&nodes, "usb-elsewhere"), None);
}

#[test]
fn a_renumbered_command_node_is_found_by_physical_path() {
    let sysfs = FakeSysfs::new();
    sysfs.add(EVENTS_NODE, &NodeSpec::nova_5(5));
    sysfs.add("hidraw9", &NodeSpec::nova_5(3));
    let nodes = discovery::scan(&sysfs.root());
    assert_eq!(
        discovery::command_node(&nodes, PHYSICAL),
        Some(sysfs.dev_node("hidraw9"))
    );
}
