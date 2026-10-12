use arctis_core::companion::Snapshot;
use arctis_core::messages::audio::SoundPath;
use arctis_core::testkit::english::english;
use arctis_core::view::presets::{
    PresetUses, deleted_toast, preset_items, preset_uses, presets_page,
};
use arctis_core::view::text::Text;

use crate::support::{custom, id, rules, snapshot};

fn using_mine(dongle: bool, bluetooth: bool, rule_count: usize) -> Snapshot {
    let mut s = snapshot();
    s.presets.all.push(custom("Mine"));
    for (uses, path) in [
        (dongle, SoundPath::Dongle),
        (bluetooth, SoundPath::Bluetooth),
    ] {
        if uses {
            s.presets.route_presets.set(path, id("mine"));
        }
    }
    let apps = ["firefox", "spotify", "mpv"];
    let entries: Vec<(&str, &str)> = apps
        .iter()
        .take(rule_count)
        .map(|app| (*app, "mine"))
        .collect();
    s.presets.rules = rules(&entries);
    s
}

#[test]
fn built_in_and_custom_presets_are_grouped() {
    let mut s = snapshot();
    s.presets.all.push(custom("Mine"));
    let page = presets_page(&s);
    assert_eq!(page.built_in.len(), 10);
    let custom_titles: Vec<Text> = page.custom.iter().map(|item| item.title.clone()).collect();
    assert_eq!(custom_titles, [Text::verbatim("Mine")]);
    assert_eq!(page.custom_description, None);
}

#[test]
fn an_empty_custom_group_explains_itself() {
    let description = presets_page(&snapshot())
        .custom_description
        .expect("description");
    assert_eq!(
        english(&description),
        "Create, duplicate or import a preset to customize it"
    );
}

#[test]
fn preset_items_list_every_preset_in_order() {
    let items = preset_items(&snapshot());
    assert_eq!(items.len(), 10);
    assert_eq!(items[0].id, id("flat"));
    assert_eq!(items[0].title, Text::plain("Flat"));
}

#[test]
fn uses_count_route_presets_and_rules() {
    let uses = preset_uses(&using_mine(true, false, 2), &id("mine"));
    assert_eq!(
        uses,
        PresetUses {
            paths: vec![SoundPath::Dongle],
            app_rules: 2
        }
    );
}

#[test]
fn the_delete_toast_names_the_uses() {
    let cases = [
        (false, false, 0, "Deleted “Mine”"),
        (true, false, 0, "Deleted “Mine”. Used by the 2.4 GHz preset"),
        (
            false,
            true,
            0,
            "Deleted “Mine”. Used by the Bluetooth preset",
        ),
        (
            true,
            true,
            0,
            "Deleted “Mine”. Used by the 2.4 GHz and Bluetooth presets",
        ),
        (
            false,
            true,
            1,
            "Deleted “Mine”. Used by the Bluetooth preset and 1 app rule",
        ),
        (
            true,
            false,
            1,
            "Deleted “Mine”. Used by the 2.4 GHz preset and 1 app rule",
        ),
        (
            true,
            true,
            2,
            "Deleted “Mine”. Used by the 2.4 GHz and Bluetooth presets and 2 app rules",
        ),
        (false, false, 1, "Deleted “Mine”. Used by 1 app rule"),
        (false, false, 3, "Deleted “Mine”. Used by 3 app rules"),
    ];
    for (dongle, bluetooth, rule_count, expected) in cases {
        let toast = deleted_toast(&using_mine(dongle, bluetooth, rule_count), &custom("Mine"));
        assert_eq!(english(&toast), expected);
    }
}
