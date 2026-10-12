use adw::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::companion::adoption::AdoptionStatus;
use arctis_core::companion::routing::Route;
use arctis_core::intent::Intent;
use arctis_core::messages::device::Phase;
use arctis_core::protocol::model::{Capabilities, Support};
use arctis_core::settings::headset_settings::Setting;
use arctis_core::settings::values::{AutoPowerOff, MicVolume, MuteLight, Sidetone};
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::{snapshot, values, with};
use arctis_core::view::headset_rows::headset_groups;
use arctis_ui::intents::IntentSink;
use arctis_ui::{HeadsetPage, ScaleRow};
use gtk::glib;

use crate::support::{
    Log, combo_text, first, has_value_text, headset_model, init, named, recording_sink,
};

const SETTING_GROUPS: [&str; 4] = [
    "microphone-group",
    "sound-group",
    "wireless-group",
    "device-group",
];

fn rendered(snapshot: &Snapshot) -> (HeadsetPage, IntentSink, Log) {
    init();
    let (sink, log) = recording_sink();
    let page: HeadsetPage = glib::Object::new();
    page.connect_intents(&sink);
    sink.rendering(|| page.render(&headset_model(snapshot)));
    (page, sink, log)
}

fn unadopted(adoption: AdoptionStatus) -> Snapshot {
    let mut s = with(Phase::HeadsetOff, Route::Dongle);
    s.dongle.adoption = adoption;
    s
}

fn unobserved(phase: Phase) -> Snapshot {
    let mut s = with(phase, Route::Dongle);
    s.dongle.observed = None;
    s
}

fn shown_groups(page: &HeadsetPage) -> Vec<&'static str> {
    SETTING_GROUPS
        .into_iter()
        .filter(|name| named::<adw::PreferencesGroup>(page, name).get_visible())
        .collect()
}

#[gtk::test]
fn a_ready_headset_shows_every_setting() {
    let (page, _, _) = rendered(&snapshot());
    assert_eq!(shown_groups(&page), SETTING_GROUPS.to_vec());
    assert!(has_value_text(&named::<ScaleRow>(&page, "sidetone"), "Low"));
    assert!(has_value_text(
        &named::<ScaleRow>(&page, "mic-volume"),
        "7 of 15"
    ));
    assert!(has_value_text(
        &named::<ScaleRow>(&page, "mute-light"),
        "Low"
    ));
    assert!(!named::<adw::SwitchRow>(&page, "volume-limiter").is_active());
    assert!(named::<adw::SwitchRow>(&page, "extended-range").is_active());
    assert_eq!(
        combo_text(&named::<adw::ComboRow>(&page, "auto-power-off")),
        "15 minutes"
    );
    assert!(!named::<adw::PreferencesGroup>(&page, "bluetooth-only-group").get_visible());
}

#[gtk::test]
fn the_device_group_shows_model_and_firmware() {
    let s = snapshot();
    let device = headset_groups(&s.dongle).expect("groups").device;
    let (page, _, _) = rendered(&s);
    assert_eq!(
        named::<adw::ActionRow>(&page, "model")
            .subtitle()
            .as_deref(),
        Some(english(&device.model).as_str())
    );
    assert_eq!(
        named::<adw::ActionRow>(&page, "firmware")
            .subtitle()
            .as_deref(),
        Some(english(&device.firmware).as_str())
    );
    assert!(!named::<adw::ActionRow>(&page, "support").get_visible());
}

#[gtk::test]
fn an_untested_model_shows_a_warning_row() {
    let mut s = snapshot();
    let mut model = s.dongle.model.expect("a model");
    model.support = Support::Untested;
    s.dongle.model = Some(model);
    let (page, _, _) = rendered(&s);
    let row = named::<adw::ActionRow>(&page, "support");
    assert!(row.get_visible());
    assert_eq!(row.title(), "Untested model");
}

#[gtk::test]
fn an_unadopted_headset_off_hides_the_headset_settings() {
    for adoption in [AdoptionStatus::NeverAdopted, AdoptionStatus::Unknown] {
        let (page, _, _) = rendered(&unadopted(adoption));
        assert!(shown_groups(&page).is_empty(), "{adoption:?}");
        assert!(!named::<adw::ComboRow>(&page, "auto-power-off").get_visible());
        assert!(!named::<adw::ComboRow>(&page, "dongle-preset").get_visible());
        assert!(named::<adw::ActionRow>(&page, "battery-history").get_visible());
    }
}

