use std::fs;
use std::path::PathBuf;

use arctis_hid::SysfsRoot;
use tempfile::TempDir;

pub const NOVA_5: u16 = 0x2232;
pub const PHYSICAL: &str = "usb-0000:0c:00.0-5.2.1";
pub const COMMAND_NODE: &str = "hidraw6";
pub const EVENTS_NODE: &str = "hidraw8";

#[derive(Clone, Copy, Debug)]
pub struct NodeSpec<'a> {
    pub vendor: u16,
    pub pid: u16,
    pub physical: &'a str,
    pub interface: u8,
    pub uniq: &'a str,
}

impl NodeSpec<'_> {
    pub const fn nova_5(interface: u8) -> Self {
        Self {
            vendor: 0x1038,
            pid: NOVA_5,
            physical: PHYSICAL,
            interface,
            uniq: "",
        }
    }

    fn uevent(&self) -> String {
        format!(
            "DRIVER=hid-generic\nHID_ID=0003:{:08X}:{:08X}\nHID_NAME=SteelSeries Arctis Nova 5\nHID_PHYS={}/input{}\nHID_UNIQ={}\nMODALIAS=hid:b0003g0001\n",
            self.vendor, self.pid, self.physical, self.interface, self.uniq
        )
    }
}

pub struct FakeSysfs {
    dir: TempDir,
}

impl FakeSysfs {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("a temporary directory");
        fs::create_dir_all(dir.path().join("class")).expect("the class directory");
        fs::create_dir_all(dir.path().join("dev")).expect("the dev directory");
        Self { dir }
    }

    pub fn root(&self) -> SysfsRoot {
        SysfsRoot::new(self.dir.path().join("class"), self.dir.path().join("dev"))
    }

    pub fn dev_node(&self, name: &str) -> PathBuf {
        self.dir.path().join("dev").join(name)
    }

    pub fn add(&self, name: &str, spec: &NodeSpec<'_>) {
        self.add_raw(name, &spec.uevent());
    }

    pub fn add_raw(&self, name: &str, uevent: &str) {
        let device = self.dir.path().join("class").join(name).join("device");
        fs::create_dir_all(&device).expect("the device directory");
        fs::write(device.join("uevent"), uevent).expect("the uevent file");
    }

    pub fn add_nova_5(&self) {
        self.add(COMMAND_NODE, &NodeSpec::nova_5(3));
        self.add(EVENTS_NODE, &NodeSpec::nova_5(5));
    }
}
