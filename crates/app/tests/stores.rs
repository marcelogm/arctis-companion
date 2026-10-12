use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};

use arctis_app::platform::battery_store::FileBattery;
use arctis_app::platform::persisted::{file_stores, load_persisted};
use arctis_app::platform::preset_store::FilePresets;
use arctis_app::platform::seen_apps_store::FileSeenApps;
use arctis_core::battery::log_format::RETENTION_SECONDS;
use arctis_core::battery::sample::{Reading, Sample, UnixSeconds};
use arctis_core::companion::seen_apps::SeenApp;
use arctis_core::eq::library;
use arctis_core::eq::preset::{Preset, PresetId};
use arctis_core::ports::BatteryRepository;
use arctis_core::ports::PresetRepository;
use arctis_core::ports::SeenAppsRepository;
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::settings::values::Percent;
use arctis_core::testkit::repository_contracts::battery_repository_contract;
use arctis_core::testkit::repository_contracts::preset_repository_contract;
use arctis_core::testkit::repository_contracts::seen_apps_repository_contract;
use arctis_core::testkit::snapshots::snapshot;
use arctis_core::time::Millis;
use tempfile::TempDir;

struct Scratch {
    root: TempDir,
    next: Cell<u32>,
}

impl Scratch {
    fn new() -> Self {
        Self {
            root: tempfile::tempdir().expect("scratch dir"),
            next: Cell::new(0),
        }
    }

    fn fresh(&self) -> PathBuf {
        self.next.set(self.next.get() + 1);
        self.root.path().join(self.next.get().to_string())
    }
}

fn custom(id: &str) -> Preset {
    Preset::custom(
        PresetId::new(id),
        id.to_uppercase(),
        library::flat().curve().clone(),
    )
}

#[test]
fn file_presets_pass_the_repository_contract() {
    let scratch = Scratch::new();
    preset_repository_contract(
        || FilePresets::new(scratch.fresh()),
        |store| FilePresets::new(store.path().to_owned()),
    );
}

#[test]
fn a_preset_is_one_json_file_named_after_its_id() {
    let scratch = Scratch::new();
    let dir = scratch.fresh();
    let mut store = FilePresets::new(dir.clone());
    store.save(&custom("bass-boost-mine")).expect("save");
    let text = fs::read_to_string(dir.join("bass-boost-mine.json")).expect("preset file");
    assert!(text.contains("\"format\": 1"));
    assert!(text.contains("\"id\": \"bass-boost-mine\""));
}

#[test]
fn an_unreadable_preset_file_is_skipped() {
    let scratch = Scratch::new();
    let dir = scratch.fresh();
    let mut store = FilePresets::new(dir.clone());
    store.save(&custom("good")).expect("save");
    fs::write(dir.join("broken.json"), "{ not json").expect("broken file");
    fs::write(dir.join("notes.txt"), "ignored").expect("other file");
    assert_eq!(store.load().expect("load"), [custom("good")]);
}

#[test]
fn an_id_that_is_not_a_slug_never_becomes_a_path() {
    let scratch = Scratch::new();
    let dir = scratch.fresh();
    let mut store = FilePresets::new(dir.clone());
    assert!(store.save(&custom("../escape")).is_err());
    assert!(store.delete(&PresetId::new("../escape")).is_err());
    assert!(
        !dir.parent()
            .map(Path::to_owned)
            .expect("parent")
            .join("escape.json")
            .exists()
    );
}

#[test]
fn a_missing_directory_loads_as_empty() {
    let scratch = Scratch::new();
    assert_eq!(FilePresets::new(scratch.fresh()).load().expect("load"), []);
}

fn sample(at: u64, percent: u8) -> Sample {
    Sample {
        at: UnixSeconds(at),
        reading: Reading {
            percent: Percent::try_from(percent).expect("percent"),
            charging: Charging::Discharging,
            link: Link::Connected,
        },
    }
}

#[test]
fn file_battery_passes_the_repository_contract() {
    let scratch = Scratch::new();
    battery_repository_contract(
        || FileBattery::new(scratch.fresh().join("battery.csv")),
        |store| FileBattery::new(store.path().to_owned()),
    );
}

