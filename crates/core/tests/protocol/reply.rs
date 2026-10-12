use arctis_core::protocol::command::Query;
use arctis_core::protocol::firmware::Firmware;
use arctis_core::protocol::frame::DecodeError;
use arctis_core::protocol::reply::{Reply, UxReadback, decode_reply};
use arctis_core::protocol::status::{Charging, Link, Status};
use arctis_core::settings::values::{MuteLight, WirelessMode};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

use crate::support::{DISABLED_BAND, auto_off, percent, version};

const DOCUMENTED_REPLIES: [(Query, &[u8]); 4] = [
    (Query::Status, &[0xB0, 0x03, 0x03, 0x25, 0x03]),
    (Query::Firmware, b"\x100.01.010.000............000000001.07"),
    (Query::Ux, &[0xA0, 0x1e, 0x0a]),
    (Query::Wireless, &[0x36, 0x01]),
];

fn frame(id: u8, fields: &[(usize, u8)]) -> Vec<u8> {
    let mut bytes = vec![0; 64];
    bytes[0] = id;
    for &(offset, value) in fields {
        bytes[offset] = value;
    }
    bytes
}

fn audio_frame(trailer: [u8; 3]) -> Vec<u8> {
    let mut bytes = vec![0x20];
    for _ in 0..10 {
        bytes.extend_from_slice(&DISABLED_BAND);
    }
    bytes.extend_from_slice(&trailer);
    bytes
}

fn firmware(receiver: &[u8; 12], headset: &[u8; 12]) -> Firmware {
    let mut bytes = vec![0x10];
    bytes.extend_from_slice(receiver);
    bytes.extend_from_slice(b"............");
    bytes.extend_from_slice(headset);
    bytes.resize(64, 0);
    let Ok(Reply::Firmware(firmware)) = decode_reply(Query::Firmware, &bytes) else {
        panic!("firmware frame does not decode");
    };
    firmware
}

#[test]
fn a_reply_must_echo_its_query() {
    assert_eq!(
        decode_reply(Query::Ux, &frame(0xB0, &[])),
        Err(DecodeError::UnexpectedId {
            expected: 0xA0,
            got: 0xB0
        })
    );
}

#[test]
fn a_reply_shorter_than_its_documented_length_is_rejected() {
    let cases = [
        (Query::Status, 5),
        (Query::Firmware, 37),
        (Query::Audio, 64),
        (Query::Ux, 3),
        (Query::Wireless, 2),
    ];
    for (query, needed) in cases {
        let short = vec![query.id(); needed - 1];
        assert_eq!(
            decode_reply(query, &short),
            Err(DecodeError::TooShort {
                needed,
                got: needed - 1
            })
        );
    }
}

#[test]
fn a_reply_of_exactly_its_documented_length_decodes() {
    assert_eq!(
        decode_reply(Query::Wireless, &[0x36, 0x01]),
        Ok(Reply::Wireless(WirelessMode::Standard))
    );
    assert_eq!(
        decode_reply(Query::Ux, &[0xA0, 0x1e, 0x0a]),
        Ok(Reply::Ux(UxReadback {
            auto_power_off: auto_off(30),
            mute_light: MuteLight::High,
        }))
    );
}

#[test]
fn a_status_reply_reads_link_battery_and_charging() {
    let bytes = frame(0xB0, &[(1, 3), (3, 73), (4, 1)]);
    let expected = Status {
        link: Link::Connected,
        battery: percent(73),
        charging: Charging::Charging,
    };
    assert_eq!(
        decode_reply(Query::Status, &bytes),
        Ok(Reply::Status(expected))
    );
}

#[test]
fn out_of_range_fields_name_their_offset() {
    let cases = [
        (Query::Status, frame(0xB0, &[(1, 4)]), 1),
        (Query::Status, frame(0xB0, &[(1, 3), (3, 101)]), 3),
        (Query::Status, frame(0xB0, &[(1, 3), (3, 50), (4, 4)]), 4),
        (Query::Audio, audio_frame([16, 0, 1]), 61),
        (Query::Audio, audio_frame([15, 11, 1]), 62),
        (Query::Audio, audio_frame([15, 0, 2]), 63),
        (Query::Wireless, frame(0x36, &[(1, 2)]), 1),
    ];
    for (query, bytes, offset) in cases {
        assert_eq!(
            decode_reply(query, &bytes),
            Err(DecodeError::InvalidValue { offset }),
            "{query:?}"
        );
    }
}

#[test]
fn an_invalid_band_in_an_audio_reply_is_reported() {
    let mut bytes = audio_frame([15, 0, 1]);
    bytes[1] = 0x13;
    bytes[2] = 0x00;
    assert_eq!(
        decode_reply(Query::Audio, &bytes),
        Err(DecodeError::InvalidValue { offset: 1 })
    );
}

#[test]
fn dotted_firmware_slots_decode_digit_pairs_in_base_16() {
    let decoded = firmware(b"0.01.010.000", b"000000000000");
    assert_eq!(decoded.receiver, Some(version(1, 16, 0)));
}

#[test]
fn packed_firmware_slots_decode_two_digit_pairs() {
    let decoded = firmware(b"0.01.010.000", b"000000001.07");
    assert_eq!(decoded.headset, Some(version(1, 7, 0)));
}

#[test]
fn an_unreachable_headset_reports_no_firmware() {
    assert_eq!(firmware(b"0.01.010.000", b"000000000000").headset, None);
}

#[test]
fn slots_with_non_digits_report_no_firmware() {
    let decoded = firmware(b"0.0:.010.000", b"0000000x1.07");
    assert_eq!(
        decoded,
        Firmware {
            receiver: None,
            headset: None
        }
    );
}

#[test]
fn versions_print_as_dotted_numbers() {
    assert_eq!(version(1, 16, 0).to_string(), "1.16.0");
}

#[test]
fn an_unlisted_auto_power_off_reads_as_the_nearest_choice() {
    let Ok(Reply::Ux(readback)) = decode_reply(Query::Ux, &[0xA0, 20, 0x0a]) else {
        panic!("the UX reply does not decode");
    };
    assert_eq!(readback.auto_power_off.minutes(), 15);
}

proptest! {
    #[test]
    fn decode_reply_never_panics(query in select(Query::ALL.to_vec()), mut bytes in vec(any::<u8>(), 0..=64)) {
        if let Some(first) = bytes.first_mut() {
            *first = query.id();
        }
        let _ = decode_reply(query, &bytes);
    }

    #[test]
    fn stale_tails_never_change_a_reply(index in 0..DOCUMENTED_REPLIES.len(), tail in vec(any::<u8>(), 64)) {
        let (query, prefix) = DOCUMENTED_REPLIES[index];
        let mut bytes = prefix.to_vec();
        bytes.extend_from_slice(&tail[prefix.len()..]);
        prop_assert!(decode_reply(query, prefix).is_ok());
        prop_assert_eq!(decode_reply(query, &bytes), decode_reply(query, prefix));
    }
}
