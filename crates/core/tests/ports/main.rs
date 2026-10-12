use arctis_core::ports::StoreError;
use arctis_core::testkit::memory_repositories::{MemoryBattery, MemoryPresets, MemorySeenApps};
use arctis_core::testkit::repository_contracts::{
    battery_repository_contract, preset_repository_contract, seen_apps_repository_contract,
};

const fn same<R>(store: R) -> R {
    store
}

#[test]
fn memory_presets_pass_the_contract() {
    preset_repository_contract(MemoryPresets::default, same);
}

#[test]
fn memory_battery_passes_the_contract() {
    battery_repository_contract(MemoryBattery::default, same);
}

#[test]
fn memory_seen_apps_pass_the_contract() {
    seen_apps_repository_contract(MemorySeenApps::default, same);
}

#[test]
#[should_panic(expected = "presets survive a reopen")]
fn a_store_that_forgets_on_reopen_fails_the_contract() {
    preset_repository_contract(MemoryPresets::default, |_| MemoryPresets::default());
}

#[test]
fn a_store_error_shows_its_message() {
    assert_eq!(StoreError("disk full".to_owned()).to_string(), "disk full");
}
