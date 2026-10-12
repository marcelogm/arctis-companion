use std::fs;

use arctis_core::eq::FilterKind;
use arctis_core::protocol::command::{Command, encode};
use arctis_core::protocol::eq_wire::decode_curve;
use arctis_core::settings::values::{MuteLight, WirelessMode};

use crate::support::{auto_off, factory_flat, filter, golden, golden_dir, mic, sidetone};

const GOLDEN_NAMES: [&str; 39] = [
    "cmd_27_00.hex",
    "cmd_33_edges.hex",
    "cmd_33_kinds.hex",
    "cmd_33_odd.hex",
    "cmd_35_00.hex",
    "cmd_35_01.hex",
    "cmd_37_0e.hex",
    "cmd_39_03.hex",
    "cmd_39_06.hex",
    "cmd_39_0a.hex",
    "cmd_a3_0f.hex",
    "cmd_ae_00.hex",
    "cmd_ae_01.hex",
    "cmd_ae_04.hex",
    "reply_10_headset_off.hex",
    "reply_10_headset_on.hex",
    "reply_20_after_33_edges.hex",
    "reply_20_after_33_kinds.hex",
    "reply_20_after_33_odd.hex",
    "reply_20_after_limiter_off.hex",
    "reply_20_after_mic_volume_14.hex",
    "reply_20_after_sidetone_10.hex",
    "reply_20_after_sidetone_3.hex",
    "reply_20_after_sidetone_6.hex",
    "reply_20_flat_headset_off.hex",
    "reply_20_flat_headset_on.hex",
    "reply_22_flat_headset_off.hex",
    "reply_36_range_headset_off.hex",
    "reply_36_speed_headset_off.hex",
    "reply_36_speed_headset_on.hex",
    "reply_36_tail_from_fwupd_b0.hex",
    "reply_a0_after_auto_off_15.hex",
    "reply_a0_after_mute_light_0.hex",
    "reply_a0_after_mute_light_1.hex",
    "reply_a0_after_mute_light_4.hex",
    "reply_a0_headset_off.hex",
    "reply_a0_headset_on.hex",
    "reply_b0_link2_headset_off.hex",
    "reply_b0_link3_headset_on.hex",
];

fn golden_names() -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(golden_dir())
        .expect("golden directory exists")
        .map(|entry| {
            entry
                .expect("golden entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

#[test]
fn the_golden_set_is_the_sp0_capture_set() {
    assert_eq!(golden_names(), GOLDEN_NAMES);
}

#[test]
fn every_golden_file_is_one_64_byte_frame() {
    for name in GOLDEN_NAMES {
        assert_eq!(golden(name).len(), 64, "{name}");
    }
}

#[test]
fn the_kinds_curve_decodes_every_filter_kind() {
    let curve = decode_curve(&golden("cmd_33_kinds.hex")).expect("kinds curve decodes");
    let expected = [
        filter(FilterKind::LowShelf, 80, 7, 707),
        filter(FilterKind::HighPass, 30, 0, 707),
        filter(FilterKind::Peaking, 1000, -5, 1414),
        filter(FilterKind::LowPass, 18000, 0, 707),
        filter(FilterKind::HighShelf, 8000, 2, 707),
    ];
    assert_eq!(curve.filters(), expected);
}

#[test]
fn the_edges_curve_keeps_its_extremes_and_drops_its_disabled_band() {
    let curve = decode_curve(&golden("cmd_33_edges.hex")).expect("edges curve decodes");
    assert_eq!(curve.filters().len(), 9);
    assert_eq!(
        curve.filters()[0],
        filter(FilterKind::Peaking, 20, -20, 200)
    );
    assert_eq!(
        curve.filters()[1],
        filter(FilterKind::Peaking, 20_000, 20, 10_000)
    );
}

#[test]
fn the_bluetooth_band_layout_is_the_factory_flat_curve() {
    assert_eq!(
        decode_curve(&golden("reply_22_flat_headset_off.hex")),
        Ok(factory_flat())
    );
}

fn captured_setters() -> [(&'static str, Command); 11] {
    [
        ("cmd_27_00.hex", Command::SetVolumeLimiter(false)),
        (
            "cmd_35_00.hex",
            Command::SetWirelessMode(WirelessMode::ExtendedRange),
        ),
        (
            "cmd_35_01.hex",
            Command::SetWirelessMode(WirelessMode::Standard),
        ),
        ("cmd_37_0e.hex", Command::SetMicVolume(mic(14))),
        ("cmd_39_03.hex", Command::SetSidetone(sidetone(3))),
        ("cmd_39_06.hex", Command::SetSidetone(sidetone(6))),
        ("cmd_39_0a.hex", Command::SetSidetone(sidetone(10))),
        ("cmd_a3_0f.hex", Command::SetAutoPowerOff(auto_off(15))),
        ("cmd_ae_00.hex", Command::SetMuteLight(MuteLight::Off)),
        ("cmd_ae_01.hex", Command::SetMuteLight(MuteLight::Low)),
        ("cmd_ae_04.hex", Command::SetMuteLight(MuteLight::Medium)),
    ]
}

#[test]
fn every_captured_setter_encodes_to_its_golden_bytes() {
    for (name, command) in captured_setters() {
        assert_eq!(
            encode(&command).payload(),
            golden(name).as_slice(),
            "{name}"
        );
    }
}

#[test]
fn every_captured_curve_write_encodes_through_set_equalizer() {
    for name in ["cmd_33_kinds.hex", "cmd_33_odd.hex"] {
        let bytes = golden(name);
        let curve = decode_curve(&bytes).expect("captured curve decodes");
        assert_eq!(
            encode(&Command::SetEqualizer(curve)).payload(),
            bytes.as_slice(),
            "{name}"
        );
    }
}
