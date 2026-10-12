use arctis_core::protocol::model::{
    BOOTLOADER_PIDS, Capabilities, Model, ModelLookup, Support, VENDOR_ID,
};
use arctis_core::settings::headset_settings::Setting;

fn supported(pid: u16) -> Model {
    let ModelLookup::Supported(model) = Model::lookup(pid) else {
        panic!("{pid:04x} is not a supported model");
    };
    model
}

#[test]
fn the_vendor_is_steelseries() {
    assert_eq!(VENDOR_ID, 0x1038);
}

#[test]
fn the_nova_5_dongle_is_a_verified_model() {
    let model = supported(0x2232);
    assert_eq!(
        (model.name, model.support),
        ("Arctis Nova 5", Support::Verified)
    );
}

#[test]
fn the_nova_5x_dongles_are_known_and_two_are_untested() {
    let supports: Vec<(u16, Support)> = [0x2253, 0x2255, 0x2264]
        .into_iter()
        .map(|pid| (pid, supported(pid).support))
        .collect();
    assert_eq!(
        supports,
        [
            (0x2253, Support::Verified),
            (0x2255, Support::Untested),
            (0x2264, Support::Untested)
        ]
    );
}

#[test]
fn every_supported_pid_resolves_to_its_model() {
    for model in Model::SUPPORTED {
        assert_eq!(Model::lookup(model.pid), ModelLookup::Supported(model));
    }
}

#[test]
fn bootloader_pids_are_never_models() {
    assert_eq!(BOOTLOADER_PIDS, [0x2233, 0x2254, 0x2265]);
    for pid in BOOTLOADER_PIDS {
        assert_eq!(Model::lookup(pid), ModelLookup::Bootloader);
    }
}

#[test]
fn other_pids_are_unknown() {
    assert_eq!(Model::lookup(0x1234), ModelLookup::Unknown);
}

#[test]
fn every_model_supports_every_headset_setting() {
    for model in Model::SUPPORTED {
        let all = Setting::ALL
            .into_iter()
            .all(|setting| model.capabilities.supports(setting));
        assert!(all, "{:04x}", model.pid);
    }
}

#[test]
fn capabilities_support_only_their_listed_settings() {
    let sidetone_only = Capabilities::new(&[Setting::Sidetone]);
    let supported: Vec<Setting> = Setting::ALL
        .into_iter()
        .filter(|setting| sidetone_only.supports(*setting))
        .collect();
    assert_eq!(supported, [Setting::Sidetone]);
}
