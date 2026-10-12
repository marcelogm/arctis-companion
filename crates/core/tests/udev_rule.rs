use std::collections::BTreeSet;

use arctis_core::protocol::model::{BOOTLOADER_PIDS, Model, VENDOR_ID};

const RULE: &str = include_str!("../../../data/udev/70-arctis-companion.rules");

fn rule_lines() -> impl Iterator<Item = &'static str> {
    RULE.lines().filter(|line| !line.trim().is_empty())
}

fn attribute(line: &str, name: &str) -> Option<u16> {
    let key = format!("ATTRS{{{name}}}==\"");
    let start = line.find(&key)? + key.len();
    let value = line[start..].split('"').next()?;
    u16::from_str_radix(value, 16).ok()
}

fn rule_products() -> BTreeSet<u16> {
    rule_lines()
        .map(|line| attribute(line, "idProduct").expect("every line names a product"))
        .collect()
}

#[test]
fn every_rule_line_grants_uaccess_on_a_steelseries_hidraw_node() {
    for line in rule_lines() {
        assert!(line.starts_with("SUBSYSTEM==\"hidraw\""), "{line}");
        assert!(line.ends_with("TAG+=\"uaccess\""), "{line}");
        assert_eq!(attribute(line, "idVendor"), Some(VENDOR_ID), "{line}");
    }
}

#[test]
fn the_rule_lists_exactly_the_supported_pids() {
    let supported: BTreeSet<u16> = Model::SUPPORTED.iter().map(|model| model.pid).collect();
    assert_eq!(rule_products(), supported);
}

#[test]
fn the_rule_lists_no_bootloader_pid() {
    let products = rule_products();
    assert!(BOOTLOADER_PIDS.iter().all(|pid| !products.contains(pid)));
}
