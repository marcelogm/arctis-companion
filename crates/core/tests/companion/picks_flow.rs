use arctis_core::companion::app_settings::{HeadsetValues, PersistedChange};
use arctis_core::companion::selection::{ActivateError, ActivePreset};
use arctis_core::companion::{Action, Input};
use arctis_core::intent::Intent;
use arctis_core::messages::audio::{AudioCommand, AudioEvent, SinkKind, SoundPath};
use arctis_core::messages::device::{DeviceUpdate, Phase, SyncPolicy};
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::settings::headset_settings::Setting;
use arctis_core::settings::values::{MicVolume, Sidetone};

use crate::support::{
    Harness, audio_commands, chosen_for, curve_of, default_sink, device_changes, device_curves,
    firefox_playing, firmware, id, nova_5, observed, percent, persisted, persisted_changes,
    picked_by_hand, playing, ready, ruled, with_rules,
};

fn firefox_joins(h: &mut Harness) {
    let _ = h.at(1_000).audio(playing(&["firefox"], SoundPath::Dongle));
    let _ = h.at(2_000).send(Input::Woke);
}

fn ten() -> Sidetone {
    Sidetone::try_from(10).expect("sidetone")
}

fn route_preset(path: SoundPath, preset: &str) -> PersistedChange {
    PersistedChange::RoutePreset(path, id(preset))
}

#[test]
fn a_headset_setting_change_is_persisted_and_enforced() {
    let mut h = ready();
    let actions = h.intent(Intent::SetSidetone(ten()));
    let mut values = HeadsetValues::from_settings(&observed());
    values.sidetone = ten();
    assert_eq!(
        persisted_changes(&actions),
        [PersistedChange::Headset(values)]
    );
    assert_eq!(
        device_changes(&actions),
        [(values.with_curve(curve_of("flat")), SyncPolicy::Enforce)]
    );
}

#[test]
fn headset_settings_cannot_change_before_adoption() {
    let mut h = Harness::new(persisted());
    let _ = h.device(DeviceUpdate::Phase(Phase::HeadsetOff));
    let _ = h.device(DeviceUpdate::Identity {
        model: nova_5(),
        uniq: None,
        firmware: firmware(16),
    });
    let actions = h.intent(Intent::SetSidetone(ten()));
    assert!(persisted_changes(&actions).is_empty());
    assert!(device_changes(&actions).is_empty());
    assert_eq!(h.snapshot().picked_path(), Err(ActivateError::NotReady));
    assert!(persisted_changes(&h.intent(Intent::ActivatePreset(id("podcast")))).is_empty());
}

#[test]
fn changes_queue_while_an_adopted_headset_is_off() {
    let mut h = ready();
    let _ = h.device(DeviceUpdate::Phase(Phase::HeadsetOff));
    let louder = MicVolume::try_from(12).expect("mic volume");
    let actions = h.intent(Intent::SetMicVolume(louder));
    let mut values = HeadsetValues::from_settings(&observed());
    values.mic_volume = louder;
    assert_eq!(
        device_changes(&actions),
        [(values.with_curve(curve_of("flat")), SyncPolicy::Enforce)]
    );
}

#[test]
fn retry_sends_the_desired_settings_again() {
    let mut h = ready();
    let actions = h.intent(Intent::RetryWrite(Setting::Sidetone));
    assert_eq!(
        device_changes(&actions),
        [(observed(), SyncPolicy::Enforce)]
    );
}

#[test]
fn the_combo_sets_the_route_preset_when_no_ruled_app_plays() {
    let mut h = ready();
    let actions = h.intent(Intent::SetRoutePreset(SoundPath::Dongle, id("podcast")));
    assert_eq!(
        persisted_changes(&actions),
        [route_preset(SoundPath::Dongle, "podcast")]
    );
    assert_eq!(device_curves(&actions), [curve_of("podcast")]);
    assert_eq!(
        h.snapshot().presets.dongle,
        ActivePreset::route(id("podcast"))
    );
}

