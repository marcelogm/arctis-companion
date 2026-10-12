use arctis_core::eq::FilterKind;
use arctis_core::eq::apo;
use arctis_core::eq::preset::PresetKind;
use arctis_core::testkit::autoeq::{SENNHEISER_HD_600, SENNHEISER_HD_600_FILE};
use arctis_core::testkit::english::english;
use arctis_core::view::preset_files::{
    export_failed, export_file_name, export_text, import_preset,
};

use crate::support::{custom, filter, id, snapshot};

const APO_LINE: &str = "Filter 1: ON PK Fc 100 Hz Gain 1 dB Q 1\n";

fn problem(file_name: &str, text: &str) -> String {
    english(&import_preset(&snapshot(), file_name, text).expect_err("rejected"))
}

fn imported_name(file_name: &str) -> String {
    let imported = import_preset(&snapshot(), file_name, APO_LINE).expect("imported");
    imported.preset.name().to_owned()
}

#[test]
fn a_published_autoeq_file_imports_as_a_custom_preset() {
    let imported =
        import_preset(&snapshot(), SENNHEISER_HD_600_FILE, SENNHEISER_HD_600).expect("imported");
    let preset = &imported.preset;
    assert_eq!(preset.name(), "Sennheiser HD 600 ParametricEQ");
    assert_eq!(preset.id(), &id("sennheiser-hd-600-parametriceq"));
    assert_eq!(preset.kind(), PresetKind::Custom);
    assert_eq!(
        english(&imported.toast),
        "Imported “Sennheiser HD 600 ParametricEQ”"
    );
    assert!(imported.clamped.is_empty());
}

#[test]
fn the_autoeq_filters_keep_their_types_and_snap_to_the_grid() {
    let imported =
        import_preset(&snapshot(), SENNHEISER_HD_600_FILE, SENNHEISER_HD_600).expect("imported");
    let filters = imported.preset.curve().filters();
    assert_eq!(filters.len(), 10);
    assert_eq!(filters[0], filter(FilterKind::LowShelf, 105, 13, 700));
    assert_eq!(filters[1], filter(FilterKind::Peaking, 125, -5, 550));
    assert_eq!(filters[5], filter(FilterKind::HighShelf, 10_000, -6, 700));
}

#[test]
fn clamped_values_are_counted_in_the_toast_and_listed() {
    let text =
        "Filter 1: ON PK Fc 10 Hz Gain 14 dB Q 0.7\nFilter 2: ON LPQ Fc 9000 Hz Gain 3 dB Q 0.7\n";
    let imported = import_preset(&snapshot(), "Loud.txt", text).expect("imported");
    assert_eq!(imported.preset.name(), "Loud");
    assert_eq!(
        english(&imported.toast),
        "Imported “Loud”. 3 values were changed to fit the headset"
    );
    let lines: Vec<String> = imported.clamped.iter().map(english).collect();
    assert_eq!(
        lines,
        ["Filter 1: Frequency", "Filter 1: Gain", "Filter 2: Gain"]
    );
}

#[test]
fn a_single_clamped_value_reads_in_the_singular() {
    let imported = import_preset(
        &snapshot(),
        "Hot.txt",
        "Filter 1: ON PK Fc 100 Hz Gain 14 dB Q 1\n",
    )
    .expect("imported");
    assert_eq!(
        english(&imported.toast),
        "Imported “Hot”. 1 value was changed to fit the headset"
    );
}

#[test]
fn problems_name_the_file() {
    assert_eq!(
        problem("eq.txt", "Preamp: -3 dB\n"),
        "“eq.txt” has no filters"
    );
    assert_eq!(
        problem("eq.txt", "Filter 1: ON PK Fc x Hz Gain 1 dB Q 1\n"),
        "Could not read line 1 of “eq.txt”"
    );
    assert_eq!(
        problem("eq.txt", "Preamp: 0 dB\nFilter 1: ON BP Fc 100 Hz Q 1\n"),
        "Line 2 of “eq.txt” uses a filter type the headset does not support"
    );
    let eleven = APO_LINE.repeat(11);
    assert_eq!(
        problem("eq.txt", &eleven),
        "“eq.txt” has 11 filters; the headset takes at most 10"
    );
    assert_eq!(
        problem("x.json", "{\"format\": 9}"),
        "“x.json” is not a preset file"
    );
}

#[test]
fn json_must_already_sit_on_the_device_grid() {
    let off_grid = r#"{"format": 1, "id": "x", "name": "X", "filters": [{"kind": "peaking", "hz": 250, "gain_db": 2.7, "q": 1.0}]}"#;
    assert_eq!(problem("x.json", off_grid), "“x.json” is not a preset file");
    let outside = r#"{"format": 1, "id": "x", "name": "X", "filters": [{"kind": "peaking", "hz": 250, "gain_db": 14.0, "q": 1.0}]}"#;
    assert_eq!(problem("x.json", outside), "“x.json” is not a preset file");
}

#[test]
fn a_json_preset_without_a_name_takes_the_file_name() {
    let text = r#"{"format": 1, "id": "x", "name": "  ", "filters": [{"kind": "peaking", "hz": 250, "gain_db": 2.0, "q": 1.0}]}"#;
    let imported = import_preset(&snapshot(), "Blank.json", text).expect("imported");
    assert_eq!(imported.preset.name(), "Blank");
    assert_eq!(imported.preset.id(), &id("blank"));
    assert_eq!(
        imported.preset.curve().filters(),
        [filter(FilterKind::Peaking, 250, 4, 1_000)]
    );
}

#[test]
fn files_are_named_without_their_last_extension() {
    assert_eq!(imported_name("eq"), "eq");
    assert_eq!(imported_name(".txt"), ".txt");
    assert_eq!(imported_name("my.eq.txt"), "my.eq");
}

#[test]
fn a_json_export_imports_back_under_a_fresh_id() {
    let mut s = snapshot();
    let mine = custom("Mine");
    s.presets.all.push(mine.clone());
    let text = export_text(&mine, "Mine.json").expect("exported");
    let imported = import_preset(&s, "Mine.json", &text).expect("imported");
    assert_eq!(imported.preset.id(), &id("mine-2"));
    assert_eq!(imported.preset.name(), "Mine");
    assert_eq!(imported.preset.curve(), mine.curve());
}

#[test]
fn a_text_file_name_exports_equalizer_apo() {
    let mine = custom("Mine");
    assert_eq!(
        export_text(&mine, "mine.TXT"),
        Some(apo::render(mine.curve()))
    );
}

#[test]
fn exports_suggest_a_json_name_and_report_failures() {
    assert_eq!(export_file_name("Mine"), "Mine.json");
    assert_eq!(
        english(&export_failed(&custom("Mine"))),
        "Could not export “Mine”"
    );
}
