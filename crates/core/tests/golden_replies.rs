mod support;

use arctis_core::protocol::command::Query;
use arctis_core::protocol::eq_wire::decode_curve;
use arctis_core::protocol::firmware::{Firmware, Version};
use arctis_core::protocol::reply::{
    AudioReadback, Reply, UxReadback, decode_reply, observed_settings,
};
use arctis_core::protocol::status::{Charging, Link, Status};
use arctis_core::settings::headset_settings::HeadsetSettings;
use arctis_core::settings::values::{MuteLight, Percent, WirelessMode};
use support::{auto_off, factory_flat, golden, mic, sidetone};

fn decoded(query: Query, name: &str) -> Reply {
    decode_reply(query, &golden(name)).expect("golden reply decodes")
}

fn audio(name: &str) -> AudioReadback {
    let Reply::Audio(readback) = decoded(Query::Audio, name) else {
        panic!("{name} is not an audio reply");
    };
    readback
}

fn ux(name: &str) -> UxReadback {
    let Reply::Ux(readback) = decoded(Query::Ux, name) else {
        panic!("{name} is not a UX reply");
    };
    readback
}

fn wireless(name: &str) -> WirelessMode {
    let Reply::Wireless(mode) = decoded(Query::Wireless, name) else {
        panic!("{name} is not a wireless reply");
    };
    mode
}

fn percent(value: u8) -> Percent {
    Percent::try_from(value).expect("percent")
}

const fn version(major: u8, minor: u8, patch: u8) -> Version {
    Version {
        major,
        minor,
        patch,
    }
}

#[test]
fn status_replies_decode_link_battery_and_charging() {
    let on = Status {
        link: Link::Connected,
        battery: percent(37),
        charging: Charging::Discharging,
    };
    let off = Status {
        link: Link::HeadsetOff,
        battery: percent(32),
        charging: Charging::Discharging,
    };
    assert_eq!(
        decoded(Query::Status, "reply_b0_link3_headset_on.hex"),
        Reply::Status(on)
    );
    assert_eq!(
        decoded(Query::Status, "reply_b0_link2_headset_off.hex"),
        Reply::Status(off)
    );
}

#[test]
fn firmware_replies_decode_receiver_and_headset_versions() {
    let on = Firmware {
        receiver: Some(version(1, 16, 0)),
        headset: Some(version(1, 7, 0)),
    };
    let off = Firmware {
        receiver: Some(version(1, 16, 0)),
        headset: None,
    };
    assert_eq!(
        decoded(Query::Firmware, "reply_10_headset_on.hex"),
        Reply::Firmware(on)
    );
    assert_eq!(
        decoded(Query::Firmware, "reply_10_headset_off.hex"),
        Reply::Firmware(off)
    );
}

#[test]
fn flat_audio_replies_decode_the_factory_curve_and_trailer() {
    let expected = AudioReadback {
        equalizer: factory_flat(),
        mic_volume: mic(15),
        sidetone: sidetone(0),
        volume_limiter: true,
    };
    for name in [
        "reply_20_flat_headset_on.hex",
        "reply_20_flat_headset_off.hex",
    ] {
        assert_eq!(audio(name), expected, "{name}");
    }
}

#[test]
fn audio_readbacks_show_each_written_value() {
    let cases = [
        ("reply_20_after_sidetone_3.hex", mic(15), sidetone(3), true),
        ("reply_20_after_sidetone_6.hex", mic(15), sidetone(6), true),
        (
            "reply_20_after_sidetone_10.hex",
            mic(15),
            sidetone(10),
            true,
        ),
        (
            "reply_20_after_mic_volume_14.hex",
            mic(14),
            sidetone(0),
            true,
        ),
        (
            "reply_20_after_limiter_off.hex",
            mic(15),
            sidetone(0),
            false,
        ),
    ];
    for (name, mic_volume, sidetone, volume_limiter) in cases {
        let readback = audio(name);
        let trailer = (
            readback.mic_volume,
            readback.sidetone,
            readback.volume_limiter,
        );
        assert_eq!(trailer, (mic_volume, sidetone, volume_limiter), "{name}");
    }
}

#[test]
fn audio_readbacks_after_curve_writes_show_the_written_curve() {
    let pairs = [
        ("reply_20_after_33_odd.hex", "cmd_33_odd.hex"),
        ("reply_20_after_33_kinds.hex", "cmd_33_kinds.hex"),
        ("reply_20_after_33_edges.hex", "cmd_33_edges.hex"),
    ];
    for (reply, command) in pairs {
        assert_eq!(
            Ok(audio(reply).equalizer),
            decode_curve(&golden(command)),
            "{reply}"
        );
    }
}

#[test]
fn ux_replies_decode_auto_power_off_and_mute_light() {
    let cases = [
        ("reply_a0_headset_on.hex", 30, MuteLight::High),
        ("reply_a0_headset_off.hex", 30, MuteLight::High),
        ("reply_a0_after_auto_off_15.hex", 15, MuteLight::High),
        ("reply_a0_after_mute_light_0.hex", 30, MuteLight::Off),
        ("reply_a0_after_mute_light_1.hex", 30, MuteLight::Low),
        ("reply_a0_after_mute_light_4.hex", 30, MuteLight::Medium),
    ];
    for (name, minutes, mute_light) in cases {
        let expected = UxReadback {
            auto_power_off: auto_off(minutes),
            mute_light,
        };
        assert_eq!(ux(name), expected, "{name}");
    }
}

#[test]
fn wireless_replies_decode_the_mode_whatever_their_stale_tail() {
    let cases = [
        ("reply_36_speed_headset_on.hex", WirelessMode::Standard),
        ("reply_36_speed_headset_off.hex", WirelessMode::Standard),
        (
            "reply_36_range_headset_off.hex",
            WirelessMode::ExtendedRange,
        ),
        ("reply_36_tail_from_fwupd_b0.hex", WirelessMode::Standard),
    ];
    for (name, mode) in cases {
        assert_eq!(wireless(name), mode, "{name}");
    }
}

#[test]
fn the_initial_headset_off_readback_assembles_the_observed_settings() {
    let observed = observed_settings(
        audio("reply_20_flat_headset_off.hex"),
        ux("reply_a0_headset_off.hex"),
        wireless("reply_36_speed_headset_off.hex"),
    );
    let expected = HeadsetSettings {
        mic_volume: mic(15),
        sidetone: sidetone(0),
        mute_light: MuteLight::High,
        volume_limiter: true,
        wireless_mode: WirelessMode::Standard,
        auto_power_off: auto_off(30),
        equalizer: factory_flat(),
    };
    assert_eq!(observed, expected);
}
