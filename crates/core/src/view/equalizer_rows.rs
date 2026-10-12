use super::presets::{PresetItem, preset_items};
use super::text::{Arg, Text, preset_arg};
use crate::companion::adoption::AdoptionStatus;
use crate::companion::routing::AudioService;
use crate::companion::selection::PresetReason;
use crate::companion::snapshot::Snapshot;
use crate::eq::preset::Preset;
use crate::intent::Intent;
use crate::messages::audio::SoundPath;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetComboModel {
    pub path: SoundPath,
    pub title: Text,
    pub subtitle: Option<Text>,
    pub items: Vec<PresetItem>,
    pub selected: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NowPlayingModel {
    pub path: SoundPath,
    pub subtitle: Text,
    pub button: Text,
    pub use_route_preset: Intent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EqualizerGroupModel {
    pub dongle_preset: Option<PresetComboModel>,
    pub bluetooth_preset: Option<PresetComboModel>,
    pub now_playing: Option<NowPlayingModel>,
    pub automatic_presets: bool,
    pub warnings: Vec<Text>,
}

#[must_use]
pub fn equalizer_group(snapshot: &Snapshot) -> EqualizerGroupModel {
    let dongle_ready =
        snapshot.dongle.is_present() && snapshot.dongle.adoption == AdoptionStatus::Adopted;
    EqualizerGroupModel {
        dongle_preset: dongle_ready.then(|| preset_combo(snapshot, SoundPath::Dongle)),
        bluetooth_preset: snapshot
            .audio
            .bluetooth_sink_seen
            .then(|| preset_combo(snapshot, SoundPath::Bluetooth)),
        now_playing: now_playing(snapshot),
        automatic_presets: snapshot.presets.automatic,
        warnings: warnings(snapshot),
    }
}

#[must_use]
pub fn now_playing_text(preset: &Preset, reason: &PresetReason) -> Text {
    let name = ("preset", preset_arg(preset));
    match reason {
        PresetReason::App(app) => Text::format(
            "{preset}, chosen for {app}",
            vec![name, ("app", Arg::Str(app.clone()))],
        ),
        PresetReason::Manual(Some(app)) => Text::format(
            "{preset}, until {app} stops",
            vec![name, ("app", Arg::Str(app.clone()))],
        ),
        PresetReason::Manual(None) | PresetReason::Route => {
            Text::format("{preset}, chosen by hand", vec![name])
        }
    }
}

fn preset_combo(snapshot: &Snapshot, path: SoundPath) -> PresetComboModel {
    let items = preset_items(snapshot);
    let route_preset = snapshot.presets.route_preset(path);
    let selected = items
        .iter()
        .position(|item| item.id == route_preset)
        .unwrap_or(0);
    let (title, subtitle) = match path {
        SoundPath::Dongle => ("2.4 GHz Preset", None),
        SoundPath::Bluetooth => (
            "Bluetooth from This Computer",
            Some("Applies when this computer plays to the headset over Bluetooth"),
        ),
    };
    PresetComboModel {
        path,
        title: Text::plain(title),
        subtitle: subtitle.map(Text::plain),
        items,
        selected,
    }
}

fn now_playing(snapshot: &Snapshot) -> Option<NowPlayingModel> {
    let path = snapshot.picked_path().ok()?;
    let active = snapshot.presets.active(path);
    if active.preset == snapshot.presets.route_preset(path) {
        return None;
    }
    let preset = snapshot.presets.resolve(&active.preset);
    Some(NowPlayingModel {
        path,
        subtitle: now_playing_text(&preset, &active.reason),
        button: Text::plain("Use Route Preset"),
        use_route_preset: Intent::UseRoutePreset(path),
    })
}

fn warnings(snapshot: &Snapshot) -> Vec<Text> {
    let audio = &snapshot.audio;
    let easy_effects = audio.default_sink.is_some_and(|sink| sink.via_easy_effects);
    [
        (
            audio.manager_denied,
            "Choose “Arctis Nova 5 (Equalizer)” as output in Sound settings",
        ),
        (
            easy_effects,
            "EasyEffects is processing audio; the system equalizer is off",
        ),
        (
            audio.service == AudioService::Unavailable,
            "App presets unavailable: audio service not reachable",
        ),
    ]
    .into_iter()
    .filter(|(applies, _)| *applies)
    .map(|(_, msgid)| Text::plain(msgid))
    .collect()
}
