use arctis_core::eq::curve::MAX_FILTERS;
use arctis_core::eq::{Curve, Filter, FilterKind, Gain, Hertz, Q};
use proptest::prelude::*;
use proptest::sample::select;

fn filter() -> impl Strategy<Value = Filter> {
    (
        select(FilterKind::ALL.to_vec()),
        Hertz::MIN..=Hertz::MAX,
        Gain::MIN_HALF_DB..=Gain::MAX_HALF_DB,
        Q::MIN_MILLI..=Q::MAX_MILLI,
    )
        .prop_map(|(kind, hz, half_db, milli_q)| {
            let half_db = if kind.has_gain() { half_db } else { 0 };
            Filter::from_parts(kind, hz, half_db, milli_q).expect("filter in envelope")
        })
}

pub fn curve() -> impl Strategy<Value = Curve> {
    proptest::collection::vec(filter(), 0..=MAX_FILTERS)
        .prop_map(|filters| Curve::new(filters).expect("at most ten filters"))
}
