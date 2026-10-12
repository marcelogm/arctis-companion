mod strategies;

use arctis_core::eq::curve::MAX_FILTERS;
use arctis_core::eq::{Curve, Filter, FilterKind};
use arctis_core::protocol::eq_wire::{decode_curve, encode_curve};
use arctis_core::protocol::frame::DecodeError;
use proptest::collection::vec;
use proptest::prelude::*;

const DISABLED_BAND: [u8; 6] = [0x21, 0x4e, 0x01, 0x14, 0xe8, 0x03];

fn filter(kind: FilterKind, hz: u16, half_db: i8, milli_q: u16) -> Filter {
    Filter::from_parts(kind, hz, half_db, milli_q).expect("filter in envelope")
}

fn frame_with_bands(bands: &[[u8; 6]]) -> Vec<u8> {
    let mut frame = vec![0x20];
    for slot in 0..MAX_FILTERS {
        frame.extend_from_slice(bands.get(slot).unwrap_or(&DISABLED_BAND));
    }
    frame
}

#[test]
fn a_filter_encodes_as_frequency_kind_gain_and_q() {
    let curve = Curve::from_array([filter(FilterKind::LowShelf, 80, 7, 707)]);
    assert_eq!(
        encode_curve(&curve)[..6],
        [0x50, 0x00, 0x04, 0x1b, 0xc3, 0x02]
    );
}

#[test]
fn each_kind_has_its_wire_code() {
    let codes: [(FilterKind, u8); 5] = [
        (FilterKind::Peaking, 1),
        (FilterKind::LowPass, 2),
        (FilterKind::HighPass, 3),
        (FilterKind::LowShelf, 4),
        (FilterKind::HighShelf, 5),
    ];
    for (kind, code) in codes {
        let curve = Curve::from_array([filter(kind, 1000, 0, 1000)]);
        assert_eq!(encode_curve(&curve)[2], code, "{kind:?}");
    }
}

#[test]
fn gain_bytes_span_0_to_40_around_20_for_flat() {
    let gains: Vec<u8> = [-20, 0, 20]
        .into_iter()
        .map(|half_db| {
            encode_curve(&Curve::from_array([filter(
                FilterKind::Peaking,
                1000,
                half_db,
                1000,
            )]))[3]
        })
        .collect();
    assert_eq!(gains, [0, 20, 40]);
}

#[test]
fn missing_filters_encode_as_disabled_bands() {
    let bytes = encode_curve(&Curve::default());
    assert!(
        bytes
            .as_chunks::<6>()
            .0
            .iter()
            .all(|band| *band == DISABLED_BAND)
    );
}

#[test]
fn disabled_filters_are_left_out_and_the_rest_close_up() {
    let kept = filter(FilterKind::Peaking, 1000, -5, 1414);
    let disabled = filter(FilterKind::LowShelf, 80, 7, 707).with_enabled(false);
    let bytes = encode_curve(&Curve::from_array([disabled, kept]));
    assert_eq!(bytes[..6], [0xe8, 0x03, 0x01, 0x0f, 0x86, 0x05]);
    assert_eq!(bytes[6..12], DISABLED_BAND);
}

#[test]
fn disabled_bands_decode_to_no_filter_whatever_their_other_bytes() {
    let frame = frame_with_bands(&[[0x21, 0x4e, 0x00, 0x00, 0x00, 0x00]]);
    assert_eq!(
        decode_curve(&frame).map(|curve| curve.filters().len()),
        Ok(0)
    );
}

#[test]
fn decoding_keeps_band_order_and_skips_disabled_slots() {
    let frame = frame_with_bands(&[
        [0x50, 0x00, 0x04, 0x1b, 0xc3, 0x02],
        DISABLED_BAND,
        [0xe8, 0x03, 0x01, 0x0f, 0x86, 0x05],
    ]);
    let expected = [
        filter(FilterKind::LowShelf, 80, 7, 707),
        filter(FilterKind::Peaking, 1000, -5, 1414),
    ];
    assert_eq!(
        decode_curve(&frame).expect("valid bands").filters(),
        expected
    );
}

#[test]
fn a_pass_band_with_a_gain_byte_decodes_with_no_gain() {
    let frame = frame_with_bands(&[[0x1e, 0x00, 0x03, 0x1b, 0xc3, 0x02]]);
    let expected = [filter(FilterKind::HighPass, 30, 0, 707)];
    assert_eq!(
        decode_curve(&frame).expect("valid band").filters(),
        expected
    );
}

#[test]
fn out_of_envelope_bands_name_the_offending_offset() {
    let cases = [
        ([0x13, 0x00, 0x01, 0x14, 0x82, 0x05], 1),
        ([0x20, 0x00, 0x00, 0x14, 0x82, 0x05], 3),
        ([0x20, 0x00, 0x06, 0x14, 0x82, 0x05], 3),
        ([0x20, 0x00, 0x01, 0x29, 0x82, 0x05], 4),
        ([0x20, 0x00, 0x01, 0xff, 0x82, 0x05], 4),
        ([0x20, 0x00, 0x03, 0x29, 0x82, 0x05], 4),
        ([0x20, 0x00, 0x01, 0x14, 0xc7, 0x00], 5),
    ];
    for (band, offset) in cases {
        assert_eq!(
            decode_curve(&frame_with_bands(&[band])),
            Err(DecodeError::InvalidValue { offset }),
            "{band:02x?}"
        );
    }
}

#[test]
fn an_invalid_band_names_its_offset_within_the_frame() {
    let invalid_gain = [0x20, 0x00, 0x01, 0x29, 0x82, 0x05];
    let frame = frame_with_bands(&[DISABLED_BAND, DISABLED_BAND, DISABLED_BAND, invalid_gain]);
    assert_eq!(
        decode_curve(&frame),
        Err(DecodeError::InvalidValue { offset: 22 })
    );
}

#[test]
fn a_frame_without_ten_bands_is_too_short() {
    assert_eq!(
        decode_curve(&[0x20; 60]),
        Err(DecodeError::TooShort {
            needed: 61,
            got: 60
        })
    );
}

proptest! {
    #[test]
    fn every_curve_round_trips_through_the_wire(curve in strategies::curve()) {
        let mut frame = vec![0x33];
        frame.extend_from_slice(&encode_curve(&curve));
        prop_assert_eq!(decode_curve(&frame), Ok(curve));
    }

    #[test]
    fn decode_curve_never_panics(bytes in vec(any::<u8>(), 0..=64)) {
        let _ = decode_curve(&bytes);
    }
}
