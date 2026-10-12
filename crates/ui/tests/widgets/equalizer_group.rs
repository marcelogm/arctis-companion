use adw::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::companion::selection::{ActivePreset, PresetReason};
use arctis_core::intent::Intent;
use arctis_core::messages::audio::SoundPath;
use arctis_core::testkit::snapshots::{id, snapshot};
use arctis_core::view::equalizer_rows::equalizer_group;
use arctis_ui::EqualizerGroup;
use arctis_ui::intents::IntentSink;
use gtk::glib;

use crate::support::{Log, combo_text, descendants, init, named, recording_sink};

fn rendered(snapshot: &Snapshot) -> (EqualizerGroup, IntentSink, Log) {
    init();
    let (sink, log) = recording_sink();
    let group: EqualizerGroup = glib::Object::new();
    group.connect_intents(&sink);
    sink.rendering(|| group.render(&equalizer_group(snapshot)));
    (group, sink, log)
}

fn playing_for_firefox() -> Snapshot {
    let mut s = snapshot();
    s.presets.active.dongle = ActivePreset {
        preset: id("footsteps"),
        reason: PresetReason::App("Firefox".to_owned()),
    };
    s
}

fn warning_titles(group: &EqualizerGroup) -> Vec<String> {
    descendants(group)
        .into_iter()
        .filter(|widget| widget.widget_name() == "equalizer-warning")
        .filter_map(|widget| widget.downcast::<adw::ActionRow>().ok())
        .map(|row| row.title().to_string())
        .collect()
}

#[gtk::test]
fn the_dongle_combo_shows_the_route_preset() {
    let (group, _, _) = rendered(&snapshot());
    let row = named::<adw::ComboRow>(&group, "dongle-preset");
    assert!(row.get_visible());
    assert_eq!(row.title(), "2.4 GHz Preset");
    assert_eq!(combo_text(&row), "Flat");
}

#[gtk::test]
fn the_bluetooth_combo_needs_a_seen_bluez_sink() {
    let (group, _, _) = rendered(&snapshot());
    assert!(!named::<adw::ComboRow>(&group, "bluetooth-preset").get_visible());
    let mut s = snapshot();
    s.audio.bluetooth_sink_seen = true;
    let (group, _, _) = rendered(&s);
    let row = named::<adw::ComboRow>(&group, "bluetooth-preset");
    assert!(row.get_visible());
    assert_eq!(
        row.subtitle().as_deref(),
        Some("Applies when this computer plays to the headset over Bluetooth")
    );
}

#[gtk::test]
fn picking_a_preset_sets_the_route_preset() {
    let (group, _, log) = rendered(&snapshot());
    let row = named::<adw::ComboRow>(&group, "dongle-preset");
    let model = equalizer_group(&snapshot());
    let items = model.dongle_preset.expect("dongle combo").items;
    let target = items
        .iter()
        .position(|item| item.id == id("bass-boost"))
        .expect("Bass Boost is a built-in");
    row.set_selected(u32::try_from(target).expect("small index"));
    assert_eq!(
        *log.borrow(),
        vec![Intent::SetRoutePreset(SoundPath::Dongle, id("bass-boost"))]
    );
}

#[gtk::test]
fn now_playing_is_hidden_while_the_route_preset_plays() {
    let (group, _, _) = rendered(&snapshot());
    assert!(!named::<adw::ActionRow>(&group, "now-playing").get_visible());
}

#[gtk::test]
fn now_playing_names_the_app_and_offers_the_route_preset() {
    let (group, _, log) = rendered(&playing_for_firefox());
    let row = named::<adw::ActionRow>(&group, "now-playing");
    assert!(row.get_visible());
    assert_eq!(
        row.subtitle().as_deref(),
        Some("Footsteps, chosen for Firefox")
    );
    let button = named::<gtk::Button>(&group, "use-route-preset");
    assert_eq!(button.label().as_deref(), Some("Use Route Preset"));
    button.emit_clicked();
    assert_eq!(
        *log.borrow(),
        vec![Intent::UseRoutePreset(SoundPath::Dongle)]
    );
}

#[gtk::test]
fn the_switch_turns_automatic_presets_off() {
    let (group, _, log) = rendered(&snapshot());
    let row = named::<adw::SwitchRow>(&group, "automatic-presets");
    assert!(row.is_active());
    row.set_active(false);
    assert_eq!(*log.borrow(), vec![Intent::SetAutomaticPresets(false)]);
}

#[gtk::test]
fn warnings_appear_and_go() {
    let mut s = snapshot();
    s.audio.manager_denied = true;
    let (group, sink, _) = rendered(&s);
    assert_eq!(
        warning_titles(&group),
        vec!["Choose “Arctis Nova 5 (Equalizer)” as output in Sound settings".to_owned()]
    );
    sink.rendering(|| group.render(&equalizer_group(&snapshot())));
    assert!(warning_titles(&group).is_empty());
}

#[gtk::test]
fn rendering_a_new_route_preset_emits_nothing() {
    let (group, sink, log) = rendered(&snapshot());
    let mut s = playing_for_firefox();
    s.presets.route_presets.dongle = id("podcast");
    s.presets.automatic = false;
    sink.rendering(|| group.render(&equalizer_group(&s)));
    assert_eq!(
        combo_text(&named::<adw::ComboRow>(&group, "dongle-preset")),
        "Podcast"
    );
    assert!(log.borrow().is_empty());
}

#[gtk::test]
fn picking_a_bluetooth_preset_sets_its_route_preset() {
    let mut s = snapshot();
    s.audio.bluetooth_sink_seen = true;
    let (group, _, log) = rendered(&s);
    let items = equalizer_group(&s)
        .bluetooth_preset
        .expect("bluetooth combo")
        .items;
    let target = items
        .iter()
        .position(|item| item.id == id("podcast"))
        .expect("Podcast is a built-in");
    named::<adw::ComboRow>(&group, "bluetooth-preset")
        .set_selected(u32::try_from(target).expect("small index"));
    assert_eq!(
        *log.borrow(),
        vec![Intent::SetRoutePreset(SoundPath::Bluetooth, id("podcast"))]
    );
}
