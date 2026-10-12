use arctis_core::companion::adoption::{AdoptionStatus, DongleIdentity};
use arctis_core::companion::app_settings::{HeadsetValues, PersistedChange};
use arctis_core::companion::selection::ActivePreset;
use arctis_core::companion::{Action, PresetChange};
use arctis_core::eq::preset::Preset;
use arctis_core::messages::audio::SoundPath;
use arctis_core::messages::device::{DeviceUpdate, Phase, SyncPolicy};
use arctis_core::settings::headset_settings::SettingChange;
use arctis_core::settings::values::Sidetone;

use crate::support::{
    Harness, NOVA_5, adopted, curve_of, defaults, device_changes, firmware, id, nova_5, observed,
    observed_with, persisted, persisted_changes, unique_curve,
};

fn ten() -> Sidetone {
    Sidetone::try_from(10).expect("sidetone")
}

#[test]
fn a_new_identity_is_adopted_from_what_the_headset_holds() {
    let mut h = Harness::new(persisted());
    let actions = h.connect(observed());
    assert_eq!(
        device_changes(&actions),
        [
            (defaults().with_curve(curve_of("flat")), SyncPolicy::Adopt),
            (observed(), SyncPolicy::Enforce),
        ]
    );
    assert_eq!(
        persisted_changes(&actions),
        [
            PersistedChange::Headset(HeadsetValues::from_settings(&observed())),
            PersistedChange::RoutePreset(SoundPath::Dongle, id("flat")),
            PersistedChange::AdoptedIdentity(DongleIdentity::new(NOVA_5, None)),
        ]
    );
    assert_eq!(h.snapshot().dongle.adoption, AdoptionStatus::Adopted);
}

#[test]
fn the_same_identity_is_enforced_without_adoption() {
    let mut h = Harness::new(adopted());
    let actions = h.connect(observed_with(curve_of("podcast")));
    assert_eq!(
        device_changes(&actions),
        [(observed(), SyncPolicy::Enforce)]
    );
    assert!(persisted_changes(&actions).is_empty());
}

#[test]
fn a_firmware_update_does_not_adopt_again() {
    let mut h = Harness::new(adopted());
    assert!(persisted_changes(&h.connect_as(None, 17, observed())).is_empty());
}

#[test]
fn a_dongle_with_another_serial_is_adopted() {
    let mut h = Harness::new(adopted());
    let actions = h.connect_as(Some("ABC"), 16, observed());
    let identity = DongleIdentity::new(NOVA_5, Some("ABC"));
    assert!(persisted_changes(&actions).contains(&PersistedChange::AdoptedIdentity(identity)));
}

#[test]
fn every_observed_field_becomes_desired() {
    let mut h = Harness::new(persisted());
    let _ = h.connect(observed());
    assert_eq!(
        h.snapshot().dongle.desired,
        HeadsetValues::from_settings(&observed())
    );
}

#[test]
fn an_unknown_headset_curve_is_saved_and_selected() {
    let mut h = Harness::new(persisted());
    let actions = h.connect(observed_with(unique_curve()));
    let headset_curve = Preset::custom(
        id("headset-curve"),
        "Headset Curve".to_owned(),
        unique_curve(),
    );
    assert!(actions.contains(&Action::StorePreset(PresetChange::Save(headset_curve))));
    let selected = PersistedChange::RoutePreset(SoundPath::Dongle, id("headset-curve"));
    assert!(persisted_changes(&actions).contains(&selected));
    assert_eq!(
        h.snapshot().presets.active.dongle,
        ActivePreset::route(id("headset-curve"))
    );
}

#[test]
fn adoption_waits_for_ready() {
    let mut h = Harness::new(persisted());
    let _ = h.device(DeviceUpdate::Phase(Phase::Syncing));
    let identity = DeviceUpdate::Identity {
        model: nova_5(),
        uniq: None,
        firmware: firmware(16),
    };
    let _ = h.device(identity);
    assert!(persisted_changes(&h.device(DeviceUpdate::Observed(observed()))).is_empty());
}

#[test]
fn a_button_press_after_adoption_becomes_desired() {
    let mut h = Harness::new(adopted());
    let _ = h.connect(observed());
    let actions = h.device(DeviceUpdate::HeadsetChanged(SettingChange::Sidetone(ten())));
    let mut values = HeadsetValues::from_settings(&observed());
    values.sidetone = ten();
    assert_eq!(
        persisted_changes(&actions),
        [PersistedChange::Headset(values)]
    );
    assert!(device_changes(&actions).is_empty());
}

#[test]
fn a_button_press_before_adoption_changes_no_desired_value() {
    let mut h = Harness::new(persisted());
    let _ = h.device(DeviceUpdate::Phase(Phase::HeadsetOff));
    let identity = DeviceUpdate::Identity {
        model: nova_5(),
        uniq: None,
        firmware: firmware(16),
    };
    let _ = h.device(identity);
    let actions = h.device(DeviceUpdate::HeadsetChanged(SettingChange::Sidetone(ten())));
    assert!(persisted_changes(&actions).is_empty());
    assert_eq!(h.snapshot().dongle.desired, defaults());
}
