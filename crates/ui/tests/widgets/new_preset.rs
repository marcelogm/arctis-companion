use std::rc::Rc;

use adw::prelude::*;
use arctis_core::eq::library;
use arctis_core::eq::preset::PresetKind;
use arctis_core::testkit::snapshots::{id, snapshot};
use arctis_ui::{EqEditorPage, PresetsPage, Window};
use gtk::glib;

use crate::support::{first, named, open_presets, rendered_window, saved, shown, visible_tag};

fn bound_action(window: &Window, trigger: &str) -> Option<String> {
    window
        .observe_controllers()
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|object| object.downcast::<gtk::ShortcutController>().ok())
        .flat_map(|controller| shortcuts_of(&controller))
        .find(|shortcut| shortcut.trigger().is_some_and(|t| t.to_str() == trigger))
        .and_then(|shortcut| shortcut.action())
        .and_then(|action| action.downcast::<gtk::NamedAction>().ok())
        .map(|action| action.action_name().to_string())
}

fn shortcuts_of(controller: &gtk::ShortcutController) -> Vec<gtk::Shortcut> {
    controller
        .iter::<glib::Object>()
        .filter_map(Result::ok)
        .filter_map(|object| object.downcast::<gtk::Shortcut>().ok())
        .collect()
}

#[gtk::test]
fn the_presets_page_offers_a_new_preset_button() {
    let (window, _) = rendered_window(snapshot());
    open_presets(&window);
    let button = named::<gtk::Button>(&first::<PresetsPage>(&window), "new-preset");
    assert!(shown(button.upcast_ref()));
    assert_eq!(button.action_name().as_deref(), Some("win.new-preset"));
    assert_eq!(button.tooltip_text().as_deref(), Some("New Preset"));
}

#[gtk::test]
fn control_n_creates_a_new_preset() {
    let (window, _) = rendered_window(snapshot());
    assert_eq!(
        bound_action(&window, "<Control>n").as_deref(),
        Some("win.new-preset")
    );
}

#[gtk::test]
fn a_new_preset_is_saved_flat_and_opened_for_editing() {
    let (window, log) = rendered_window(snapshot());
    open_presets(&window);
    WidgetExt::activate_action(&window, "win.new-preset", None).expect("the action exists");
    let preset = saved(&log);
    assert_eq!(preset.name(), "Custom Preset");
    assert_eq!(preset.id(), &id("custom-preset"));
    assert_eq!(preset.kind(), PresetKind::Custom);
    assert_eq!(preset.curve(), library::flat().curve());
    assert_eq!(log.borrow().len(), 1);
    assert_eq!(visible_tag(&window).as_deref(), Some("eq-editor"));
    let editor = first::<EqEditorPage>(&window);
    assert_eq!(editor.title(), "Custom Preset");
    assert!(shown(
        named::<gtk::MenuButton>(&editor, "editor-menu").upcast_ref()
    ));
    assert!(!shown(
        named::<gtk::Button>(&editor, "duplicate").upcast_ref()
    ));
}

#[gtk::test]
fn a_second_new_preset_gets_the_next_free_name() {
    let (window, log) = rendered_window(snapshot());
    WidgetExt::activate_action(&window, "win.new-preset", None).expect("the action exists");
    let mut s = snapshot();
    s.presets.all.push(saved(&log));
    window.render(&Rc::new(s));
    WidgetExt::activate_action(&window, "win.new-preset", None).expect("the action exists");
    let second = saved(&log);
    assert_eq!(second.name(), "Custom Preset 2");
    assert_eq!(second.id(), &id("custom-preset-2"));
    assert_eq!(first::<EqEditorPage>(&window).title(), "Custom Preset 2");
}
