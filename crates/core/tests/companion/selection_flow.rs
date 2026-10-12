use arctis_core::companion::Input;
use arctis_core::companion::selection::ActivePreset;
use arctis_core::intent::Intent;
use arctis_core::messages::audio::{AudioCommand, SinkKind, SoundPath};
use arctis_core::settings::values::Sidetone;

use crate::support::{
    Harness, START, audio_commands, chosen_for, curve_of, custom, default_sink, device_curves, id,
    observed, playing, ruled, wakes, with_rules,
};

#[test]
fn a_ruled_app_switches_the_headset_after_one_second() {
    let mut h = ruled(&[("firefox", "footsteps")]);
    let started = h.at(1_000).audio(playing(&["firefox"], SoundPath::Dongle));
    assert!(device_curves(&started).is_empty());
    assert_eq!(wakes(&started), [START.after(2_000)]);
    let joined = h.at(2_000).send(Input::Woke);
    assert_eq!(device_curves(&joined), [curve_of("footsteps")]);
    assert_eq!(
        h.snapshot().presets.active.dongle,
        chosen_for("footsteps", "firefox")
    );
}

#[test]
fn the_route_preset_returns_ten_seconds_after_the_app_stops() {
    let mut h = ruled(&[("firefox", "footsteps")]);
    let _ = h.at(1_000).audio(playing(&["firefox"], SoundPath::Dongle));
    let _ = h.at(2_000).send(Input::Woke);
    let _ = h.headset_holds("footsteps");
    let stopped = h.at(40_000).audio(playing(&[], SoundPath::Dongle));
    assert_eq!(wakes(&stopped), [START.after(50_000)]);
    let left = h.at(50_000).send(Input::Woke);
    assert_eq!(device_curves(&left), [curve_of("flat")]);
    assert_eq!(
        h.snapshot().presets.active.dongle,
        ActivePreset::route(id("flat"))
    );
}

#[test]
fn pausing_and_resuming_within_ten_seconds_changes_nothing() {
    let mut h = ruled(&[("firefox", "footsteps")]);
    let _ = h.at(1_000).audio(playing(&["firefox"], SoundPath::Dongle));
    let _ = h.at(2_000).send(Input::Woke);
    let _ = h.headset_holds("footsteps");
    let _ = h.at(3_000).audio(playing(&[], SoundPath::Dongle));
    let _ = h.at(8_000).audio(playing(&["firefox"], SoundPath::Dongle));
    assert!(device_curves(&h.at(13_000).send(Input::Woke)).is_empty());
    assert_eq!(
        h.snapshot().presets.active.dongle,
        chosen_for("footsteps", "firefox")
    );
}

#[test]
fn the_app_that_joined_last_chooses() {
    let mut h = ruled(&[("firefox", "footsteps"), ("spotify", "podcast")]);
    let _ = h.audio(default_sink(SinkKind::NovaBluetooth));
    let _ = h
        .at(1_000)
        .audio(playing(&["firefox"], SoundPath::Bluetooth));
    let first = h.at(2_000).send(Input::Woke);
    assert_eq!(
        audio_commands(&first),
        [AudioCommand::Retarget(curve_of("footsteps"))]
    );
    let _ = h
        .at(5_000)
        .audio(playing(&["firefox", "spotify"], SoundPath::Bluetooth));
    let second = h.at(6_000).send(Input::Woke);
    assert_eq!(
        audio_commands(&second),
        [AudioCommand::Retarget(curve_of("podcast"))]
    );
    assert_eq!(
        h.snapshot().presets.active.bluetooth,
        chosen_for("podcast", "spotify")
    );
}

#[test]
fn the_headset_switches_at_most_once_per_thirty_seconds() {
    let mut h = ruled(&[
        ("firefox", "footsteps"),
        ("spotify", "podcast"),
        ("mpv", "bass-boost"),
    ]);
    let _ = h.at(1_000).audio(playing(&["firefox"], SoundPath::Dongle));
    let _ = h.at(2_000).send(Input::Woke);
    let _ = h.headset_holds("footsteps");
    let _ = h
        .at(5_000)
        .audio(playing(&["firefox", "spotify"], SoundPath::Dongle));
    let held = h.at(6_000).send(Input::Woke);
    assert!(device_curves(&held).is_empty());
    assert_eq!(wakes(&held), [START.after(32_000)]);
    assert_eq!(
        h.snapshot().presets.active.get(SoundPath::Dongle),
        &chosen_for("footsteps", "firefox")
    );
    let _ = h
        .at(10_000)
        .audio(playing(&["firefox", "spotify", "mpv"], SoundPath::Dongle));
    let _ = h.at(11_000).send(Input::Woke);
    let coalesced = h.at(32_000).send(Input::Woke);
    assert_eq!(device_curves(&coalesced), [curve_of("bass-boost")]);
}

