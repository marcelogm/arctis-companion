use std::time::{Duration, Instant};

use arctis_core::messages::device::{DeviceUpdate, Phase};
use arctis_hid::DeviceHandle;

use crate::sysfs::FakeSysfs;

const PATIENCE: Duration = Duration::from_secs(2);

fn next_update(handle: &DeviceHandle) -> Option<DeviceUpdate> {
    let start = Instant::now();
    while start.elapsed() < PATIENCE {
        if let Ok(update) = handle.updates.try_recv() {
            return Some(update);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    None
}

#[test]
fn a_dongle_whose_nodes_cannot_be_opened_is_permission_denied() {
    let sysfs = FakeSysfs::new();
    sysfs.add_nova_5();
    let handle = DeviceHandle::spawn(sysfs.root()).expect("the device thread");
    assert_eq!(
        next_update(&handle),
        Some(DeviceUpdate::Phase(Phase::PermissionDenied))
    );
}

#[test]
fn dropping_the_handle_stops_the_thread_promptly() {
    let sysfs = FakeSysfs::new();
    let handle = DeviceHandle::spawn(sysfs.root()).expect("the device thread");
    let start = Instant::now();
    drop(handle);
    assert!(start.elapsed() < Duration::from_millis(500));
}
