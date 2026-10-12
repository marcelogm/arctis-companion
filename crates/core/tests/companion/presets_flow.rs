use arctis_core::companion::{Action, Input, PresetChange};
use arctis_core::eq::Curve;
use arctis_core::eq::preset::Preset;
use arctis_core::intent::Intent;
use arctis_core::messages::audio::{AudioCommand, SinkKind, SoundPath};
use arctis_core::messages::device::DeviceUpdate;

use crate::support::{
    Harness, START, adopted, audio_commands, curve_of, default_sink, device_curves, id, observed,
    observed_with, persisted, ready, unique_curve, wakes,
};

fn mine(curve: Curve) -> Preset {
    Preset::custom(id("mine"), "Mine".to_owned(), curve)
}

fn using_mine() -> Harness {
    let mut saved = adopted();
    saved.custom_presets = vec![mine(unique_curve())];
    saved.settings.route_presets.dongle = id("mine");
    let mut h = Harness::new(saved);
    let _ = h.connect(observed());
    let _ = h.device(DeviceUpdate::Observed(observed_with(unique_curve())));
    h
}

#[test]
fn saving_a_custom_preset_stores_it() {
    let mut h = Harness::new(persisted());
    let actions = h.intent(Intent::SavePreset(mine(unique_curve())));
    assert!(
        actions.contains(&Action::StorePreset(PresetChange::Save(mine(
            unique_curve()
        ))))
    );
    assert!(h.snapshot().presets.find(&id("mine")).is_some());
}

#[test]
fn built_in_presets_cannot_be_overwritten() {
    let mut h = Harness::new(persisted());
    let fake_flat = Preset::custom(id("flat"), "Flat".to_owned(), unique_curve());
    let actions = h.intent(Intent::SavePreset(fake_flat));
    assert!(
        !actions
            .iter()
            .any(|action| matches!(action, Action::StorePreset(_)))
    );
}

#[test]
fn editing_the_active_headset_preset_is_throttled() {
    let mut h = using_mine();
    let first = h
        .at(1_000)
        .intent(Intent::SavePreset(mine(curve_of("podcast"))));
    assert_eq!(device_curves(&first), [curve_of("podcast")]);
    let second = h
        .at(1_100)
        .intent(Intent::SavePreset(mine(curve_of("bass-boost"))));
    assert!(device_curves(&second).is_empty());
    assert_eq!(wakes(&second), [START.after(1_250)]);
    let third = h
        .at(1_200)
        .intent(Intent::SavePreset(mine(curve_of("v-shape"))));
    assert!(device_curves(&third).is_empty());
    let due = h.at(1_250).send(Input::Woke);
    assert_eq!(device_curves(&due), [curve_of("v-shape")]);
}

#[test]
fn editing_the_active_bluetooth_preset_retargets_at_once() {
    let mut saved = persisted();
    saved.custom_presets = vec![mine(unique_curve())];
    saved.settings.route_presets.bluetooth = id("mine");
    let mut h = Harness::new(saved);
    let inserted = h.audio(default_sink(SinkKind::NovaBluetooth));
    assert_eq!(
        audio_commands(&inserted),
        [AudioCommand::InsertEqualizer(unique_curve())]
    );
    for (offset, preset) in [(100, "podcast"), (150, "bass-boost")] {
        let edited = h
            .at(offset)
            .intent(Intent::SavePreset(mine(curve_of(preset))));
        assert_eq!(
            audio_commands(&edited),
            [AudioCommand::Retarget(curve_of(preset))]
        );
    }
}

#[test]
fn editing_an_inactive_preset_writes_nothing() {
    let mut h = ready();
    assert!(device_curves(&h.intent(Intent::SavePreset(mine(unique_curve())))).is_empty());
}

#[test]
fn deleting_the_active_preset_falls_back_to_flat() {
    let mut h = using_mine();
    let actions = h.intent(Intent::DeletePreset(id("mine")));
    assert!(actions.contains(&Action::StorePreset(PresetChange::Delete(id("mine")))));
    assert_eq!(device_curves(&actions), [curve_of("flat")]);
    let snapshot = h.snapshot();
    assert_eq!(snapshot.presets.route_presets.dongle, id("mine"));
    assert_eq!(snapshot.presets.route_preset(SoundPath::Dongle), id("flat"));
}

#[test]
fn undo_restores_the_deleted_preset() {
    let mut h = using_mine();
    let _ = h.intent(Intent::DeletePreset(id("mine")));
    let _ = h.headset_holds("flat");
    let restored = h.intent(Intent::SavePreset(mine(unique_curve())));
    assert_eq!(device_curves(&restored), [unique_curve()]);
}
