use arctis_core::eq::{Curve, Filter, FilterKind};

const FACTORY_BANDS: [u16; 10] = [32, 64, 125, 250, 500, 1_000, 2_000, 4_000, 8_000, 16_000];

pub fn filter(kind: FilterKind, hz: u16, half_db: i8, milli_q: u16) -> Filter {
    Filter::from_parts(kind, hz, half_db, milli_q).expect("filter inside the envelope")
}

pub fn peaking(hz: u16, half_db: i8, milli_q: u16) -> Filter {
    filter(FilterKind::Peaking, hz, half_db, milli_q)
}

pub fn factory_curve(milli_q: u16) -> Curve {
    let filters = FACTORY_BANDS.iter().map(|&hz| peaking(hz, 0, milli_q));
    Curve::new(filters.collect()).expect("ten filters")
}

pub fn assert_close(actual: f64, expected: f64, tolerance: f64, case: &str) {
    let error = (actual - expected).abs();
    assert!(
        error <= tolerance,
        "{case}: got {actual}, expected {expected} ± {tolerance}"
    );
}
