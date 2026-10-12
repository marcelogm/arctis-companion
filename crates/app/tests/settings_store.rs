#[path = "common/schema.rs"]
mod schema;

use gtk::gio::prelude::*;

use schema::{compiled_dir, memory_pair};

const KEYS: [&str; 15] = [
    "mic-volume",
    "sidetone",
    "mute-light",
    "volume-limiter",
    "extended-range",
    "auto-power-off",
    "preset-dongle",
    "preset-bluetooth",
    "app-rules",
    "automatic-presets",
    "adopted-identity",
    "bluetooth-sink-seen",
    "low-battery-threshold",
    "run-in-background",
    "background-requested",
];

#[test]
fn the_schema_declares_every_key() {
    let dir = compiled_dir();
    let (settings, _) = memory_pair(dir.path());
    let schema = settings.settings_schema().expect("schema");
    for key in KEYS {
        assert!(schema.has_key(key), "{key}");
    }
    assert_eq!(schema.list_keys().len(), KEYS.len());
}
