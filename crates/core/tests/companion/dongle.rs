use arctis_core::companion::adoption::{AdoptionStatus, DongleIdentity};
use arctis_core::companion::dongle::{DongleState, WriteFailure};
use arctis_core::messages::device::Phase;
use arctis_core::settings::headset_settings::{Setting, SettingChange};
use arctis_core::settings::values::Sidetone;

use crate::support::{NOVA_5, curve_of, firmware, nova_5, observed, observed_with};

fn identified() -> DongleState {
    let mut dongle = DongleState::new();
    dongle.set_phase(Phase::Settling);
    dongle.identify(nova_5(), None, firmware(16));
    dongle
}

#[test]
fn adoption_compares_the_identity_with_the_adopted_one() {
    let adopted = DongleIdentity::new(NOVA_5, None);
    let other = DongleIdentity::new(NOVA_5, Some("ABC"));
    assert_eq!(
        DongleState::new().adoption(Some(&adopted)),
        AdoptionStatus::Unknown
    );
    assert_eq!(
        identified().adoption(Some(&adopted)),
        AdoptionStatus::Adopted
    );
    assert_eq!(
        identified().adoption(Some(&other)),
        AdoptionStatus::NeverAdopted
    );
    assert_eq!(identified().adoption(None), AdoptionStatus::NeverAdopted);
}

#[test]
fn a_firmware_update_keeps_the_identity() {
    let mut dongle = identified();
    dongle.identify(nova_5(), None, firmware(17));
    assert_eq!(dongle.identity, Some(DongleIdentity::new(NOVA_5, None)));
    assert_eq!(dongle.firmware, Some(firmware(17)));
}

#[test]
fn losing_the_dongle_forgets_it() {
    for phase in [Phase::Absent, Phase::PermissionDenied] {
        let mut dongle = identified();
        dongle.observe(observed(), &observed());
        dongle.set_phase(phase);
        assert!(!dongle.is_present());
        assert_eq!(
            (
                dongle.identity.clone(),
                dongle.model,
                dongle.observed.clone()
            ),
            (None, None, None)
        );
    }
}

#[test]
fn rebinding_keeps_the_dongle() {
    let mut dongle = identified();
    dongle.observe(observed(), &observed());
    dongle.set_phase(Phase::Rebinding);
    assert!(dongle.is_present());
    assert_eq!(dongle.observed, Some(observed()));
}

#[test]
fn a_write_failure_is_not_applied_until_observed_matches_desired() {
    let mut dongle = identified();
    dongle.write_failed(Setting::Sidetone);
    dongle.write_failed(Setting::Sidetone);
    assert_eq!(dongle.not_applied, [Setting::Sidetone]);
    assert_eq!(
        dongle.last_failure,
        Some(WriteFailure {
            serial: 2,
            setting: Setting::Sidetone
        })
    );
    let mut desired = observed();
    desired.sidetone = Sidetone::try_from(10).expect("sidetone");
    dongle.observe(observed(), &desired);
    assert_eq!(dongle.not_applied, [Setting::Sidetone]);
    dongle.observe(desired.clone(), &desired);
    assert!(dongle.not_applied.is_empty());
}

#[test]
fn the_equalizer_is_applied_when_the_curves_are_equivalent() {
    let mut dongle = identified();
    dongle.write_failed(Setting::Equalizer);
    dongle.observe(observed_with(curve_of("flat")), &observed());
    assert!(dongle.not_applied.is_empty());
}

#[test]
fn a_headset_change_updates_the_observed_copy() {
    let mut dongle = identified();
    dongle.observe(observed(), &observed());
    let ten = Sidetone::try_from(10).expect("sidetone");
    dongle.headset_changed(&SettingChange::Sidetone(ten));
    assert_eq!(dongle.observed.map(|o| o.sidetone), Some(ten));
}

#[test]
fn the_default_dongle_state_is_a_fresh_one() {
    assert_eq!(DongleState::default(), DongleState::new());
}
