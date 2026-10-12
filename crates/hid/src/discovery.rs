use std::fs;
use std::path::{Path, PathBuf};

use arctis_core::protocol::model::{Model, ModelLookup, VENDOR_ID};

use crate::uevent::{self, Uevent};

pub const COMMAND_INTERFACE: u8 = 3;
pub const EVENTS_INTERFACE: u8 = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SysfsRoot {
    class: PathBuf,
    dev: PathBuf,
}

impl SysfsRoot {
    #[must_use]
    pub fn new(class: impl Into<PathBuf>, dev: impl Into<PathBuf>) -> Self {
        Self {
            class: class.into(),
            dev: dev.into(),
        }
    }

    #[must_use]
    pub fn system() -> Self {
        Self::new("/sys/class/hidraw", "/dev")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub pid: u16,
    pub uniq: Option<String>,
    pub physical: String,
    pub interface: u8,
    pub path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dongle {
    pub pid: u16,
    pub uniq: Option<String>,
    pub physical: String,
    pub command: PathBuf,
    pub events: PathBuf,
}

#[must_use]
pub fn scan(root: &SysfsRoot) -> Vec<Node> {
    let Ok(entries) = fs::read_dir(&root.class) else {
        return Vec::new();
    };
    let mut nodes: Vec<Node> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| node(root, &entry.file_name().to_string_lossy()))
        .collect();
    nodes.sort_by(|a, b| (&a.physical, a.interface).cmp(&(&b.physical, b.interface)));
    nodes
}

#[must_use]
pub fn first_dongle(nodes: &[Node]) -> Option<Dongle> {
    nodes
        .iter()
        .find_map(|node| dongle_at(nodes, &node.physical))
}

#[must_use]
pub fn command_node(nodes: &[Node], physical: &str) -> Option<PathBuf> {
    interface_node(nodes, physical, COMMAND_INTERFACE).map(|node| node.path.clone())
}

fn dongle_at(nodes: &[Node], physical: &str) -> Option<Dongle> {
    let command = interface_node(nodes, physical, COMMAND_INTERFACE)?;
    let events = interface_node(nodes, physical, EVENTS_INTERFACE)?;
    Some(Dongle {
        pid: command.pid,
        uniq: command.uniq.clone(),
        physical: physical.to_owned(),
        command: command.path.clone(),
        events: events.path.clone(),
    })
}

fn interface_node<'a>(nodes: &'a [Node], physical: &str, interface: u8) -> Option<&'a Node> {
    nodes
        .iter()
        .find(|node| node.physical == physical && node.interface == interface)
}

fn node(root: &SysfsRoot, name: &str) -> Option<Node> {
    let text = fs::read_to_string(uevent_path(&root.class, name)).ok()?;
    let uevent = uevent::parse(&text).filter(is_supported)?;
    Some(Node {
        pid: uevent.product,
        uniq: uevent.uniq,
        physical: uevent.physical,
        interface: uevent.interface,
        path: root.dev.join(name),
    })
}

fn uevent_path(class: &Path, name: &str) -> PathBuf {
    class.join(name).join("device").join("uevent")
}

fn is_supported(uevent: &Uevent) -> bool {
    uevent.vendor == VENDOR_ID && matches!(Model::lookup(uevent.product), ModelLookup::Supported(_))
}
