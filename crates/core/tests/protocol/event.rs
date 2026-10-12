use arctis_core::protocol::event::{DeviceEvent, decode_event};
use arctis_core::protocol::status::{Charging, ChatMix, Link};
use arctis_core::settings::headset_settings::SettingChange;
use arctis_core::settings::values::MuteLight;
use proptest::collection::vec;
use proptest::prelude::*;

use crate::support::{auto_off, mic, percent, sidetone};

fn event(bytes: &[u8]) -> Option<DeviceEvent> {
    let mut frame = bytes.to_vec();
    frame.resize(64, 0);
    decode_event(&frame)
}

#[test]
fn status_events_decode_battery_charging_and_link() {
    assert_eq!(event(&[0xB7, 73]), Some(DeviceEvent::Battery(percent(73))));
    assert_eq!(
        event(&[0xBB, 1]),
        Some(DeviceEvent::Charging(Charging::Charging))
    );
    assert_eq!(event(&[0xB9, 2]), Some(DeviceEvent::Link(Link::HeadsetOff)));
}

#[test]
fn the_chat_mix_dial_decodes_game_and_chat() {
    let expected = ChatMix {
        game: percent(80),
        chat: percent(20),
    };
    assert_eq!(event(&[0x45, 80, 20]), Some(DeviceEvent::ChatMix(expected)));
}

#[test]
fn setting_events_decode_to_setting_changes() {
    let cases = [
        ([0x27, 0], SettingChange::VolumeLimiter(false)),
        ([0x27, 1], SettingChange::VolumeLimiter(true)),
        ([0x37, 7], SettingChange::MicVolume(mic(7))),
        ([0x39, 6], SettingChange::Sidetone(sidetone(6))),
        ([0xA3, 15], SettingChange::AutoPowerOff(auto_off(15))),
        ([0xA3, 20], SettingChange::AutoPowerOff(auto_off(15))),
        ([0xAE, 4], SettingChange::MuteLight(MuteLight::Medium)),
    ];
    for (bytes, change) in cases {
        assert_eq!(
            event(&bytes),
            Some(DeviceEvent::Setting(change)),
            "{bytes:02x?}"
        );
    }
}

#[test]
fn out_of_range_event_values_are_dropped() {
    let frames: [&[u8]; 8] = [
        &[0xB7, 101],
        &[0xBB, 4],
        &[0xB9, 4],
        &[0x45, 101, 0],
        &[0x45, 0, 101],
        &[0x27, 2],
        &[0x37, 16],
        &[0x39, 11],
    ];
    for bytes in frames {
        assert_eq!(event(bytes), None, "{bytes:02x?}");
    }
}

#[test]
fn unknown_or_truncated_frames_are_not_events() {
    let truncated: [&[u8]; 6] = [&[], &[0xB7], &[0x45, 10], &[0x27], &[0xAE], &[0x10]];
    for bytes in truncated {
        assert_eq!(decode_event(bytes), None, "{bytes:02x?}");
    }
    assert_eq!(event(&[0x10, 1]), None);
    assert_eq!(event(&[0x20]), None);
}

proptest! {
    #[test]
    fn decode_event_never_panics(bytes in vec(any::<u8>(), 0..=64)) {
        let _ = decode_event(&bytes);
    }
}
