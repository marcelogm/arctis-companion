use arctis_core::eq::{Filter, FilterKind};

pub fn filter(kind: FilterKind, hz: u16, half_db: i8, milli_q: u16) -> Filter {
    Filter::from_parts(kind, hz, half_db, milli_q).expect("filter inside the envelope")
}

pub fn peaking(hz: u16, half_db: i8, milli_q: u16) -> Filter {
    filter(FilterKind::Peaking, hz, half_db, milli_q)
}
