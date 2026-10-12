use super::equalizer_rows::{EqualizerGroupModel, equalizer_group};
use super::headset_rows::{HeadsetGroupsModel, headset_groups};
use super::status_page::StatusPageModel;
use super::text::{Arg, Text};
use crate::battery::sample::Reading;
use crate::companion::adoption::AdoptionStatus;
use crate::companion::routing::Route;
use crate::companion::snapshot::Snapshot;
use crate::intent::Intent;
use crate::messages::device::Phase;
use crate::protocol::status::Charging;
use crate::settings::headset_settings::Setting;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StatusKind {
    NoHeadset,
    PermissionRequired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BannerKind {
    PermissionNeeded,
    TurnOnToRead,
    HeadsetOff,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BannerModel {
    pub kind: BannerKind,
    pub title: Text,
    pub button: Option<Text>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderModel {
    pub icon: String,
    pub battery: Option<Text>,
    pub route: Text,
    pub subtitle: Option<Text>,
    pub busy: bool,
    pub description: Text,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToastModel {
    pub serial: u32,
    pub title: Text,
    pub retry: Intent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsArea {
    Groups(Box<HeadsetGroupsModel>),
    Hidden,
    BluetoothOnly(Text),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadsetModel {
    pub banner: Option<BannerModel>,
    pub header: HeaderModel,
    pub equalizer: EqualizerGroupModel,
    pub settings: SettingsArea,
    pub failure_toast: Option<ToastModel>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeadsetPageModel {
    Status(StatusKind, Box<StatusPageModel>),
    Headset(Box<HeadsetModel>),
}

#[must_use]
pub fn headset_page(snapshot: &Snapshot) -> HeadsetPageModel {
    let bluetooth = snapshot.audio.route == Route::Bluetooth;
    match snapshot.dongle.phase {
        Phase::Absent if !bluetooth => {
            HeadsetPageModel::Status(StatusKind::NoHeadset, Box::new(no_headset()))
        }
        Phase::PermissionDenied if !bluetooth => HeadsetPageModel::Status(
            StatusKind::PermissionRequired,
            Box::new(permission_required()),
        ),
        _ => HeadsetPageModel::Headset(Box::new(headset(snapshot))),
    }
}

#[must_use]
pub fn banner(snapshot: &Snapshot) -> Option<BannerModel> {
    let dongle = &snapshot.dongle;
    let headset_off = dongle.phase == Phase::HeadsetOff;
    let adopted = dongle.adoption == AdoptionStatus::Adopted;
    let denied_on_bluetooth =
        dongle.phase == Phase::PermissionDenied && snapshot.audio.route == Route::Bluetooth;
    [
        (denied_on_bluetooth, BannerKind::PermissionNeeded),
        (headset_off && !adopted, BannerKind::TurnOnToRead),
        (headset_off && adopted, BannerKind::HeadsetOff),
    ]
    .into_iter()
    .find(|(applies, _)| *applies)
    .map(|(_, kind)| banner_model(kind))
}

#[must_use]
pub fn battery_icon(reading: Option<Reading>) -> String {
    let Some(reading) = reading else {
        return "battery-missing-symbolic".to_owned();
    };
    let level = reading.percent.get() / 10 * 10;
    match reading.charging {
        Charging::PluggedIn if level == 100 => "battery-level-100-charged-symbolic".to_owned(),
        Charging::Charging | Charging::PluggedIn => {
            format!("battery-level-{level}-charging-symbolic")
        }
        Charging::Discharging | Charging::Unknown => format!("battery-level-{level}-symbolic"),
    }
}

#[must_use]
pub const fn route_msgid(route: Route) -> &'static str {
    match route {
        Route::Dongle => "Sound from this computer over 2.4 GHz",
        Route::Bluetooth => "Sound from this computer over Bluetooth",
        Route::None => "Not this computer's sound output",
    }
}

fn headset(snapshot: &Snapshot) -> HeadsetModel {
    HeadsetModel {
        banner: banner(snapshot),
        header: header(snapshot),
        equalizer: equalizer_group(snapshot),
        settings: settings_area(snapshot),
        failure_toast: failure_toast(snapshot),
    }
}

fn no_headset() -> StatusPageModel {
    StatusPageModel::new(
        "audio-headphones-symbolic",
        "No Headset Connected",
        "Connect the wireless dongle or pair the headset over Bluetooth",
        None,
    )
}

fn permission_required() -> StatusPageModel {
    StatusPageModel::new(
        "dialog-password-symbolic",
        "Permission Required",
        "The wireless dongle needs a udev rule before the app can use it",
        Some("Show Instructions"),
    )
}

fn banner_model(kind: BannerKind) -> BannerModel {
    let (title, button) = match kind {
        BannerKind::PermissionNeeded => (
            "Permission is needed to use the wireless dongle",
            Some("Show Instructions"),
        ),
        BannerKind::TurnOnToRead => ("Turn on the headset to read its settings", None),
        BannerKind::HeadsetOff => ("Headset is off. Changes apply when it turns on.", None),
    };
    BannerModel {
        kind,
        title: Text::plain(title),
        button: button.map(Text::plain),
    }
}

fn settings_area(snapshot: &Snapshot) -> SettingsArea {
    let dongle = &snapshot.dongle;
    if !dongle.is_present() {
        return SettingsArea::BluetoothOnly(Text::plain(
            "Connect the wireless dongle to change headset settings",
        ));
    }
    if dongle.adoption != AdoptionStatus::Adopted {
        return SettingsArea::Hidden;
    }
    headset_groups(dongle).map_or(SettingsArea::Hidden, |groups| {
        SettingsArea::Groups(Box::new(groups))
    })
}

fn header(snapshot: &Snapshot) -> HeaderModel {
    let present = snapshot.dongle.is_present();
    let reading = snapshot
        .battery
        .reading
        .filter(|level| level.is_connected() && present);
    let route = route_msgid(snapshot.audio.route);
    let subtitle = connection_subtitle(snapshot);
    HeaderModel {
        icon: battery_icon(reading),
        battery: reading.map(battery_text),
        route: Text::plain(route),
        busy: subtitle.is_some(),
        subtitle,
        description: header_description(reading, route),
    }
}

fn connection_subtitle(snapshot: &Snapshot) -> Option<Text> {
    match snapshot.dongle.phase {
        phase if phase.is_connecting() => Some(Text::plain("Connecting…")),
        Phase::Rebinding => Some(Text::plain("Reconnecting…")),
        _ => None,
    }
}

fn battery_text(reading: Reading) -> Text {
    let level = ("percent", Arg::Percent(reading.percent));
    match charging_msgid(reading.charging) {
        Some(state) => Text::format(
            "{percent} · {state}",
            vec![level, ("state", Arg::Msgid(state.to_owned()))],
        ),
        None => Text::format("{percent}", vec![level]),
    }
}

const fn charging_msgid(charging: Charging) -> Option<&'static str> {
    match charging {
        Charging::Charging => Some("Charging"),
        Charging::PluggedIn => Some("Charged"),
        Charging::Discharging => Some("Discharging"),
        Charging::Unknown => None,
    }
}

fn header_description(reading: Option<Reading>, route: &str) -> Text {
    let route = ("route", Arg::Msgid(route.to_owned()));
    match reading {
        Some(level) => Text::format(
            "Battery {percent}. {route}",
            vec![("percent", Arg::Percent(level.percent)), route],
        ),
        None => Text::format("Battery level unknown. {route}", vec![route]),
    }
}

fn failure_toast(snapshot: &Snapshot) -> Option<ToastModel> {
    let failure = snapshot.dongle.last_failure?;
    snapshot
        .dongle
        .not_applied
        .contains(&failure.setting)
        .then(|| ToastModel {
            serial: failure.serial,
            title: Text::plain(failure_msgid(failure.setting)),
            retry: Intent::RetryWrite,
        })
}

const fn failure_msgid(setting: Setting) -> &'static str {
    match setting {
        Setting::MicVolume => "Could Not Change Microphone Volume",
        Setting::Sidetone => "Could Not Change Sidetone",
        Setting::MuteLight => "Could Not Change Mute Light Brightness",
        Setting::VolumeLimiter => "Could Not Change Volume Limiter",
        Setting::WirelessMode => "Could Not Change Extended Range",
        Setting::AutoPowerOff => "Could Not Change Automatic Power Off",
        Setting::Equalizer => "Could Not Change the Headset Equalizer",
    }
}
