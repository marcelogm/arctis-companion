mod strategies;

use arctis_core::eq::Curve;
use arctis_core::protocol::command::{Command, Query, encode};
use arctis_core::settings::headset_settings::SettingChange;
use arctis_core::settings::values::{AutoPowerOff, MicVolume, MuteLight, Sidetone, WirelessMode};
use proptest::prelude::*;
use proptest::sample::select;

const ALLOWED_IDS: [u8; 12] = [
    0xB0, 0x10, 0x20, 0xA0, 0x36, 0x37, 0x39, 0x27, 0x33, 0xAE, 0xA3, 0x35,
];
const FORBIDDEN_IDS: [u8; 5] = [0x09, 0xA5, 0xA7, 0x22, 0x34];
const SET_WIRELESS_MODE: u8 = 0x35;

fn mic(level: u8) -> MicVolume {
    MicVolume::try_from(level).expect("mic volume")
}

fn sidetone(level: u8) -> Sidetone {
    Sidetone::try_from(level).expect("sidetone")
}

fn empty_curve() -> Curve {
    Curve::default()
}

#[test]
fn queries_encode_to_their_bare_command_byte() {
    for query in Query::ALL {
        let frame = encode(&Command::from(query));
        assert_eq!(frame.payload()[0], query.id());
        assert!(
            frame.payload()[1..].iter().all(|&byte| byte == 0),
            "{query:?}"
        );
    }
    let ids: Vec<u8> = Query::ALL.into_iter().map(Query::id).collect();
    assert_eq!(ids, [0xB0, 0x10, 0x20, 0xA0, 0x36]);
}

#[test]
fn a_frame_is_65_bytes_led_by_a_zero_report_id() {
    let frame = encode(&Command::QueryStatus);
    assert_eq!(frame.as_bytes().len(), 65);
    assert_eq!(frame.as_bytes()[0], 0);
    assert_eq!(frame.payload(), &frame.as_bytes()[1..]);
}

#[test]
fn setters_encode_to_their_cited_bytes() {
    let cases = [
        (Command::SetMicVolume(mic(15)), [0x37, 0x0f]),
        (Command::SetSidetone(sidetone(6)), [0x39, 0x06]),
        (Command::SetVolumeLimiter(true), [0x27, 0x01]),
        (Command::SetMuteLight(MuteLight::High), [0xAE, 0x0A]),
        (Command::SetAutoPowerOff(AutoPowerOff::NEVER), [0xA3, 0x00]),
        (
            Command::SetWirelessMode(WirelessMode::ExtendedRange),
            [0x35, 0x00],
        ),
    ];
    for (command, bytes) in cases {
        assert_eq!(encode(&command).payload()[..2], bytes, "{command:?}");
    }
}

#[test]
fn set_equalizer_carries_ten_bands_after_its_id() {
    let frame = encode(&Command::SetEqualizer(empty_curve()));
    assert_eq!(frame.payload()[0], 0x33);
    assert_eq!(frame.payload()[1..7], [0x21, 0x4e, 0x01, 0x14, 0xe8, 0x03]);
    assert_eq!(frame.payload()[61..], [0, 0, 0]);
}

#[test]
fn every_setter_names_its_readback_query() {
    let cases = [
        (Command::SetMicVolume(mic(1)), Query::Audio),
        (Command::SetSidetone(sidetone(1)), Query::Audio),
        (Command::SetVolumeLimiter(false), Query::Audio),
        (Command::SetEqualizer(empty_curve()), Query::Audio),
        (Command::SetMuteLight(MuteLight::Off), Query::Ux),
        (Command::SetAutoPowerOff(AutoPowerOff::NEVER), Query::Ux),
        (
            Command::SetWirelessMode(WirelessMode::Standard),
            Query::Wireless,
        ),
    ];
    for (command, query) in cases {
        assert_eq!(command.readback(), Some(query), "{command:?}");
    }
}

#[test]
fn queries_have_no_readback() {
    for query in Query::ALL {
        assert_eq!(Command::from(query).readback(), None);
    }
}

#[test]
fn each_setting_change_becomes_its_setter() {
    let cases = [
        (
            SettingChange::MicVolume(mic(3)),
            Command::SetMicVolume(mic(3)),
        ),
        (
            SettingChange::Sidetone(sidetone(3)),
            Command::SetSidetone(sidetone(3)),
        ),
        (
            SettingChange::MuteLight(MuteLight::Low),
            Command::SetMuteLight(MuteLight::Low),
        ),
        (
            SettingChange::VolumeLimiter(true),
            Command::SetVolumeLimiter(true),
        ),
        (
            SettingChange::WirelessMode(WirelessMode::Standard),
            Command::SetWirelessMode(WirelessMode::Standard),
        ),
        (
            SettingChange::AutoPowerOff(AutoPowerOff::NEVER),
            Command::SetAutoPowerOff(AutoPowerOff::NEVER),
        ),
        (
            SettingChange::Equalizer(empty_curve()),
            Command::SetEqualizer(empty_curve()),
        ),
    ];
    for (change, command) in cases {
        assert_eq!(Command::from(change), command);
    }
}

fn setter() -> impl Strategy<Value = Command> {
    prop_oneof![
        (0..=MicVolume::MAX).prop_map(|level| Command::SetMicVolume(mic(level))),
        (0..=Sidetone::MAX).prop_map(|level| Command::SetSidetone(sidetone(level))),
        any::<bool>().prop_map(Command::SetVolumeLimiter),
        strategies::curve().prop_map(Command::SetEqualizer),
        select(MuteLight::ALL.to_vec()).prop_map(Command::SetMuteLight),
        select(AutoPowerOff::CHOICES.to_vec()).prop_map(Command::SetAutoPowerOff),
        select(WirelessMode::ALL.to_vec()).prop_map(Command::SetWirelessMode),
    ]
}

fn command() -> impl Strategy<Value = Command> {
    prop_oneof![
        select(Query::ALL.to_vec()).prop_map(Command::from),
        setter()
    ]
}

fn argument_len(command: &Command) -> usize {
    match command {
        Command::SetEqualizer(_) => 60,
        other if other.readback().is_none() => 0,
        _ => 1,
    }
}

proptest! {
    #[test]
    fn no_command_encodes_to_a_forbidden_frame(command in command()) {
        let frame = encode(&command);
        let id = frame.payload()[0];
        prop_assert!(ALLOWED_IDS.contains(&id));
        prop_assert!(!FORBIDDEN_IDS.contains(&id));
        prop_assert!(frame.payload()[1 + argument_len(&command)..].iter().all(|&byte| byte == 0));
        prop_assert_eq!(id == SET_WIRELESS_MODE, matches!(command, Command::SetWirelessMode(_)));
    }
}