#[gtk::test]
fn bluetooth_only_replaces_the_settings_with_one_group() {
    let mut s = with(Phase::Absent, Route::Bluetooth);
    s.audio.bluetooth_sink_seen = true;
    let (page, _, _) = rendered(&s);
    let group = named::<adw::PreferencesGroup>(&page, "bluetooth-only-group");
    assert!(group.get_visible());
    assert_eq!(
        group.description().as_deref(),
        Some("Connect the wireless dongle to change headset settings")
    );
    assert!(shown_groups(&page).is_empty());
    assert!(!named::<adw::ComboRow>(&page, "dongle-preset").get_visible());
    assert!(named::<adw::ComboRow>(&page, "bluetooth-preset").get_visible());
    assert!(named::<adw::ActionRow>(&page, "battery-history").get_visible());
}

#[gtk::test]
fn settings_are_insensitive_while_connecting_until_values_are_observed() {
    for phase in [Phase::Settling, Phase::Probing, Phase::Syncing] {
        let (page, _, _) = rendered(&unobserved(phase));
        let group = named::<adw::PreferencesGroup>(&page, "microphone-group");
        assert!(!group.is_sensitive(), "{phase:?}");
        assert!(!named::<adw::ComboRow>(&page, "auto-power-off").is_sensitive());
    }
}

#[gtk::test]
fn settings_stay_sensitive_while_off_or_reconnecting() {
    for phase in [Phase::HeadsetOff, Phase::Rebinding] {
        let (page, _, _) = rendered(&unobserved(phase));
        let group = named::<adw::PreferencesGroup>(&page, "microphone-group");
        assert!(group.is_sensitive(), "{phase:?}");
        assert!(named::<adw::ComboRow>(&page, "auto-power-off").is_sensitive());
    }
}

#[gtk::test]
fn rows_the_model_lacks_are_omitted() {
    let mut s = snapshot();
    let mut model = s.dongle.model.expect("a model");
    model.capabilities = Capabilities::new(&[Setting::MicVolume, Setting::AutoPowerOff]);
    s.dongle.model = Some(model);
    let (page, _, _) = rendered(&s);
    assert_eq!(shown_groups(&page), ["microphone-group", "device-group"]);
    assert!(named::<ScaleRow>(&page, "mic-volume").get_visible());
    assert!(!named::<ScaleRow>(&page, "sidetone").get_visible());
    assert!(!named::<ScaleRow>(&page, "mute-light").get_visible());
    assert!(named::<adw::ComboRow>(&page, "auto-power-off").get_visible());
}

#[gtk::test]
fn a_setting_not_applied_shows_the_observed_value() {
    let mut s = snapshot();
    s.dongle.not_applied = vec![Setting::Sidetone];
    let mut observed = s.dongle.observed.clone().expect("observed");
    observed.sidetone = Sidetone::try_from(6).expect("sidetone");
    s.dongle.observed = Some(observed);
    let (page, _, _) = rendered(&s);
    let row = named::<ScaleRow>(&page, "sidetone");
    assert!(has_value_text(&row, "Medium"));
    assert_eq!(named::<gtk::Label>(&row, "subtitle").label(), "Not applied");
}

#[gtk::test]
fn each_setting_control_emits_its_intent() {
    let (page, _, log) = rendered(&snapshot());
    first::<gtk::Scale>(&named::<ScaleRow>(&page, "mic-volume")).set_value(9.0);
    first::<gtk::Scale>(&named::<ScaleRow>(&page, "sidetone")).set_value(6.0);
    first::<gtk::Scale>(&named::<ScaleRow>(&page, "mute-light")).set_value(3.0);
    named::<adw::SwitchRow>(&page, "volume-limiter").set_active(true);
    named::<adw::SwitchRow>(&page, "extended-range").set_active(false);
    named::<adw::ComboRow>(&page, "auto-power-off").set_selected(0);
    assert_eq!(
        *log.borrow(),
        vec![
            Intent::SetMicVolume(MicVolume::try_from(9).expect("volume")),
            Intent::SetSidetone(Sidetone::try_from(6).expect("sidetone")),
            Intent::SetMuteLight(MuteLight::High),
            Intent::SetVolumeLimiter(true),
            Intent::SetExtendedRange(false),
            Intent::SetAutoPowerOff(AutoPowerOff::NEVER),
        ]
    );
}

#[gtk::test]
fn rendering_new_values_emits_nothing() {
    let (page, sink, log) = rendered(&snapshot());
    let mut s = snapshot();
    let mut desired = values();
    desired.sidetone = Sidetone::try_from(10).expect("sidetone");
    desired.volume_limiter = true;
    desired.auto_power_off = AutoPowerOff::NEVER;
    s.dongle.desired = desired;
    s.dongle.observed =
        Some(desired.with_curve(s.dongle.observed.clone().expect("observed").equalizer));
    sink.rendering(|| page.render(&headset_model(&s)));
    assert!(has_value_text(
        &named::<ScaleRow>(&page, "sidetone"),
        "High"
    ));
    assert!(log.borrow().is_empty());
}