#[test]
fn the_combo_with_a_ruled_app_playing_also_picks_by_hand() {
    let mut h = firefox_playing();
    let actions = h
        .at(5_000)
        .intent(Intent::SetRoutePreset(SoundPath::Dongle, id("podcast")));
    assert_eq!(
        persisted_changes(&actions),
        [route_preset(SoundPath::Dongle, "podcast")]
    );
    assert_eq!(device_curves(&actions), [curve_of("podcast")]);
    assert_eq!(
        h.snapshot().presets.dongle,
        picked_by_hand("podcast", Some("firefox"))
    );
}

#[test]
fn the_shell_sets_the_route_preset_when_no_ruled_app_plays() {
    let mut h = ready();
    let actions = h.intent(Intent::ActivatePreset(id("podcast")));
    assert_eq!(
        persisted_changes(&actions),
        [route_preset(SoundPath::Dongle, "podcast")]
    );
}

#[test]
fn the_shell_with_a_ruled_app_playing_picks_without_persisting() {
    let mut h = firefox_playing();
    let actions = h.at(5_000).intent(Intent::ActivatePreset(id("podcast")));
    assert!(persisted_changes(&actions).is_empty());
    assert_eq!(device_curves(&actions), [curve_of("podcast")]);
    assert_eq!(
        h.snapshot().presets.dongle,
        picked_by_hand("podcast", Some("firefox"))
    );
}

#[test]
fn a_shell_pick_does_not_survive_a_restart() {
    let mut h = firefox_playing();
    assert!(
        persisted_changes(&h.at(5_000).intent(Intent::ActivatePreset(id("podcast")))).is_empty()
    );
    let mut restarted = ruled(&[("firefox", "footsteps"), ("spotify", "bass-boost")]);
    firefox_joins(&mut restarted);
    assert_eq!(
        restarted.snapshot().presets.dongle,
        chosen_for("footsteps", "firefox")
    );
}

#[test]
fn a_combo_pick_survives_a_restart_only_as_the_route_preset() {
    let mut h = firefox_playing();
    let picked = h
        .at(5_000)
        .intent(Intent::SetRoutePreset(SoundPath::Dongle, id("podcast")));
    assert_eq!(
        persisted_changes(&picked),
        [route_preset(SoundPath::Dongle, "podcast")]
    );
    let mut saved = with_rules(&[("firefox", "footsteps")]);
    saved.settings.route_presets.dongle = id("podcast");
    let mut restarted = Harness::new(saved);
    let _ = restarted.connect(observed());
    assert_eq!(
        restarted.snapshot().presets.dongle,
        ActivePreset::route(id("podcast"))
    );
    firefox_joins(&mut restarted);
    assert_eq!(
        restarted.snapshot().presets.dongle,
        chosen_for("footsteps", "firefox")
    );
}

#[test]
fn a_new_ruled_app_ends_the_manual_pick() {
    let mut h = firefox_playing();
    let _ = h.at(5_000).intent(Intent::ActivatePreset(id("podcast")));
    let _ = h.headset_holds("podcast");
    let _ = h
        .at(10_000)
        .audio(playing(&["firefox", "spotify"], SoundPath::Dongle));
    let _ = h.at(11_000).send(Input::Woke);
    let switched = h.at(32_000).send(Input::Woke);
    assert_eq!(device_curves(&switched), [curve_of("bass-boost")]);
    assert_eq!(
        h.snapshot().presets.dongle,
        chosen_for("bass-boost", "spotify")
    );
}

#[test]
fn the_now_playing_button_picks_the_route_preset() {
    let mut h = firefox_playing();
    let actions = h
        .at(5_000)
        .intent(Intent::UseRoutePreset(SoundPath::Dongle));
    assert_eq!(device_curves(&actions), [curve_of("flat")]);
    assert_eq!(
        h.snapshot().presets.dongle,
        picked_by_hand("flat", Some("firefox"))
    );
}

