use crate::protocol::eq_wire::encode_curve;
use crate::protocol::status::Status;
use crate::settings::headset_settings::HeadsetSettings;

const RECEIVER_AND_UNKNOWN: &[u8; 24] = b"0.01.010.000............";
const HEADSET_REACHABLE: &[u8; 12] = b"000000001.07";
const HEADSET_UNREACHABLE: &[u8; 12] = b"000000000000";
const UX_TRAILER: [u8; 4] = [0x01, 0x01, 0x00, 0x01];
const UX_HEADSET_ON: u8 = 0x00;
const UX_HEADSET_OFF: u8 = 0xFF;

pub(super) fn status(status: Status) -> Vec<u8> {
    let link = status.link.raw();
    vec![
        0xB0,
        link,
        link,
        status.battery.get(),
        status.charging.raw(),
        0x00,
        0x00,
    ]
}

pub(super) fn firmware(headset_on: bool) -> Vec<u8> {
    let headset = if headset_on {
        HEADSET_REACHABLE
    } else {
        HEADSET_UNREACHABLE
    };
    let mut bytes = vec![0x10];
    bytes.extend_from_slice(RECEIVER_AND_UNKNOWN);
    bytes.extend_from_slice(headset);
    bytes
}

pub(super) fn audio(settings: &HeadsetSettings) -> Vec<u8> {
    let mut bytes = vec![0x20];
    bytes.extend_from_slice(&encode_curve(&settings.equalizer));
    bytes.extend_from_slice(&[
        settings.mic_volume.get(),
        settings.sidetone.get(),
        u8::from(settings.volume_limiter),
    ]);
    bytes
}

pub(super) fn ux(settings: &HeadsetSettings, headset_on: bool) -> Vec<u8> {
    let presence = if headset_on {
        UX_HEADSET_ON
    } else {
        UX_HEADSET_OFF
    };
    let mut bytes = vec![
        0xA0,
        settings.auto_power_off.minutes(),
        settings.mute_light.raw(),
        presence,
    ];
    bytes.extend_from_slice(&UX_TRAILER);
    bytes
}

pub(super) fn wireless(settings: &HeadsetSettings) -> Vec<u8> {
    vec![0x36, settings.wireless_mode.raw()]
}
