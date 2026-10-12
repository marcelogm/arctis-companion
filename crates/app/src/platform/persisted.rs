use std::path::Path;

use arctis_core::battery::log_format::RETENTION_SECONDS;
use arctis_core::battery::sample::UnixSeconds;
use arctis_core::companion::app_settings::{AppSettings, Persisted};
use arctis_core::ports::StoreError;
use arctis_core::time::Millis;

use super::battery_store::FileBattery;
use super::preset_store::FilePresets;
use super::seen_apps_store::FileSeenApps;
use crate::logging::{report, warn};
use crate::ports::Stores;

pub const PRESETS_DIR: &str = "presets";
pub const BATTERY_FILE: &str = "battery.csv";
pub const SEEN_APPS_FILE: &str = "seen-apps.json";

#[must_use]
pub fn file_stores(dir: &Path) -> Stores {
    Stores {
        presets: Box::new(FilePresets::new(dir.join(PRESETS_DIR))),
        battery: Box::new(FileBattery::new(dir.join(BATTERY_FILE))),
        seen_apps: Box::new(FileSeenApps::new(dir.join(SEEN_APPS_FILE))),
    }
}

#[must_use]
pub fn load_persisted(settings: AppSettings, stores: &mut Stores, now: Millis) -> Persisted {
    let cutoff = UnixSeconds::from_millis(now).minus(RETENTION_SECONDS);
    report(stores.battery.retain_since(cutoff));
    Persisted {
        settings,
        custom_presets: loaded(stores.presets.load()),
        seen_apps: loaded(stores.seen_apps.load()),
        battery: loaded(stores.battery.load()),
    }
}

fn loaded<T>(result: Result<Vec<T>, StoreError>) -> Vec<T> {
    result.unwrap_or_else(|error| {
        warn(&format!("starting without stored data: {error}"));
        Vec::new()
    })
}
