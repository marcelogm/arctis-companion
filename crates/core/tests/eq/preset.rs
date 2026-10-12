use arctis_core::eq::{Curve, FilterKind, Preset, PresetId, PresetKind, library};

use crate::support::{filter, peaking};

#[test]
fn a_custom_preset_keeps_what_it_was_given() {
    let curve = Curve::from_array([peaking(1_000, 4, 1_000)]);
    let preset = Preset::custom(PresetId::new("mine"), "Mine".to_owned(), curve.clone());
    assert_eq!(preset.id().as_str(), "mine");
    assert_eq!(preset.name(), "Mine");
    assert_eq!(preset.curve(), &curve);
    assert_eq!(preset.kind(), PresetKind::Custom);
}

#[test]
fn a_built_in_preset_is_marked_built_in() {
    let preset = Preset::built_in("flat", "Flat", Curve::default());
    assert_eq!(preset.id(), &PresetId::new("flat"));
    assert_eq!(preset.kind(), PresetKind::BuiltIn);
}

const SPEC_EXAMPLE: &str = r#"{"format": 1, "id": "bass-boost-mine", "name": "Bass Boost (Mine)",
 "filters": [{"kind": "low-shelf", "hz": 105, "gain_db": 5.5, "q": 0.71}]}"#;

fn document(format: u32, filters: &str) -> String {
    format!(r#"{{"format": {format}, "id": "x", "name": "X", "filters": [{filters}]}}"#)
}

#[test]
fn the_spec_example_reads_as_a_custom_preset() {
    let preset: Preset = serde_json::from_str(SPEC_EXAMPLE).expect("valid preset");
    assert_eq!(preset.id().as_str(), "bass-boost-mine");
    assert_eq!(preset.name(), "Bass Boost (Mine)");
    let expected = Curve::from_array([filter(FilterKind::LowShelf, 105, 11, 710)]);
    assert_eq!(preset.curve(), &expected);
    assert_eq!(preset.kind(), PresetKind::Custom);
}

#[test]
fn a_preset_writes_the_spec_shape() {
    let preset: Preset = serde_json::from_str(SPEC_EXAMPLE).expect("valid preset");
    let written = serde_json::to_value(&preset).expect("serializable");
    let expected: serde_json::Value = serde_json::from_str(SPEC_EXAMPLE).expect("valid json");
    assert_eq!(written, expected);
}

#[test]
fn only_disabled_filters_write_enabled() {
    let curve = Curve::from_array([
        peaking(100, 2, 1_000),
        peaking(200, 2, 1_000).with_enabled(false),
    ]);
    let preset = Preset::custom(PresetId::new("p"), "P".to_owned(), curve.clone());
    let written = serde_json::to_value(&preset).expect("serializable");
    assert_eq!(written["filters"][0].get("enabled"), None);
    assert_eq!(
        written["filters"][1]["enabled"],
        serde_json::Value::Bool(false)
    );
    let read: Preset = serde_json::from_value(written).expect("readable");
    assert_eq!(read.curve(), &curve);
}

#[test]
fn every_built_in_survives_json() {
    for preset in library::library() {
        let text = serde_json::to_string(&preset).expect("serializable");
        let read: Preset = serde_json::from_str(&text).expect("readable");
        assert_eq!(
            (read.id(), read.name(), read.curve()),
            (preset.id(), preset.name(), preset.curve())
        );
        assert_eq!(read.kind(), PresetKind::Custom);
    }
}

#[test]
fn files_outside_the_format_or_the_envelope_are_refused() {
    let low_shelf = r#"{"kind": "low-shelf", "hz": 105, "gain_db": 5.5, "q": 0.71}"#;
    let cases = [
        document(2, low_shelf),
        document(
            1,
            r#"{"kind": "peaking", "hz": 100, "gain_db": 5.3, "q": 1}"#,
        ),
        document(
            1,
            r#"{"kind": "peaking", "hz": 100, "gain_db": 10.5, "q": 1}"#,
        ),
        document(
            1,
            r#"{"kind": "low-pass", "hz": 100, "gain_db": 3, "q": 1}"#,
        ),
        document(1, r#"{"kind": "peaking", "hz": 19, "gain_db": 0, "q": 1}"#),
        document(
            1,
            r#"{"kind": "peaking", "hz": 100, "gain_db": 0, "q": 0.1}"#,
        ),
        document(
            1,
            r#"{"kind": "band-pass", "hz": 100, "gain_db": 0, "q": 1}"#,
        ),
        document(1, &[low_shelf; 11].join(",")),
    ];
    for text in cases {
        assert!(serde_json::from_str::<Preset>(&text).is_err(), "{text}");
    }
}

#[test]
fn an_unknown_format_names_itself() {
    let error = serde_json::from_str::<Preset>(&document(2, "")).expect_err("format 2");
    assert!(
        error.to_string().contains("unsupported preset format 2"),
        "{error}"
    );
}
