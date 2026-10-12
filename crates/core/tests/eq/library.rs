use arctis_core::eq::curve::{self, MAX_FILTERS};
use arctis_core::eq::frequency_response::FrequencyResponse;
use arctis_core::eq::library::{self, FLAT_ID, HARMAN_ID};
use arctis_core::eq::reference::{Target, headset_response, headset_response_with};
use arctis_core::eq::{EnvelopeError, Filter, Preset, PresetId, PresetKind, apo};

use crate::support::factory_curve;

const IDS: [&str; 11] = [
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
    "harman",
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
fn the_library_ships_eleven_built_in_presets() {
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

const AUTOEQ_NOVA_5: &str = include_str!("../../data/nova5_autoeq_parametric_eq.txt");
const RESIDUAL_UP_TO_HZ: f64 = 10_000.0;
const STOCK_ERROR_AT_LEAST_DB: f64 = 4.0;
const HARMAN_ERROR_AT_MOST_DB: f64 = 1.6;

fn harman() -> Preset {
    library::find(&PresetId::new(HARMAN_ID)).expect("built-in")
}

fn rms_error_db(response: &FrequencyResponse, target: &FrequencyResponse) -> f64 {
    let errors: Vec<f64> = response
        .points()
        .iter()
        .filter(|point| point.hz <= RESIDUAL_UP_TO_HZ)
        .map(|point| point.db - target.db_at(point.hz))
        .collect();
    let squares: f64 = errors.iter().map(|error| error * error).sum();
    (squares / f64::from(u32::try_from(errors.len()).expect("few points"))).sqrt()
}

#[test]
fn harman_is_autoeqs_fit_rounded_to_the_device_grid() {
    let import = apo::parse(AUTOEQ_NOVA_5).expect("AutoEq text parses");
    assert!(import.clamped.is_empty(), "{:?}", import.clamped);
    let mut by_frequency = import.curve.filters().to_vec();
    by_frequency.sort_by_key(|filter| filter.hz());
    assert_eq!(harman().curve().filters(), by_frequency);
    assert_eq!(harman().name(), "Harman target");
}

#[test]
fn harman_brings_the_headset_close_to_the_harman_target() {
    let target = Target::Harman.response();
    let stock = rms_error_db(&headset_response(), &target);
    let equalized = rms_error_db(&headset_response_with(harman().curve()), &target);
    assert!(stock >= STOCK_ERROR_AT_LEAST_DB, "stock {stock} dB");
    assert!(
        equalized <= HARMAN_ERROR_AT_MOST_DB,
        "with Harman {equalized} dB"
    );
}
