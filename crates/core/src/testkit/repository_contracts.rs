use crate::battery::sample::{Reading, Sample, UnixSeconds};
use crate::companion::seen_apps::{MAX_SEEN_APPS, SeenApp};
use crate::eq::library;
use crate::eq::preset::{Preset, PresetId};
use crate::ports::{BatteryRepository, PresetRepository, SeenAppsRepository};
use crate::protocol::status::{Charging, Link};
use crate::settings::values::Percent;

pub fn preset_repository_contract<R: PresetRepository>(
    fresh: impl Fn() -> R,
    reopen: impl Fn(R) -> R,
) {
    assert_eq!(loaded_presets(&fresh()), Vec::new());
    saved_presets_load_back(&reopen(saved(fresh())));
    saving_an_id_again_replaces_it(&reopen(saved_twice(fresh())));
    deleted_presets_are_gone(&reopen(deleted(fresh())));
}

pub fn battery_repository_contract<R: BatteryRepository>(
    fresh: impl Fn() -> R,
    reopen: impl Fn(R) -> R,
) {
    assert_eq!(fresh().load().expect("samples load"), Vec::new());
    appended_samples_load_back_in_order(&reopen(appended(fresh())));
    retention_drops_only_older_samples(&reopen(retained(fresh())));
}

pub fn seen_apps_repository_contract<R: SeenAppsRepository>(
    fresh: impl Fn() -> R,
    reopen: impl Fn(R) -> R,
) {
    assert_eq!(fresh().load().expect("seen apps load"), Vec::new());
    recorded_apps_load_newest_first(&reopen(recorded(fresh())));
    at_most_fifty_apps_are_kept(&reopen(recorded_many(fresh())));
}

fn custom(id: &str, name: &str) -> Preset {
    Preset::custom(
        PresetId::new(id),
        name.to_owned(),
        library::flat().curve().clone(),
    )
}

fn loaded_presets(store: &impl PresetRepository) -> Vec<Preset> {
    let mut presets = store.load().expect("presets load");
    presets.sort_by(|a, b| a.id().cmp(b.id()));
    presets
}

fn saved<R: PresetRepository>(mut store: R) -> R {
    store.save(&custom("b", "B")).expect("save b");
    store.save(&custom("a", "A")).expect("save a");
    store
}

fn saved_presets_load_back(store: &impl PresetRepository) {
    assert_eq!(
        loaded_presets(store),
        [custom("a", "A"), custom("b", "B")],
        "presets survive a reopen"
    );
}

fn saved_twice<R: PresetRepository>(mut store: R) -> R {
    store.save(&custom("a", "A")).expect("save a");
    store.save(&custom("a", "Renamed")).expect("save a again");
    store
}

fn saving_an_id_again_replaces_it(store: &impl PresetRepository) {
    assert_eq!(loaded_presets(store), [custom("a", "Renamed")]);
}

fn deleted<R: PresetRepository>(mut store: R) -> R {
    store.save(&custom("a", "A")).expect("save a");
    store.save(&custom("b", "B")).expect("save b");
    store.delete(&PresetId::new("a")).expect("delete a");
    store.delete(&PresetId::new("a")).expect("delete a twice");
    store
}

fn deleted_presets_are_gone(store: &impl PresetRepository) {
    assert_eq!(loaded_presets(store), [custom("b", "B")]);
}

fn row(at: u64, percent: u8) -> Sample {
    Sample {
        at: UnixSeconds(at),
        reading: Reading {
            percent: Percent::try_from(percent).expect("percent"),
            charging: Charging::Discharging,
            link: Link::Connected,
        },
    }
}

fn appended<R: BatteryRepository>(mut store: R) -> R {
    store.append(&row(1, 90)).expect("append");
    store.append(&row(2, 89)).expect("append");
    store
}

fn appended_samples_load_back_in_order(store: &impl BatteryRepository) {
    assert_eq!(store.load().expect("load"), [row(1, 90), row(2, 89)]);
}

fn retained<R: BatteryRepository>(mut store: R) -> R {
    for (at, percent) in [(1, 90), (2, 89), (3, 88)] {
        store.append(&row(at, percent)).expect("append");
    }
    store.retain_since(UnixSeconds(2)).expect("retain");
    store
}

fn retention_drops_only_older_samples(store: &impl BatteryRepository) {
    assert_eq!(store.load().expect("load"), [row(2, 89), row(3, 88)]);
}

fn app(key: &str) -> SeenApp {
    SeenApp {
        key: key.to_owned(),
        name: key.to_uppercase(),
        identifiers: vec![key.to_owned()],
    }
}

fn recorded<R: SeenAppsRepository>(mut store: R) -> R {
    for key in ["a", "b", "a"] {
        store.record(&app(key)).expect("record");
    }
    store
}

fn recorded_apps_load_newest_first(store: &impl SeenAppsRepository) {
    assert_eq!(store.load().expect("load"), [app("a"), app("b")]);
}

fn recorded_many<R: SeenAppsRepository>(mut store: R) -> R {
    for n in 0..=MAX_SEEN_APPS {
        store.record(&app(&format!("app{n}"))).expect("record");
    }
    store
}

fn at_most_fifty_apps_are_kept(store: &impl SeenAppsRepository) {
    let apps = store.load().expect("load");
    assert_eq!(apps.len(), MAX_SEEN_APPS);
    assert_eq!(apps.first(), Some(&app(&format!("app{MAX_SEEN_APPS}"))));
}
