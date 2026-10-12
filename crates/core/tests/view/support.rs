use arctis_core::eq::library;
use arctis_core::eq::preset::Preset;
use arctis_core::eq::{Gain, Hertz};

pub use arctis_core::testkit::snapshots::{id, nova_5, percent, rules, snapshot, with};

pub fn gain(half_db: i8) -> Gain {
    Gain::from_half_db(half_db).expect("gain in the envelope")
}

pub fn hertz(hz: u16) -> Hertz {
    Hertz::new(hz).expect("frequency in the envelope")
}

pub fn custom(name: &str) -> Preset {
    Preset::custom(
        id(&name.to_lowercase()),
        name.to_owned(),
        library::flat().curve().clone(),
    )
}

pub fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "got {actual}, expected {expected} ± {tolerance}"
    );
}