#[test]
fn battery_rows_are_csv_lines() {
    let scratch = Scratch::new();
    let path = scratch.fresh().join("battery.csv");
    let mut store = FileBattery::new(path.clone());
    store.append(&sample(1_760_000_000, 73)).expect("append");
    let text = fs::read_to_string(path).expect("csv");
    assert_eq!(text, "1760000000,73,discharging,connected\n");
}

#[test]
fn a_malformed_row_loses_only_itself() {
    let scratch = Scratch::new();
    let path = scratch.fresh().join("battery.csv");
    fs::create_dir_all(path.parent().expect("parent")).expect("dir");
    fs::write(
        &path,
        "1,90,discharging,connected\ngarbage\n2,89,discharging,connected\n",
    )
    .expect("csv");
    let store = FileBattery::new(path);
    assert_eq!(store.load().expect("load"), [sample(1, 90), sample(2, 89)]);
}

#[test]
fn file_seen_apps_pass_the_repository_contract() {
    let scratch = Scratch::new();
    seen_apps_repository_contract(
        || FileSeenApps::new(scratch.fresh().join("seen-apps.json")),
        |store| FileSeenApps::new(store.path().to_owned()),
    );
}

#[test]
fn an_unreadable_seen_apps_file_is_an_error_and_the_next_record_replaces_it() {
    let scratch = Scratch::new();
    let path = scratch.fresh().join("seen-apps.json");
    fs::create_dir_all(path.parent().expect("parent")).expect("dir");
    fs::write(&path, "[ broken").expect("file");
    let mut store = FileSeenApps::new(path);
    assert!(store.load().is_err());
    let firefox = SeenApp {
        key: "org.mozilla.firefox".to_owned(),
        name: "Firefox".to_owned(),
        identifiers: vec!["org.mozilla.firefox".to_owned(), "firefox".to_owned()],
    };
    store.record(&firefox).expect("record");
    assert_eq!(store.load().expect("load"), [firefox]);
}

const NOW_SECONDS: u64 = 1_760_000_000;

fn app_settings() -> arctis_core::companion::app_settings::AppSettings {
    let snapshot = snapshot();
    arctis_core::companion::app_settings::AppSettings {
        headset: snapshot.dongle.desired,
        route_presets: snapshot.presets.route_presets.clone(),
        app_rules: snapshot.presets.rules.clone(),
        automatic_presets: true,
        adopted_identity: None,
        bluetooth_sink_seen: false,
        low_battery_threshold: snapshot.battery.low_threshold,
    }
}

#[test]
fn start_up_drops_battery_rows_older_than_eight_days() {
    let scratch = Scratch::new();
    let dir = scratch.fresh();
    let mut stores = file_stores(&dir);
    let old = sample(NOW_SECONDS - RETENTION_SECONDS - 1, 90);
    let recent = sample(NOW_SECONDS - 60, 80);
    stores.battery.append(&old).expect("old");
    stores.battery.append(&recent).expect("recent");
    let persisted = load_persisted(app_settings(), &mut stores, Millis(NOW_SECONDS * 1_000));
    assert_eq!(persisted.battery, [recent]);
    assert_eq!(
        FileBattery::new(dir.join("battery.csv"))
            .load()
            .expect("load"),
        [recent]
    );
}

#[test]
fn start_up_reads_every_store_and_survives_a_broken_one() {
    let scratch = Scratch::new();
    let dir = scratch.fresh();
    let mut stores = file_stores(&dir);
    stores.presets.save(&custom("mine")).expect("preset");
    fs::write(dir.join("seen-apps.json"), "[ broken").expect("broken seen apps");
    let persisted = load_persisted(app_settings(), &mut stores, Millis(NOW_SECONDS * 1_000));
    assert_eq!(persisted.custom_presets, [custom("mine")]);
    assert_eq!(persisted.seen_apps, []);
    assert_eq!(persisted.settings, app_settings());
    assert!(dir.join("presets").join("mine.json").exists());
}
