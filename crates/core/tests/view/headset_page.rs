use arctis_core::battery::sample::Reading;
use arctis_core::companion::adoption::AdoptionStatus;
use arctis_core::companion::dongle::WriteFailure;
use arctis_core::companion::routing::Route;
use arctis_core::intent::Intent;
use arctis_core::messages::device::Phase;
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::settings::headset_settings::Setting;
use arctis_core::testkit::english::english;
use arctis_core::view::headset_page::{
    BannerKind, HeadsetModel, HeadsetPageModel, SettingsArea, StatusKind, banner, battery_icon,
    headset_page,
};

use crate::support::{percent, snapshot, with};

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
enum Page {
    NoHeadset,
    PermissionRequired,
    BluetoothOnly,
    Full,
}

fn page_of(model: &HeadsetPageModel) -> Page {
    match model {
        HeadsetPageModel::Status(StatusKind::NoHeadset, _) => Page::NoHeadset,
        HeadsetPageModel::Status(StatusKind::PermissionRequired, _) => Page::PermissionRequired,
        HeadsetPageModel::Headset(page) => match page.settings {
            SettingsArea::BluetoothOnly(_) => Page::BluetoothOnly,
            SettingsArea::Groups(_) | SettingsArea::Hidden => Page::Full,
        },
    }
}

const fn expected(phase: Phase, route: Route) -> Page {
    match (phase, route) {
        (Phase::Absent | Phase::PermissionDenied, Route::Bluetooth) => Page::BluetoothOnly,
        (Phase::Absent, _) => Page::NoHeadset,
        (Phase::PermissionDenied, _) => Page::PermissionRequired,
        _ => Page::Full,
    }
}

fn headset(model: HeadsetPageModel) -> HeadsetModel {
    match model {
        HeadsetPageModel::Headset(page) => *page,
        HeadsetPageModel::Status(..) => panic!("a headset page"),
    }
}

fn reading(value: u8, charging: Charging) -> Reading {
    Reading {
        percent: percent(value),
        charging,
        link: Link::Connected,
    }
}

#[test]
fn every_phase_and_route_gives_a_page() {
    for phase in PHASES {
        for route in ROUTES {
            let page = page_of(&headset_page(&with(phase, route)));
            assert_eq!(page, expected(phase, route), "{phase:?} × {route:?}");
        }
    }
}

#[test]
fn absent_with_bluetooth_keeps_the_equalizer_rows() {
    let mut s = with(Phase::Absent, Route::Bluetooth);
    s.audio.bluetooth_sink_seen = true;
    let page = headset(headset_page(&s));
    assert!(page.equalizer.bluetooth_preset.is_some());
    assert!(page.equalizer.dongle_preset.is_none());
    let SettingsArea::BluetoothOnly(notice) = page.settings else {
        panic!("Bluetooth-only page");
    };
    assert_eq!(
        english(&notice),
        "Connect the wireless dongle to change headset settings"
    );
}

#[test]
fn status_pages_explain_what_to_do() {
    let HeadsetPageModel::Status(_, absent) = headset_page(&with(Phase::Absent, Route::None))
    else {
        panic!("status page");
    };
    assert_eq!(english(&absent.title), "No Headset Connected");
    assert_eq!(
        english(&absent.description),
        "Connect the wireless dongle or pair the headset over Bluetooth"
    );
    let HeadsetPageModel::Status(_, denied) =
        headset_page(&with(Phase::PermissionDenied, Route::Dongle))
    else {
        panic!("status page");
    };
    assert_eq!(english(&denied.title), "Permission Required");
    assert_eq!(
        denied.button.map(|text| english(&text)).as_deref(),
        Some("Show Instructions")
    );
}

#[test]
fn the_banner_follows_its_priority() {
    let cases = [
        (
            Phase::PermissionDenied,
            Route::Bluetooth,
            AdoptionStatus::Unknown,
            Some(BannerKind::PermissionNeeded),
        ),
        (
            Phase::HeadsetOff,
            Route::Dongle,
            AdoptionStatus::NeverAdopted,
            Some(BannerKind::TurnOnToRead),
        ),
        (
            Phase::HeadsetOff,
            Route::None,
            AdoptionStatus::Adopted,
            Some(BannerKind::HeadsetOff),
        ),
        (Phase::Ready, Route::Dongle, AdoptionStatus::Adopted, None),
        (
            Phase::Absent,
            Route::Bluetooth,
            AdoptionStatus::Unknown,
            None,
        ),
    ];
    for (phase, route, adoption, expected) in cases {
        let mut s = with(phase, route);
        s.dongle.adoption = adoption;
        assert_eq!(
            banner(&s).map(|b| b.kind),
            expected,
            "{phase:?} {route:?} {adoption:?}"
        );
    }
}

#[test]
fn banners_say_what_happens() {
    let mut s = with(Phase::HeadsetOff, Route::Dongle);
    let off = banner(&s).expect("banner");
    assert_eq!(
        english(&off.title),
        "Headset is off. Changes apply when it turns on."
    );
    s.dongle.adoption = AdoptionStatus::NeverAdopted;
    let unread = banner(&s).expect("banner");
    assert_eq!(
        english(&unread.title),
        "Turn on the headset to read its settings"
    );
    let denied = banner(&with(Phase::PermissionDenied, Route::Bluetooth)).expect("banner");
    assert_eq!(
        english(&denied.title),
        "Permission is needed to use the wireless dongle"
    );
    assert_eq!(
        denied.button.map(|text| english(&text)).as_deref(),
        Some("Show Instructions")
    );
}

