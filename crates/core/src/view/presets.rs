use super::text::{Text, preset_arg, preset_title};
use crate::companion::per_path::PATHS;
use crate::companion::snapshot::Snapshot;
use crate::eq::preset::{Preset, PresetId, PresetKind};
use crate::messages::audio::SoundPath;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetItem {
    pub id: PresetId,
    pub title: Text,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetsPageModel {
    pub built_in: Vec<PresetItem>,
    pub custom: Vec<PresetItem>,
    pub custom_description: Option<Text>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetUses {
    pub paths: Vec<SoundPath>,
    pub app_rules: u32,
}

#[must_use]
pub fn preset_items(snapshot: &Snapshot) -> Vec<PresetItem> {
    snapshot.presets.all.iter().map(item).collect()
}

#[must_use]
pub fn presets_page(snapshot: &Snapshot) -> PresetsPageModel {
    let (built_in, custom): (Vec<&Preset>, Vec<&Preset>) = snapshot
        .presets
        .all
        .iter()
        .partition(|preset| preset.kind() == PresetKind::BuiltIn);
    PresetsPageModel {
        custom_description: custom
            .is_empty()
            .then(|| Text::plain("Duplicate or import a preset to customize it")),
        built_in: built_in.into_iter().map(item).collect(),
        custom: custom.into_iter().map(item).collect(),
    }
}

#[must_use]
pub fn preset_uses(snapshot: &Snapshot, id: &PresetId) -> PresetUses {
    let presets = &snapshot.presets;
    let paths = PATHS
        .into_iter()
        .filter(|path| presets.route_presets.get(*path) == id)
        .collect();
    let rules = presets
        .rules
        .entries()
        .filter(|(_, preset)| *preset == id)
        .count();
    PresetUses {
        paths,
        app_rules: u32::try_from(rules).unwrap_or(u32::MAX),
    }
}

#[must_use]
pub fn deleted_toast(snapshot: &Snapshot, preset: &Preset) -> Text {
    let uses = preset_uses(snapshot, preset.id());
    let name = vec![("preset", preset_arg(preset))];
    if uses.app_rules == 0 {
        return Text::format(unruled_msgid(&uses.paths), name);
    }
    Text::plural(ruled_msgids(&uses.paths), uses.app_rules, name)
}

const fn unruled_msgid(paths: &[SoundPath]) -> &'static str {
    match paths {
        [] => "Deleted “{preset}”",
        [SoundPath::Dongle] => "Deleted “{preset}”. Used by the 2.4 GHz preset",
        [SoundPath::Bluetooth] => "Deleted “{preset}”. Used by the Bluetooth preset",
        _ => "Deleted “{preset}”. Used by the 2.4 GHz and Bluetooth presets",
    }
}

const fn ruled_msgids(paths: &[SoundPath]) -> [&'static str; 2] {
    match paths {
        [] => [
            "Deleted “{preset}”. Used by {n} app rule",
            "Deleted “{preset}”. Used by {n} app rules",
        ],
        [SoundPath::Dongle] => [
            "Deleted “{preset}”. Used by the 2.4 GHz preset and {n} app rule",
            "Deleted “{preset}”. Used by the 2.4 GHz preset and {n} app rules",
        ],
        [SoundPath::Bluetooth] => [
            "Deleted “{preset}”. Used by the Bluetooth preset and {n} app rule",
            "Deleted “{preset}”. Used by the Bluetooth preset and {n} app rules",
        ],
        _ => [
            "Deleted “{preset}”. Used by the 2.4 GHz and Bluetooth presets and {n} app rule",
            "Deleted “{preset}”. Used by the 2.4 GHz and Bluetooth presets and {n} app rules",
        ],
    }
}

fn item(preset: &Preset) -> PresetItem {
    PresetItem {
        id: preset.id().clone(),
        title: preset_title(preset),
    }
}
