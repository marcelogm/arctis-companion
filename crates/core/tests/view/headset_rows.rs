use arctis_core::companion::snapshot::DongleSnapshot;
use arctis_core::intent::Intent;
use arctis_core::messages::device::Phase;
use arctis_core::protocol::model::{Capabilities, Model, Support};
use arctis_core::settings::headset_settings::Setting;
use arctis_core::settings::values::{MuteLight, Sidetone, WirelessMode};
use arctis_core::testkit::english::english;
use arctis_core::view::headset_rows::{
    HeadsetGroupsModel, ScaleModel, headset_groups, sidetone_text,
};

use crate::support::{nova_5, snapshot};

const MIC_ONLY: Capabilities = Capabilities::new(&[Setting::MicVolume]);

fn groups(dongle: &DongleSnapshot) -> HeadsetGroupsModel {
    headset_groups(dongle).expect("headset groups")
}

fn mark_labels(scale: &ScaleModel) -> Vec<String> {
    scale
        .marks
        .iter()
        .map(|mark| english(&mark.label))
        .collect()
}

#[test]
fn sidetone_marks_are_off_low_medium_high() {
    let sidetone = groups(&snapshot().dongle).sidetone.expect("sidetone row");
    let values: Vec<u8> = sidetone.marks.iter().map(|mark| mark.value).collect();
    assert_eq!(values, [0, 3, 6, 10]);
    assert_eq!(mark_labels(&sidetone), ["Off", "Low", "Medium", "High"]);
    assert_eq!((sidetone.value, sidetone.max), (3, 10));
}

#[test]
fn sidetone_value_texts_name_the_marks_and_count_the_rest() {
    let cases = [
        (0, "Off"),
        (3, "Low"),
        (4, "4 of 10"),
        (6, "Medium"),
        (9, "9 of 10"),
        (10, "High"),
    ];
    for (value, expected) in cases {
        assert_eq!(english(&sidetone_text(value)), expected);
    }
}

#[test]
fn mic_volume_reads_as_a_step_of_fifteen() {
    let mic = groups(&snapshot().dongle).mic_volume.expect("mic row");
    assert_eq!((mic.value, mic.max), (7, 15));
    assert_eq!(english(&mic.value_text), "7 of 15");
    assert!(mic.marks.is_empty());
}

#[test]
fn mute_light_steps_are_labelled() {
    let light = groups(&snapshot().dongle)
        .mute_light
        .expect("mute light row");
    assert_eq!((light.value, light.max), (1, 3));
    assert_eq!(english(&light.value_text), "Low");
    assert_eq!(mark_labels(&light), ["Off", "Low", "Medium", "High"]);
}

#[test]
fn every_mute_light_step_turns_back_into_its_level() {
    for light in MuteLight::ALL {
        let mut s = snapshot();
        s.dongle.desired.mute_light = light;
        let row = groups(&s.dongle).mute_light.expect("mute light row");
        assert_eq!(
            Intent::from_scale(Setting::MuteLight, row.value),
            Some(Intent::SetMuteLight(light))
        );
    }
}

#[test]
fn switches_and_the_power_off_combo_show_desired_values() {
    let rows = groups(&snapshot().dongle);
    assert!(!rows.volume_limiter.expect("limiter row").active);
    let range = rows.extended_range.expect("range row");
    assert!(range.active);
    assert_eq!(
        range.subtitle.map(|text| english(&text)),
        Some("Increases range, lowers microphone quality".to_owned())
    );
    let power = rows.auto_power_off.expect("power row");
    assert_eq!(power.selected, 4);
    let first: Vec<String> = power.items.iter().take(3).map(english).collect();
    assert_eq!(first, ["Never", "1 minute", "5 minutes"]);
}

#[test]
fn a_row_not_applied_shows_the_observed_value() {
    let mut s = snapshot();
    let observed = s.dongle.observed.as_mut().expect("observed");
    observed.sidetone = Sidetone::try_from(6).expect("sidetone");
    observed.wireless_mode = WirelessMode::Standard;
    s.dongle.not_applied = vec![Setting::Sidetone, Setting::WirelessMode];
    let rows = groups(&s.dongle);
    let sidetone = rows.sidetone.expect("sidetone row");
    assert_eq!(sidetone.value, 6);
    assert_eq!(
        sidetone.subtitle.map(|text| english(&text)),
        Some("Not applied".to_owned())
    );
    assert_eq!(rows.mic_volume.expect("mic row").subtitle, None);
    let range = rows.extended_range.expect("range row");
    assert!(!range.active);
    assert_eq!(
        range.subtitle.map(|text| english(&text)),
        Some("Not applied".to_owned())
    );
}

#[test]
fn controls_wait_for_values_while_connecting() {
    let cases = [
        (Phase::Syncing, false, false),
        (Phase::Probing, true, true),
        (Phase::HeadsetOff, false, true),
        (Phase::Rebinding, false, true),
    ];
    for (phase, observed, sensitive) in cases {
        let mut s = snapshot();
        s.dongle.phase = phase;
        if !observed {
            s.dongle.observed = None;
        }
        assert_eq!(groups(&s.dongle).sensitive, sensitive, "{phase:?}");
    }
}

#[test]
fn rows_follow_the_model_capabilities() {
    let mut s = snapshot();
    s.dongle.model = Some(Model {
        capabilities: MIC_ONLY,
        ..nova_5()
    });
    let rows = groups(&s.dongle);
    assert!(rows.mic_volume.is_some());
    assert!(rows.sidetone.is_none() && rows.mute_light.is_none() && rows.volume_limiter.is_none());
    assert!(rows.extended_range.is_none() && rows.auto_power_off.is_none());
}

#[test]
fn the_device_group_names_model_and_firmware() {
    let device = groups(&snapshot().dongle).device;
    assert_eq!(english(&device.model), "Arctis Nova 5");
    assert_eq!(device.support, None);
    assert_eq!(english(&device.firmware), "Receiver 1.16.0, headset 0.9.0");
    let mut s = snapshot();
    s.dongle.model = Some(Model {
        support: Support::Untested,
        ..nova_5()
    });
    s.dongle.firmware = None;
    let untested = groups(&s.dongle).device;
    assert_eq!(
        untested.support.map(|text| english(&text)),
        Some("Untested model".to_owned())
    );
    assert_eq!(
        english(&untested.firmware),
        "Receiver Unknown, headset Unknown"
    );
}

#[test]
fn no_model_means_no_headset_groups() {
    let mut s = snapshot();
    s.dongle.model = None;
    assert!(headset_groups(&s.dongle).is_none());
}
