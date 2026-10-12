use std::collections::BTreeMap;

use arctis_core::companion::Snapshot;
use arctis_core::companion::selection::PresetReason;
use arctis_core::view::text::preset_title;
use arctis_ui::text::resolve;
use gtk::glib;
use gtk::glib::prelude::*;

use super::state::{
    battery_level, charging_name, reason_pair, route_name, state_name, warning_name,
};

pub type PropertyMap = BTreeMap<&'static str, glib::Variant>;

#[must_use]
pub fn property_values(snapshot: &Snapshot) -> PropertyMap {
    let reading = snapshot.battery.reading;
    let (active, reason) = active_preset(snapshot);
    PropertyMap::from([
        ("State", state(snapshot).to_variant()),
        ("Model", model_name(snapshot).to_variant()),
        ("BatteryLevel", battery_level(reading).to_variant()),
        ("Charging", charging_name(reading).to_variant()),
        (
            "BatteryWarning",
            warning_name(snapshot.battery.warning).to_variant(),
        ),
        ("Route", route_name(snapshot.audio.route).to_variant()),
        ("Presets", preset_list(snapshot).to_variant()),
        ("ActivePreset", active.to_variant()),
        ("ActivePresetReason", reason.to_variant()),
        ("AppRuleCount", rule_count(snapshot).to_variant()),
        ("AutomaticPresets", snapshot.presets.automatic.to_variant()),
    ])
}

#[must_use]
pub fn changed(before: &PropertyMap, after: &PropertyMap) -> PropertyMap {
    after
        .iter()
        .filter(|(name, value)| before.get(*name) != Some(*value))
        .map(|(name, value)| (*name, value.clone()))
        .collect()
}

const fn state(snapshot: &Snapshot) -> &'static str {
    state_name(
        snapshot.dongle.phase,
        snapshot.dongle.adoption,
        snapshot.audio.route,
    )
}

fn model_name(snapshot: &Snapshot) -> &'static str {
    snapshot
        .dongle
        .model
        .as_ref()
        .map_or("", |model| model.name)
}

fn preset_list(snapshot: &Snapshot) -> Vec<(String, String)> {
    snapshot
        .presets
        .all
        .iter()
        .map(|preset| {
            (
                preset.id().as_str().to_owned(),
                resolve(&preset_title(preset)),
            )
        })
        .collect()
}

fn rule_count(snapshot: &Snapshot) -> u32 {
    u32::try_from(snapshot.presets.rules.len()).unwrap_or(u32::MAX)
}

fn active_preset(snapshot: &Snapshot) -> (String, (&'static str, String)) {
    match snapshot.picked_path() {
        Ok(path) => {
            let active = snapshot.presets.active.get(path);
            (
                active.preset.as_str().to_owned(),
                reason_pair(&active.reason),
            )
        }
        Err(_) => (String::new(), reason_pair(&PresetReason::Route)),
    }
}
