use std::rc::Rc;

use adw::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::companion::adoption::AdoptionStatus;
use arctis_core::companion::routing::Route;
use arctis_core::intent::Intent;
use arctis_core::messages::device::Phase;
use arctis_core::settings::values::Sidetone;
use arctis_core::testkit::snapshots::{id, snapshot, values, with};
use arctis_core::view::preferences::Background;
use arctis_ui::{ScaleRow, Window};
use gtk::{gio, glib};

use crate::support::{first, init, named, rendered_window};

const PHASES: [Phase; 8] = [
    Phase::Absent,
    Phase::PermissionDenied,
    Phase::Settling,
    Phase::Probing,
    Phase::HeadsetOff,
    Phase::Syncing,
    Phase::Ready,
    Phase::Rebinding,
];
const ROUTES: [Route; 3] = [Route::Dongle, Route::Bluetooth, Route::None];

#[derive(Debug, PartialEq, Eq)]
enum Shown {
    NoHeadset,
    PermissionRequired,
    BluetoothOnly,
    Full,
}

fn expected(phase: Phase, route: Route) -> Shown {
    match (phase, route) {
        (Phase::Absent | Phase::PermissionDenied, Route::Bluetooth) => Shown::BluetoothOnly,
        (Phase::Absent, _) => Shown::NoHeadset,
        (Phase::PermissionDenied, _) => Shown::PermissionRequired,
        _ => Shown::Full,
    }
}

fn status_kind(title: &str) -> Shown {
    match title {
        "No Headset Connected" => Shown::NoHeadset,
        "Permission Required" => Shown::PermissionRequired,
        other => panic!("unexpected status page {other}"),
    }
}

fn headset_kind(window: &Window) -> Shown {
    if named::<adw::PreferencesGroup>(window, "bluetooth-only-group").get_visible() {
        return Shown::BluetoothOnly;
    }
    assert!(named::<adw::PreferencesGroup>(window, "microphone-group").get_visible());
    Shown::Full
}

fn shown_page(window: &Window) -> Shown {
    let stack = named::<gtk::Stack>(window, "main-stack");
    if stack.visible_child_name().as_deref() == Some("status") {
        return status_kind(&first::<adw::StatusPage>(window).title());
    }
    headset_kind(window)
}

fn never_adopted(phase: Phase) -> Snapshot {
    let mut s = with(phase, Route::Dongle);
    s.dongle.adoption = AdoptionStatus::NeverAdopted;
    s
}

fn new_window() -> Window {
    init();
    glib::Object::new()
}

#[gtk::test]
fn window_is_titled_arctis_companion() {
    assert_eq!(new_window().title().as_deref(), Some("Arctis Companion"));
}

#[gtk::test]
fn window_keeps_the_minimum_size_of_the_design() {
    let window = new_window();
    assert_eq!(
        (window.width_request(), window.height_request()),
        (360, 294)
    );
}

fn banner_of(window: &Window) -> Option<(String, Option<String>)> {
    let banner = first::<adw::Banner>(window);
    banner.is_revealed().then(|| {
        (
            banner.title().to_string(),
            banner.button_label().map(String::from),
        )
    })
}

fn owned(title: &str, button: Option<&str>) -> (String, Option<String>) {
    (title.to_owned(), button.map(str::to_owned))
}

#[gtk::test]
fn every_phase_and_route_shows_its_page() {
    let cases = PHASES
        .into_iter()
        .flat_map(|phase| ROUTES.map(|route| (phase, route)));
    for (phase, route) in cases {
        let (window, _) = rendered_window(with(phase, route));
        assert_eq!(
            shown_page(&window),
            expected(phase, route),
            "{phase:?} × {route:?}"
        );
    }
}

#[gtk::test]
fn the_banner_shows_what_the_model_picks() {
    let cases = [
        (
            with(Phase::PermissionDenied, Route::Bluetooth),
            Some(owned(
                "Permission is needed to use the wireless dongle",
                Some("Show Instructions"),
            )),
        ),
        (
            never_adopted(Phase::HeadsetOff),
            Some(owned("Turn on the headset to read its settings", None)),
        ),
        (
            with(Phase::HeadsetOff, Route::Dongle),
            Some(owned(
                "Headset is off. Changes apply when it turns on.",
                None,
            )),
        ),
        (with(Phase::Ready, Route::Dongle), None),
    ];
    for (snapshot, banner) in cases {
        let (window, _) = rendered_window(snapshot);
        assert_eq!(banner_of(&window), banner);
    }
}

#[gtk::test]
fn an_unknown_adoption_after_a_restart_asks_to_turn_the_headset_on() {
    let mut s = with(Phase::HeadsetOff, Route::Dongle);
    s.dongle.adoption = AdoptionStatus::Unknown;
    let (window, _) = rendered_window(s);
    assert_eq!(
        banner_of(&window),
        Some(owned("Turn on the headset to read its settings", None))
    );
}