#[test]
fn the_same_preset_for_an_app_only_relabels() {
    let mut h = ruled(&[("firefox", "flat"), ("spotify", "podcast")]);
    let _ = h.at(1_000).audio(playing(&["firefox"], SoundPath::Dongle));
    assert!(device_curves(&h.at(2_000).send(Input::Woke)).is_empty());
    assert_eq!(
        h.snapshot().presets.active.dongle,
        chosen_for("flat", "firefox")
    );
    let _ = h
        .at(5_000)
        .audio(playing(&["firefox", "spotify"], SoundPath::Dongle));
    assert_eq!(
        device_curves(&h.at(6_000).send(Input::Woke)),
        [curve_of("podcast")]
    );
}

#[test]
fn a_switch_to_the_curve_on_the_headset_spends_no_budget() {
    let mut saved = with_rules(&[("firefox", "copy"), ("spotify", "podcast")]);
    saved.custom_presets = vec![custom("copy", curve_of("flat"))];
    let mut h = Harness::new(saved);
    let _ = h.connect(observed());
    let _ = h.at(1_000).audio(playing(&["firefox"], SoundPath::Dongle));
    assert_eq!(
        device_curves(&h.at(2_000).send(Input::Woke)),
        [curve_of("flat")]
    );
    assert_eq!(
        h.snapshot().presets.active.dongle,
        chosen_for("copy", "firefox")
    );
    let _ = h
        .at(5_000)
        .audio(playing(&["firefox", "spotify"], SoundPath::Dongle));
    assert_eq!(
        device_curves(&h.at(6_000).send(Input::Woke)),
        [curve_of("podcast")]
    );
}

#[test]
fn a_switch_back_before_the_readback_still_reaches_the_session() {
    let mut h = ruled(&[("firefox", "footsteps")]);
    let _ = h.at(1_000).audio(playing(&["firefox"], SoundPath::Dongle));
    assert_eq!(
        device_curves(&h.at(2_000).send(Input::Woke)),
        [curve_of("footsteps")]
    );
    let _ = h.at(3_000).audio(playing(&[], SoundPath::Dongle));
    assert!(device_curves(&h.at(13_000).send(Input::Woke)).is_empty());
    assert_eq!(
        device_curves(&h.at(32_000).send(Input::Woke)),
        [curve_of("flat")]
    );
}

#[test]
fn a_rule_naming_a_missing_preset_never_switches() {
    let mut h = ruled(&[("firefox", "deleted")]);
    let started = h.at(1_000).audio(playing(&["firefox"], SoundPath::Dongle));
    assert!(wakes(&started).is_empty());
    assert_eq!(
        h.snapshot().presets.active.dongle,
        ActivePreset::route(id("flat"))
    );
}

#[test]
fn rules_wait_while_automatic_presets_are_off() {
    let mut saved = with_rules(&[("firefox", "footsteps")]);
    saved.settings.automatic_presets = false;
    let mut h = Harness::new(saved);
    let _ = h.connect(observed());
    let _ = h.at(1_000).audio(playing(&["firefox"], SoundPath::Dongle));
    assert!(device_curves(&h.at(2_000).send(Input::Woke)).is_empty());
}

#[test]
fn a_rule_applies_only_on_its_path() {
    let mut h = ruled(&[("firefox", "footsteps")]);
    let _ = h
        .at(1_000)
        .audio(playing(&["firefox"], SoundPath::Bluetooth));
    assert!(device_curves(&h.at(2_000).send(Input::Woke)).is_empty());
    assert_eq!(
        h.snapshot().presets.active.bluetooth,
        chosen_for("footsteps", "firefox")
    );
}

#[test]
fn an_unrelated_intent_leaves_a_held_switch_held() {
    let mut h = ruled(&[("firefox", "footsteps"), ("spotify", "podcast")]);
    let _ = h.at(1_000).audio(playing(&["firefox"], SoundPath::Dongle));
    let _ = h.at(2_000).send(Input::Woke);
    let _ = h.headset_holds("footsteps");
    let _ = h
        .at(5_000)
        .audio(playing(&["firefox", "spotify"], SoundPath::Dongle));
    let _ = h.at(6_000).send(Input::Woke);
    let sidetone = Sidetone::try_from(5).expect("sidetone");
    let unrelated = h.at(7_000).intent(Intent::SetSidetone(sidetone));
    assert_eq!(device_curves(&unrelated), [curve_of("footsteps")]);
    assert_eq!(
        device_curves(&h.at(32_000).send(Input::Woke)),
        [curve_of("podcast")]
    );
}

#[test]
fn an_unrelated_intent_leaves_a_held_bluetooth_switch_held() {
    let mut h = ruled(&[("firefox", "footsteps"), ("spotify", "podcast")]);
    let _ = h.audio(default_sink(SinkKind::NovaBluetooth));
    let _ = h
        .at(1_000)
        .audio(playing(&["firefox"], SoundPath::Bluetooth));
    let _ = h.at(2_000).send(Input::Woke);
    let _ = h
        .at(2_500)
        .audio(playing(&["firefox", "spotify"], SoundPath::Bluetooth));
    let _ = h.at(3_500).send(Input::Woke);
    let unrelated = h.at(3_600).intent(Intent::SetAutomaticPresets(true));
    assert!(audio_commands(&unrelated).is_empty());
    assert_eq!(
        audio_commands(&h.at(4_000).send(Input::Woke)),
        [AudioCommand::Retarget(curve_of("podcast"))]
    );
}
