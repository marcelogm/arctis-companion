mod support;

use std::fs;

use support::{golden, golden_dir};

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