#[gtk::test]
fn navigation_rows_target_their_pages() {
    let (window, _) = rendered_window(snapshot());
    let rows = [
        ("presets-row", "presets"),
        ("app-presets-row", "app-presets"),
        ("battery-history", "battery"),
    ];
    for (name, tag) in rows {
        let row = named::<adw::ActionRow>(&window, name);
        assert_eq!(
            row.action_name().as_deref(),
            Some("win.show-page"),
            "{name}"
        );
        let target = row
            .action_target_value()
            .and_then(|value| value.get::<String>());
        assert_eq!(target.as_deref(), Some(tag), "{name}");
    }
}

#[gtk::test]
fn the_banner_button_opens_the_instructions() {
    let (window, _) = rendered_window(with(Phase::PermissionDenied, Route::Bluetooth));
    assert_eq!(
        first::<adw::Banner>(&window).action_name().as_deref(),
        Some("win.permission-instructions")
    );
}

#[gtk::test]
fn a_status_page_hides_the_banner() {
    let (window, _) = rendered_window(with(Phase::PermissionDenied, Route::Bluetooth));
    window.render(&Rc::new(with(Phase::Absent, Route::Dongle)));
    assert_eq!(banner_of(&window), None);
}

#[gtk::test]
fn the_permission_status_page_opens_the_instructions() {
    let (window, _) = rendered_window(with(Phase::PermissionDenied, Route::Dongle));
    let button = named::<gtk::Button>(&window, "status-button");
    assert!(button.get_visible());
    assert_eq!(
        button.action_name().as_deref(),
        Some("win.permission-instructions")
    );
}

#[gtk::test]
fn a_never_adopted_headset_off_hides_the_headset_groups() {
    let (window, _) = rendered_window(never_adopted(Phase::HeadsetOff));
    for name in [
        "microphone-group",
        "sound-group",
        "wireless-group",
        "device-group",
    ] {
        assert!(
            !named::<adw::PreferencesGroup>(&window, name).get_visible(),
            "{name}"
        );
    }
    assert!(!named::<adw::ComboRow>(&window, "dongle-preset").get_visible());
}

#[gtk::test]
fn a_control_reaches_the_intent_handler() {
    let (window, log) = rendered_window(snapshot());
    first::<gtk::Scale>(&named::<ScaleRow>(&window, "sidetone")).set_value(6.0);
    assert_eq!(
        *log.borrow(),
        vec![Intent::SetSidetone(
            Sidetone::try_from(6).expect("sidetone")
        )]
    );
}

#[gtk::test]
fn rendering_a_changed_snapshot_emits_nothing() {
    let (window, log) = rendered_window(snapshot());
    let mut s = snapshot();
    let mut desired = values();
    desired.sidetone = Sidetone::try_from(10).expect("sidetone");
    desired.volume_limiter = true;
    s.dongle.desired = desired;
    s.presets.route_presets.dongle = id("bass-boost");
    s.presets.automatic = false;
    s.battery.low_threshold = arctis_core::testkit::snapshots::percent(30);
    window.render(&Rc::new(s));
    window.set_background(Background::Denied);
    assert!(log.borrow().is_empty());
}

fn open_presets(window: &Window) {
    WidgetExt::activate_action(window, "win.show-page", Some(&"presets".to_variant()))
        .expect("the action exists");
}

#[gtk::test]
fn navigation_rows_open_their_page_once_it_exists() {
    let (window, _) = rendered_window(snapshot());
    let navigation = named::<adw::NavigationView>(&window, "navigation");
    let tag = || {
        navigation
            .visible_page()
            .and_then(|page| page.tag())
            .map(String::from)
    };
    open_presets(&window);
    assert_eq!(tag().as_deref(), Some("main"));
    navigation.add(&adw::NavigationPage::with_tag(
        &gtk::Label::new(None),
        "Presets",
        "presets",
    ));
    open_presets(&window);
    assert_eq!(tag().as_deref(), Some("presets"));
}

#[gtk::test]
fn the_background_state_reaches_the_preferences() {
    let (window, log) = rendered_window(snapshot());
    window.set_background(Background::Denied);
    let row = named::<adw::SwitchRow>(&window.preferences_dialog(), "run-in-background");
    assert!(!row.is_active());
    assert_eq!(
        row.subtitle().as_deref(),
        Some("Allow it in Settings → Apps → Arctis Companion")
    );
    row.set_active(true);
    assert_eq!(*log.borrow(), vec![Intent::SetRunInBackground(true)]);
}

#[gtk::test]
fn the_primary_menu_holds_preferences_shortcuts_and_about() {
    let (window, _) = rendered_window(snapshot());
    let menu = named::<gtk::MenuButton>(&window, "primary-menu")
        .menu_model()
        .expect("a menu model");
    let section = menu
        .item_link(0, gio::MENU_LINK_SECTION)
        .expect("one section");
    let actions: Vec<String> = (0..section.n_items())
        .filter_map(|item| {
            section.item_attribute_value(
                item,
                gio::MENU_ATTRIBUTE_ACTION,
                Some(glib::VariantTy::STRING),
            )
        })
        .filter_map(|value| value.get::<String>())
        .collect();
    assert_eq!(actions, ["win.preferences", "win.shortcuts", "win.about"]);
}
