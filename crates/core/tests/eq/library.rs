use arctis_core::eq::curve::{self, MAX_FILTERS};
use arctis_core::eq::library::{self, FLAT_ID};
use arctis_core::eq::{EnvelopeError, Filter, PresetId, PresetKind};

use crate::support::factory_curve;

const IDS: [&str; 10] = [
    "flat",
    "bass-boost",
    "bass-reduction",
    "treble-soften",
    "vocal-clarity",
    "footsteps",
    "v-shape",
    "movie-night",
    "podcast",
    "late-night",
];

fn rebuilt(filter: Filter) -> Result<Filter, EnvelopeError> {
    let (hz, half_db, milli_q) = (
        filter.hz().get(),
        filter.gain().half_db(),
        filter.q().milli(),
    );
    Filter::from_parts(filter.kind(), hz, half_db, milli_q)
}

#[test]
fn the_library_ships_ten_built_in_presets() {
    let presets = library::library();
    let ids: Vec<&str> = presets.iter().map(|preset| preset.id().as_str()).collect();
    assert_eq!(ids, IDS);
    assert!(
        presets
            .iter()
            .all(|preset| preset.kind() == PresetKind::BuiltIn)
    );
}

#[test]
fn every_built_in_fits_the_envelope() {
    for preset in library::library() {
        let filters = preset.curve().filters();
        assert!(filters.len() <= MAX_FILTERS, "{}", preset.name());
        assert!(
            filters.iter().all(|&f| rebuilt(f) == Ok(f)),
            "{}",
            preset.name()
        );
    }
}

#[test]
fn flat_is_the_factory_curve() {
    let flat = library::flat();
    assert_eq!(flat.id().as_str(), FLAT_ID);
    assert!(curve::equivalent(flat.curve(), &factory_curve(1_410)));
    assert!(curve::equivalent(flat.curve(), &factory_curve(1_414)));
}

#[test]
fn presets_are_found_by_id() {
    let footsteps = library::find(&PresetId::new("footsteps"));
    assert_eq!(
        footsteps.map(|preset| preset.name().to_owned()),
        Some("Footsteps".to_owned())
    );
    assert_eq!(library::find(&PresetId::new("missing")), None);
}