#[test]
fn a_never_adopted_headset_off_hides_the_headset_groups() {
    let mut s = with(Phase::HeadsetOff, Route::Dongle);
    s.dongle.adoption = AdoptionStatus::NeverAdopted;
    let page = headset(headset_page(&s));
    assert!(matches!(page.settings, SettingsArea::Hidden));
    assert!(page.equalizer.dongle_preset.is_none());
    assert_eq!(page.banner.map(|b| b.kind), Some(BannerKind::TurnOnToRead));
}

#[test]
fn the_header_shows_battery_and_route() {
    let header = headset(headset_page(&snapshot())).header;
    assert_eq!(
        header.battery.map(|text| english(&text)).as_deref(),
        Some("73 % · Discharging")
    );
    assert_eq!(
        english(&header.route),
        "Sound from this computer over 2.4 GHz"
    );
    assert_eq!(header.icon, "battery-level-70-symbolic");
    assert_eq!(
        english(&header.description),
        "Battery 73 %. Sound from this computer over 2.4 GHz"
    );
    assert_eq!((header.subtitle, header.busy), (None, false));
}

#[test]
fn every_route_has_its_header_text() {
    let cases = [
        (Route::Dongle, "Sound from this computer over 2.4 GHz"),
        (Route::Bluetooth, "Sound from this computer over Bluetooth"),
        (Route::None, "Not this computer's sound output"),
    ];
    for (route, expected) in cases {
        let header = headset(headset_page(&with(Phase::Ready, route))).header;
        assert_eq!(english(&header.route), expected);
    }
}

#[test]
fn the_battery_icon_follows_level_and_charging() {
    let cases = [
        (
            Some(reading(73, Charging::Discharging)),
            "battery-level-70-symbolic",
        ),
        (
            Some(reading(9, Charging::Discharging)),
            "battery-level-0-symbolic",
        ),
        (
            Some(reading(45, Charging::Charging)),
            "battery-level-40-charging-symbolic",
        ),
        (
            Some(reading(100, Charging::PluggedIn)),
            "battery-level-100-charged-symbolic",
        ),
        (None, "battery-missing-symbolic"),
    ];
    for (level, expected) in cases {
        assert_eq!(battery_icon(level), expected);
    }
}

#[test]
fn connecting_and_reconnecting_show_in_the_header() {
    let cases = [
        (Phase::Settling, Some("Connecting…")),
        (Phase::Probing, Some("Connecting…")),
        (Phase::Syncing, Some("Connecting…")),
        (Phase::Rebinding, Some("Reconnecting…")),
        (Phase::HeadsetOff, None),
        (Phase::Ready, None),
    ];
    for (phase, expected) in cases {
        let s = with(phase, Route::Dongle);
        let header = headset(headset_page(&s)).header;
        assert_eq!(
            header.subtitle.map(|text| english(&text)).as_deref(),
            expected,
            "{phase:?}"
        );
        assert_eq!(header.busy, expected.is_some());
    }
}

#[test]
fn an_unknown_battery_is_described() {
    let mut s = snapshot();
    s.battery.reading = Some(Reading {
        percent: percent(0),
        charging: Charging::Unknown,
        link: Link::HeadsetOff,
    });
    let header = headset(headset_page(&s)).header;
    assert_eq!(header.battery, None);
    assert_eq!(header.icon, "battery-missing-symbolic");
    assert_eq!(
        english(&header.description),
        "Battery level unknown. Sound from this computer over 2.4 GHz"
    );
}

#[test]
fn a_write_failure_offers_a_retry_toast() {
    let mut s = snapshot();
    s.dongle.last_failure = Some(WriteFailure {
        serial: 3,
        setting: Setting::Sidetone,
    });
    let toast = headset(headset_page(&s)).failure_toast.expect("toast");
    assert_eq!(toast.serial, 3);
    assert_eq!(english(&toast.title), "Could Not Change Sidetone");
    assert_eq!(toast.retry, Intent::RetryWrite);
}

#[test]
fn the_header_battery_names_the_charging_state() {
    let cases = [
        (reading(45, Charging::Charging), "45 % · Charging"),
        (reading(100, Charging::PluggedIn), "100 % · Charged"),
        (reading(50, Charging::Unknown), "50 %"),
    ];
    for (level, expected) in cases {
        let mut s = snapshot();
        s.battery.reading = Some(level);
        let header = headset(headset_page(&s)).header;
        assert_eq!(
            header.battery.map(|text| english(&text)).as_deref(),
            Some(expected)
        );
    }
}

#[test]
fn every_setting_has_its_failure_title() {
    let cases = [
        (Setting::MicVolume, "Could Not Change Microphone Volume"),
        (Setting::Sidetone, "Could Not Change Sidetone"),
        (Setting::MuteLight, "Could Not Change Mute Light Brightness"),
        (Setting::VolumeLimiter, "Could Not Change Volume Limiter"),
        (Setting::WirelessMode, "Could Not Change Extended Range"),
        (
            Setting::AutoPowerOff,
            "Could Not Change Automatic Power Off",
        ),
        (Setting::Equalizer, "Could Not Change the Headset Equalizer"),
    ];
    for (setting, expected) in cases {
        let mut s = snapshot();
        s.dongle.last_failure = Some(WriteFailure { serial: 1, setting });
        let toast = headset(headset_page(&s)).failure_toast.expect("toast");
        assert_eq!(english(&toast.title), expected);
    }
}
