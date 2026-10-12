use arctis_core::companion::app_settings::{HeadsetValues, PersistedChange};
use arctis_core::companion::{Action, Input};
use arctis_core::messages::audio::SoundPath;
use arctis_core::messages::device::{DeviceUpdate, SyncPolicy};
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::settings::values::Sidetone;

use crate::support::{
    Harness, adopted, curve_of, device_changes, device_curves, id, observed, percent, persisted,
    persisted_changes, published, ready,
};

#[test]
fn unchanged_settings_do_nothing() {
    let mut h = ready();
    let actions = h.send(Input::SettingsChanged(adopted().settings));
    assert!(actions.is_empty());
}

#[test]
fn changed_headset_values_are_enforced() {
    let mut h = ready();
    let mut settings = adopted().settings;
    settings.headset.sidetone = Sidetone::try_from(10).expect("sidetone");
    let actions = h.send(Input::SettingsChanged(settings.clone()));
    assert_eq!(
        device_changes(&actions),
        [(
            settings.headset.with_curve(curve_of("flat")),
            SyncPolicy::Enforce
        )]
    );
    assert!(persisted_changes(&actions).is_empty());
    assert!(published(&actions).is_some());
}

#[test]
fn a_changed_route_preset_is_applied() {
    let mut h = ready();
    let mut settings = adopted().settings;
    settings.route_presets.set(SoundPath::Dongle, id("podcast"));
    assert_eq!(
        device_curves(&h.send(Input::SettingsChanged(settings))),
        [curve_of("podcast")]
    );
}

#[test]
fn a_changed_threshold_re_evaluates_alerts() {
    let mut h = Harness::new(persisted());
    let status = DeviceUpdate::Status {
        battery: percent(18),
        charging: Charging::Discharging,
        link: Link::Connected,
    };
    let _ = h.device(status);
    let mut settings = persisted().settings;
    settings.low_battery_threshold = percent(20);
    let actions = h.send(Input::SettingsChanged(settings));
    assert!(
        actions
            .iter()
            .any(|action| matches!(action, Action::Notify(_)))
    );
}

#[test]
fn clearing_the_adopted_identity_adopts_again() {
    let mut h = ready();
    let mut settings = adopted().settings;
    settings.adopted_identity = None;
    let actions = h.send(Input::SettingsChanged(settings));
    let values = PersistedChange::Headset(HeadsetValues::from_settings(&observed()));
    assert!(persisted_changes(&actions).contains(&values));
}
