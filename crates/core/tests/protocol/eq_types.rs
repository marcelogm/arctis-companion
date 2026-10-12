use arctis_core::eq::curve::MAX_FILTERS;
use arctis_core::eq::{Curve, EnvelopeError, Filter, FilterKind, Gain, Hertz, Q};

fn peaking(hz: u16) -> Filter {
    Filter::from_parts(FilterKind::Peaking, hz, 0, 1410).expect("filter in envelope")
}

#[test]
fn frequency_spans_20_hz_to_20_khz() {
    assert_eq!(Hertz::new(20).map(Hertz::get), Ok(20));
    assert_eq!(Hertz::new(20_000).map(Hertz::get), Ok(20_000));
    assert_eq!(Hertz::new(19), Err(EnvelopeError::Frequency));
    assert_eq!(Hertz::new(20_001), Err(EnvelopeError::Frequency));
}

#[test]
fn gain_spans_minus_to_plus_ten_db_in_half_steps() {
    assert_eq!(Gain::from_half_db(-20).map(Gain::half_db), Ok(-20));
    assert_eq!(Gain::from_half_db(20).map(Gain::half_db), Ok(20));
    assert_eq!(Gain::from_half_db(-21), Err(EnvelopeError::Gain));
    assert_eq!(Gain::from_half_db(21), Err(EnvelopeError::Gain));
}

#[test]
fn q_in_thousandths_spans_200_to_10000() {
    assert_eq!(Q::from_milli(200).map(Q::milli), Ok(200));
    assert_eq!(Q::from_milli(10_000).map(Q::milli), Ok(10_000));
    assert_eq!(Q::from_milli(199), Err(EnvelopeError::Q));
    assert_eq!(Q::from_milli(10_001), Err(EnvelopeError::Q));
}

#[test]
fn only_pass_filters_have_no_gain() {
    let gainless: Vec<FilterKind> = FilterKind::ALL
        .into_iter()
        .filter(|kind| !kind.has_gain())
        .collect();
    assert_eq!(gainless, [FilterKind::LowPass, FilterKind::HighPass]);
}

#[test]
fn pass_filters_refuse_gain() {
    assert_eq!(
        Filter::from_parts(FilterKind::LowPass, 1000, 1, 707),
        Err(EnvelopeError::PassGain)
    );
    assert!(Filter::from_parts(FilterKind::HighPass, 1000, 0, 707).is_ok());
}

#[test]
fn from_parts_reports_the_first_field_outside_the_envelope() {
    let kind = FilterKind::Peaking;
    assert_eq!(
        Filter::from_parts(kind, 10, 40, 50),
        Err(EnvelopeError::Frequency)
    );
    assert_eq!(
        Filter::from_parts(kind, 100, 40, 50),
        Err(EnvelopeError::Gain)
    );
    assert_eq!(Filter::from_parts(kind, 100, 4, 50), Err(EnvelopeError::Q));
}

#[test]
fn a_filter_keeps_its_parts() {
    let filter = Filter::from_parts(FilterKind::LowShelf, 80, 7, 707).expect("in envelope");
    assert_eq!(filter.kind(), FilterKind::LowShelf);
    assert_eq!(filter.hz().get(), 80);
    assert_eq!(filter.gain().half_db(), 7);
    assert_eq!(filter.q().milli(), 707);
}

#[test]
fn filters_start_enabled_and_can_be_disabled() {
    assert!(peaking(1000).is_enabled());
    assert!(!peaking(1000).with_enabled(false).is_enabled());
}

#[test]
fn a_curve_keeps_up_to_ten_filters_in_order() {
    let ten: Vec<Filter> = (1..=10).map(|step| peaking(step * 100)).collect();
    let curve = Curve::new(ten.clone()).expect("ten filters fit");
    assert_eq!(curve.filters(), ten.as_slice());
}

#[test]
fn a_curve_rejects_an_eleventh_filter() {
    let eleven: Vec<Filter> = (1..=11).map(|step| peaking(step * 100)).collect();
    assert_eq!(eleven.len(), MAX_FILTERS + 1);
    assert_eq!(Curve::new(eleven), Err(EnvelopeError::TooManyFilters));
}

#[test]
fn the_default_curve_has_no_filters() {
    assert_eq!(Curve::default().filters(), []);
    assert_eq!(Curve::from_array([peaking(500)]).filters(), [peaking(500)]);
}
