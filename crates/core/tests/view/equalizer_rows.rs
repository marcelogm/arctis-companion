use arctis_core::companion::adoption::AdoptionStatus;
use arctis_core::companion::routing::{AudioService, Route};
use arctis_core::companion::selection::{ActivePreset, PresetReason};
use arctis_core::intent::Intent;
use arctis_core::messages::audio::{DefaultSink, SinkKind, SoundPath};
use arctis_core::messages::device::Phase;
use arctis_core::testkit::english::english;
use arctis_core::view::equalizer_rows::{EqualizerGroupModel, equalizer_group};

use crate::support::{custom, id, snapshot};

fn active(preset: &str, reason: PresetReason) -> ActivePreset {
    ActivePreset {
        preset: id(preset),
        reason,
    }
}

fn app(name: &str) -> PresetReason {
    PresetReason::App(name.to_owned())
}

fn now_playing_text(group: &EqualizerGroupModel) -> Option<String> {
    group.now_playing.as_ref().map(|row| english(&row.subtitle))
}

#[test]
fn the_dongle_combo_shows_the_route_preset() {
    let mut s = snapshot();
    s.presets.route_presets.dongle = id("podcast");
    let combo = equalizer_group(&s).dongle_preset.expect("2.4 GHz row");
    assert_eq!(english(&combo.title), "2.4 GHz Preset");
    assert_eq!(combo.items[combo.selected].id, id("podcast"));
    assert_eq!(combo.subtitle, None);
}

#[test]
fn the_dongle_combo_needs_a_present_adopted_dongle() {
    let cases = [
        (Phase::Ready, AdoptionStatus::NeverAdopted, false),
        (Phase::Absent, AdoptionStatus::Adopted, false),
        (Phase::HeadsetOff, AdoptionStatus::Adopted, true),
        (Phase::Rebinding, AdoptionStatus::Adopted, true),
    ];
    for (phase, adoption, shown) in cases {
        let mut s = snapshot();
        s.dongle.phase = phase;
        s.dongle.adoption = adoption;
        assert_eq!(
            equalizer_group(&s).dongle_preset.is_some(),
            shown,
            "{phase:?} {adoption:?}"
        );
    }
}

#[test]
fn the_bluetooth_row_appears_after_a_bluez_node_was_seen() {
    let mut s = snapshot();
    assert!(equalizer_group(&s).bluetooth_preset.is_none());
    s.audio.bluetooth_sink_seen = true;
    let combo = equalizer_group(&s).bluetooth_preset.expect("Bluetooth row");
    assert_eq!(english(&combo.title), "Bluetooth from This Computer");
    let subtitle = combo.subtitle.map(|text| english(&text));
    assert_eq!(
        subtitle.as_deref(),
        Some("Applies when this computer plays to the headset over Bluetooth")
    );
}

#[test]
fn now_playing_explains_an_app_preset() {
    let mut s = snapshot();
    s.presets.active.dongle = active("footsteps", app("Firefox"));
    let group = equalizer_group(&s);
    assert_eq!(
        now_playing_text(&group).as_deref(),
        Some("Footsteps, chosen for Firefox")
    );
    let row = group.now_playing.expect("Now Playing");
    assert_eq!(english(&row.button), "Use Route Preset");
    assert_eq!(
        row.use_route_preset,
        Intent::UseRoutePreset(SoundPath::Dongle)
    );
}

#[test]
fn now_playing_explains_a_manual_pick() {
    let cases = [
        (
            PresetReason::Manual(Some("Firefox".to_owned())),
            "Bass Boost, until Firefox stops",
        ),
        (PresetReason::Manual(None), "Bass Boost, chosen by hand"),
    ];
    for (reason, expected) in cases {
        let mut s = snapshot();
        s.presets.active.dongle = active("bass-boost", reason);
        assert_eq!(
            now_playing_text(&equalizer_group(&s)).as_deref(),
            Some(expected)
        );
    }
}

#[test]
fn now_playing_hides_while_the_route_preset_plays() {
    let mut s = snapshot();
    assert!(equalizer_group(&s).now_playing.is_none());
    s.presets.active.dongle = active("flat", app("Firefox"));
    assert!(equalizer_group(&s).now_playing.is_none());
}

#[test]
fn now_playing_hides_while_a_held_route_preset_plays() {
    let mut s = snapshot();
    s.presets.active.dongle = ActivePreset::route(id("podcast"));
    assert!(equalizer_group(&s).now_playing.is_none());
}

#[test]
fn now_playing_follows_the_picked_path() {
    let mut s = snapshot();
    s.audio.route = Route::Bluetooth;
    s.presets.active.dongle = active("footsteps", app("Firefox"));
    s.presets.active.bluetooth = active("podcast", app("Spotify"));
    let row = equalizer_group(&s).now_playing.expect("Now Playing");
    assert_eq!(row.path, SoundPath::Bluetooth);
    assert_eq!(english(&row.subtitle), "Podcast, chosen for Spotify");
}

#[test]
fn a_custom_preset_keeps_its_name() {
    let mut s = snapshot();
    s.presets.all.push(custom("Mine"));
    s.presets.active.dongle = active("mine", app("Firefox"));
    assert_eq!(
        now_playing_text(&equalizer_group(&s)).as_deref(),
        Some("Mine, chosen for Firefox")
    );
}

#[test]
fn warning_rows_name_each_problem() {
    let mut s = snapshot();
    s.audio.manager_denied = true;
    s.audio.default_sink = Some(DefaultSink {
        terminal: SinkKind::NovaBluetooth,
        via_equalizer: false,
        via_easy_effects: true,
    });
    s.audio.service = AudioService::Unavailable;
    let warnings: Vec<String> = equalizer_group(&s).warnings.iter().map(english).collect();
    assert_eq!(
        warnings,
        [
            "Choose “Arctis Nova 5 (Equalizer)” as output in Sound settings",
            "EasyEffects is processing audio; the system equalizer is off",
            "App presets unavailable: audio service not reachable",
        ]
    );
}

#[test]
fn the_automatic_switch_mirrors_the_setting() {
    let mut s = snapshot();
    s.presets.automatic = false;
    assert!(!equalizer_group(&s).automatic_presets);
}
