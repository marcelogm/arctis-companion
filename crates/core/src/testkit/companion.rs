use crate::companion::Input;
use crate::companion::app_settings::{AppRules, AppSettings, Persisted, RoutePresets};
use crate::eq::curve::Curve;
use crate::eq::library;
use crate::eq::reference::Overlays;
use crate::messages::audio::{AppIdentifiers, SoundPath, StreamInfo};
use crate::messages::device::DeviceUpdate;
use crate::protocol::status::{Charging, Link};

use super::snapshots::{id, percent, values};

pub const NOVA_5_PRODUCT: u16 = 0x2232;

#[must_use]
pub fn persisted() -> Persisted {
    Persisted {
        settings: AppSettings {
            headset: values(),
            route_presets: RoutePresets {
                dongle: id("flat"),
                bluetooth: id("flat"),
            },
            app_rules: AppRules::default(),
            automatic_presets: true,
            adopted_identity: None,
            bluetooth_sink_seen: false,
            low_battery_threshold: percent(15),
            overlays: Overlays::default(),
        },
        custom_presets: Vec::new(),
        seen_apps: Vec::new(),
        battery: Vec::new(),
    }
}

#[must_use]
pub fn with_rule(app: &str, preset: &str) -> Persisted {
    let mut saved = persisted();
    saved.settings.app_rules.set(app.to_owned(), id(preset));
    saved
}

#[must_use]
pub fn curve(preset: &str) -> Curve {
    library::find(&id(preset))
        .expect("built-in preset")
        .curve()
        .clone()
}

#[must_use]
pub fn firefox(path: Option<SoundPath>) -> StreamInfo {
    StreamInfo {
        ids: AppIdentifiers::new(Some("org.mozilla.firefox"), None, Some("firefox")),
        name: "Firefox".to_owned(),
        running: true,
        path,
    }
}

#[must_use]
pub fn status(level: u8, charging: Charging) -> Input {
    Input::Device(DeviceUpdate::Status {
        battery: percent(level),
        charging,
        link: Link::Connected,
    })
}
