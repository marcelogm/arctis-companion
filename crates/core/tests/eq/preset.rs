use arctis_core::eq::{Curve, Preset, PresetId, PresetKind};

use crate::support::peaking;

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
