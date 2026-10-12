use super::command::Query;
use super::eq_wire::decode_curve;
use super::firmware::{FIRMWARE_LEN, Firmware, decode_firmware};
use super::frame::{DecodeError, expect_frame, field, flag_field};
use super::status::Status;
use crate::eq::Curve;
use crate::settings::headset_settings::HeadsetSettings;
use crate::settings::values::{AutoPowerOff, MicVolume, MuteLight, Sidetone, WirelessMode};

const STATUS_LEN: usize = 5;
const AUDIO_LEN: usize = 64;
const UX_LEN: usize = 3;
const WIRELESS_LEN: usize = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioReadback {
    pub equalizer: Curve,
    pub mic_volume: MicVolume,
    pub sidetone: Sidetone,
    pub volume_limiter: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UxReadback {
    pub auto_power_off: AutoPowerOff,
    pub mute_light: MuteLight,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    Status(Status),
    Firmware(Firmware),
    Audio(AudioReadback),
    Ux(UxReadback),
    Wireless(WirelessMode),
}

pub fn decode_reply(query: Query, bytes: &[u8]) -> Result<Reply, DecodeError> {
    match query {
        Query::Status => decode_status(bytes).map(Reply::Status),
        Query::Firmware => expect_frame(bytes, query.id(), FIRMWARE_LEN)
            .map(|frame| Reply::Firmware(decode_firmware(frame))),
        Query::Audio => decode_audio(bytes).map(Reply::Audio),
        Query::Ux => decode_ux(bytes).map(Reply::Ux),
        Query::Wireless => decode_wireless(bytes).map(Reply::Wireless),
    }
}

#[must_use]
pub fn observed_settings(
    audio: AudioReadback,
    ux: UxReadback,
    wireless_mode: WirelessMode,
) -> HeadsetSettings {
    HeadsetSettings {
        mic_volume: audio.mic_volume,
        sidetone: audio.sidetone,
        mute_light: ux.mute_light,
        volume_limiter: audio.volume_limiter,
        wireless_mode,
        auto_power_off: ux.auto_power_off,
        equalizer: audio.equalizer,
    }
}

fn decode_status(bytes: &[u8]) -> Result<Status, DecodeError> {
    let frame = expect_frame(bytes, Query::Status.id(), STATUS_LEN)?;
    Ok(Status {
        link: field(frame, 1)?,
        battery: field(frame, 3)?,
        charging: field(frame, 4)?,
    })
}

fn decode_audio(bytes: &[u8]) -> Result<AudioReadback, DecodeError> {
    let frame = expect_frame(bytes, Query::Audio.id(), AUDIO_LEN)?;
    Ok(AudioReadback {
        equalizer: decode_curve(frame)?,
        mic_volume: field(frame, 61)?,
        sidetone: field(frame, 62)?,
        volume_limiter: flag_field(frame, 63)?,
    })
}

fn decode_ux(bytes: &[u8]) -> Result<UxReadback, DecodeError> {
    let frame = expect_frame(bytes, Query::Ux.id(), UX_LEN)?;
    Ok(UxReadback {
        auto_power_off: AutoPowerOff::nearest(frame[1]),
        mute_light: MuteLight::from_raw(frame[2]),
    })
}

fn decode_wireless(bytes: &[u8]) -> Result<WirelessMode, DecodeError> {
    let frame = expect_frame(bytes, Query::Wireless.id(), WIRELESS_LEN)?;
    field(frame, 1)
}