#[test]
fn a_route_change_ends_the_manual_pick() {
    let mut h = firefox_playing();
    let _ = h.at(5_000).intent(Intent::ActivatePreset(id("podcast")));
    let _ = h.headset_holds("podcast");
    let actions = h.at(40_000).audio(default_sink(SinkKind::NovaDongle));
    assert_eq!(device_curves(&actions), [curve_of("footsteps")]);
}

#[test]
fn unknown_presets_and_missing_paths_are_ignored() {
    let mut h = ready();
    assert!(persisted_changes(&h.intent(Intent::ActivatePreset(id("nope")))).is_empty());
    let mut alone = Harness::new(persisted());
    assert!(persisted_changes(&alone.intent(Intent::ActivatePreset(id("podcast")))).is_empty());
}

#[test]
fn switching_automatic_presets_off_returns_to_the_route_preset() {
    let mut h = firefox_playing();
    let actions = h.at(5_000).intent(Intent::SetAutomaticPresets(false));
    assert_eq!(
        persisted_changes(&actions),
        [PersistedChange::AutomaticPresets(false)]
    );
    assert_eq!(device_curves(&actions), [curve_of("flat")]);
}

#[test]
fn app_rules_are_persisted() {
    let mut h = ready();
    let set = h.intent(Intent::SetAppRule {
        identifier: "firefox".to_owned(),
        preset: id("footsteps"),
    });
    let rule = PersistedChange::AppRule {
        identifier: "firefox".to_owned(),
        preset: Some(id("footsteps")),
    };
    assert_eq!(persisted_changes(&set), [rule]);
    assert_eq!(h.snapshot().presets.rules.len(), 1);
    let removed = h.intent(Intent::RemoveAppRule("firefox".to_owned()));
    let gone = PersistedChange::AppRule {
        identifier: "firefox".to_owned(),
        preset: None,
    };
    assert_eq!(persisted_changes(&removed), [gone]);
    assert!(h.snapshot().presets.rules.is_empty());
}

#[test]
fn a_new_low_battery_threshold_is_persisted_and_re_evaluated() {
    let mut h = Harness::new(persisted());
    let status = DeviceUpdate::Status {
        battery: percent(18),
        charging: Charging::Discharging,
        link: Link::Connected,
    };
    let _ = h.device(status);
    let actions = h.intent(Intent::SetLowBatteryThreshold(percent(20)));
    assert_eq!(
        persisted_changes(&actions),
        [PersistedChange::LowBatteryThreshold(percent(20))]
    );
    assert!(
        actions
            .iter()
            .any(|action| matches!(action, Action::Notify(_)))
    );
    assert_eq!(h.snapshot().battery.low_threshold, percent(20));
}

#[test]
fn preview_picks_the_edited_preset_until_the_page_closes() {
    let mut h = ready();
    let opened = h.intent(Intent::StartPreview(id("podcast")));
    assert_eq!(device_curves(&opened), [curve_of("podcast")]);
    assert_eq!(h.snapshot().presets.dongle, picked_by_hand("podcast", None));
    let _ = h.headset_holds("podcast");
    assert_eq!(
        device_curves(&h.intent(Intent::EndPreview)),
        [curve_of("flat")]
    );
}

#[test]
fn a_combo_pick_reaches_the_headset_while_pipewire_is_down() {
    let mut h = ready();
    let _ = h.audio(AudioEvent::Unavailable);
    let actions = h.intent(Intent::SetRoutePreset(SoundPath::Dongle, id("podcast")));
    assert_eq!(device_curves(&actions), [curve_of("podcast")]);
}

#[test]
fn run_in_background_is_left_to_the_bridge() {
    let mut h = ready();
    assert!(h.intent(Intent::SetRunInBackground(false)).is_empty());
}

#[test]
fn the_bluetooth_combo_retargets_the_system_equalizer() {
    let mut h = Harness::new(persisted());
    let _ = h.audio(default_sink(SinkKind::NovaBluetooth));
    let actions = h.intent(Intent::SetRoutePreset(SoundPath::Bluetooth, id("podcast")));
    assert_eq!(
        audio_commands(&actions),
        [AudioCommand::Retarget(curve_of("podcast"))]
    );
}
