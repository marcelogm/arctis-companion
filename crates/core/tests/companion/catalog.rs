use arctis_core::companion::catalog::{Catalog, fresh_id};
use arctis_core::eq::library;

use crate::support::{curve_of, custom, id, unique_curve};

#[test]
fn the_catalog_lists_built_ins_then_custom_presets() {
    let catalog = Catalog::new(vec![custom("mine", unique_curve())]);
    let ids: Vec<String> = catalog
        .all()
        .iter()
        .map(|p| p.id().as_str().to_owned())
        .collect();
    assert_eq!(ids.len(), 11);
    assert_eq!(ids.first().map(String::as_str), Some("flat"));
    assert_eq!(ids.last().map(String::as_str), Some("mine"));
}

#[test]
fn an_unknown_id_resolves_to_flat() {
    let catalog = Catalog::default();
    assert!(!catalog.contains(&id("gone")));
    assert_eq!(catalog.resolve(&id("gone")), library::flat());
    assert_eq!(catalog.resolve_id(&id("gone")), id("flat"));
}

#[test]
fn a_known_id_resolves_to_itself() {
    let catalog = Catalog::new(vec![custom("mine", unique_curve())]);
    assert_eq!(catalog.resolve(&id("mine")), custom("mine", unique_curve()));
    assert_eq!(catalog.resolve_id(&id("bass-boost")), id("bass-boost"));
}

#[test]
fn matching_prefers_built_ins_then_custom_presets() {
    let catalog = Catalog::new(vec![
        custom("copy", curve_of("bass-boost")),
        custom("mine", unique_curve()),
    ]);
    assert_eq!(
        catalog.matching(&curve_of("bass-boost")),
        Some(id("bass-boost"))
    );
    assert_eq!(catalog.matching(&unique_curve()), Some(id("mine")));
    assert_eq!(Catalog::default().matching(&unique_curve()), None);
}

#[test]
fn saving_replaces_by_id_and_refuses_built_in_ids() {
    let mut catalog = Catalog::default();
    assert!(catalog.save(custom("mine", unique_curve())));
    assert!(catalog.save(custom("mine", curve_of("podcast"))));
    assert_eq!(
        catalog.find(&id("mine")),
        Some(&custom("mine", curve_of("podcast")))
    );
    assert_eq!(catalog.all().len(), library::library().len() + 1);
    assert!(!catalog.save(custom("flat", unique_curve())));
    assert!(!catalog.save(library::flat()));
}

#[test]
fn deleting_removes_only_custom_presets() {
    let mut catalog = Catalog::new(vec![custom("mine", unique_curve())]);
    assert!(!catalog.delete(&id("flat")));
    assert!(catalog.delete(&id("mine")));
    assert!(!catalog.delete(&id("mine")));
}

#[test]
fn fresh_ids_are_slugs_with_a_free_suffix() {
    let taken = Catalog::new(vec![custom("headset-curve", unique_curve())]);
    let cases = [
        (Catalog::default(), "Headset Curve", "headset-curve"),
        (taken, "Headset Curve", "headset-curve-2"),
        (Catalog::default(), "Bass Boost", "bass-boost-2"),
        (Catalog::default(), "!!!", "preset"),
        (Catalog::default(), "Late  Night (v2)", "late-night-v2"),
    ];
    for (catalog, name, expected) in cases {
        assert_eq!(catalog.fresh_id(name), id(expected), "{name}");
    }
}

#[test]
fn a_fresh_id_skips_every_taken_suffix() {
    let taken = [
        custom("mine", unique_curve()),
        custom("mine-2", unique_curve()),
    ];
    assert_eq!(fresh_id(&taken, "Mine"), id("mine-3"));
}
