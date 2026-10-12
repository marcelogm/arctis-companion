use super::filter::{EnvelopeError, Filter, Q};

pub const MAX_FILTERS: usize = 10;
pub const FREQUENCY_TOLERANCE_HZ: u16 = 1;
pub const Q_TOLERANCE_PERCENT: u32 = 1;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Curve {
    filters: Vec<Filter>,
}

impl Curve {
    pub fn new(filters: Vec<Filter>) -> Result<Self, EnvelopeError> {
        if filters.len() > MAX_FILTERS {
            return Err(EnvelopeError::TooManyFilters);
        }
        Ok(Self { filters })
    }

    #[must_use]
    pub fn from_array<const N: usize>(filters: [Filter; N]) -> Self {
        const { assert!(N <= MAX_FILTERS) };
        Self {
            filters: filters.to_vec(),
        }
    }

    #[must_use]
    pub fn filters(&self) -> &[Filter] {
        &self.filters
    }
}

#[must_use]
pub fn equivalent(a: &Curve, b: &Curve) -> bool {
    enabled(a).count() == enabled(b).count()
        && enabled(a)
            .zip(enabled(b))
            .all(|(left, right)| filters_equivalent(left, right))
}

fn enabled(curve: &Curve) -> impl Iterator<Item = Filter> {
    curve
        .filters
        .iter()
        .copied()
        .filter(|filter| filter.is_enabled())
}

fn filters_equivalent(a: Filter, b: Filter) -> bool {
    a.kind() == b.kind()
        && a.gain() == b.gain()
        && a.hz().get().abs_diff(b.hz().get()) <= FREQUENCY_TOLERANCE_HZ
        && q_within_tolerance(a.q(), b.q())
}

fn q_within_tolerance(a: Q, b: Q) -> bool {
    let difference = u32::from(a.milli().abs_diff(b.milli()));
    let larger = u32::from(a.milli().max(b.milli()));
    difference * 100 <= larger * Q_TOLERANCE_PERCENT
}
