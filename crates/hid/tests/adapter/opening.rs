use std::fs;
use std::os::unix::fs::PermissionsExt;

use arctis_hid::discovery;
use arctis_hid::port::{OpenError, Port, classify};
use rustix::io::Errno;

use crate::sysfs::{COMMAND_NODE, FakeSysfs};

#[test]
fn a_reported_node_missing_from_dev_is_denied() {
    let sysfs = FakeSysfs::new();
    sysfs.add_nova_5();
    let nodes = discovery::scan(&sysfs.root());
    let dongle = discovery::first_dongle(&nodes).expect("the dongle");
    assert_eq!(Port::open(&dongle.command).err(), Some(OpenError::Denied));
}

#[test]
fn a_node_without_the_access_rule_is_denied() {
    let sysfs = FakeSysfs::new();
    sysfs.add_nova_5();
    let node = sysfs.dev_node(COMMAND_NODE);
    fs::write(&node, b"").expect("a stand-in node");
    fs::set_permissions(&node, fs::Permissions::from_mode(0o000)).expect("mode 0000");
    let opened = Port::open(&node);
    if rustix::process::geteuid().is_root() {
        assert!(opened.is_ok());
    } else {
        assert_eq!(opened.err(), Some(OpenError::Denied));
    }
}

#[test]
fn open_errors_split_into_denied_and_gone() {
    for errno in [Errno::ACCESS, Errno::PERM, Errno::NOENT] {
        assert_eq!(classify(errno), OpenError::Denied, "{errno:?}");
    }
    for errno in [Errno::NODEV, Errno::IO, Errno::NXIO, Errno::BUSY] {
        assert_eq!(classify(errno), OpenError::Gone, "{errno:?}");
    }
}
