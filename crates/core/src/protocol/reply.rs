use super::command::Query;
use super::eq_wire::{BANDS_LEN, FIRST_BAND, decode_curve};
use super::firmware::{Firmware, decode_firmware};
use super::frame::{DecodeError, byte, expect_frame, field, flag_field};
use super::status::Status;
use crate::eq::Curve;
use crate::settings::headset_settings::HeadsetSettings;
use crate::settings::values::{AutoPowerOff, MicVolume, MuteLight, Sidetone, WirelessMode};

const LINK_AT: usize = 1;
const BATTERY_AT: usize = 3;
const CHARGING_AT: usize = 4;
const STATUS_LEN: usize = CHARGING_AT + 1;
const MIC_VOLUME_AT: usize = FIRST_BAND + BANDS_LEN;
const SIDETONE_AT: usize = MIC_VOLUME_AT + 1;
const LIMITER_AT: usize = MIC_VOLUME_AT + 2;
const AUDIO_LEN: usize = LIMITER_AT + 1;
const AUTO_POWER_OFF_AT: usize = 1;
const MUTE_LIGHT_AT: usize = 2;
const UX_LEN: usize = MUTE_LIGHT_AT + 1;
const WIRELESS_MODE_AT: usize = 1;
const WIRELESS_LEN: usize = WIRELESS_MODE_AT + 1;

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
        Query::Firmware => decode_firmware(bytes).map(Reply::Firmware),
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
    let frame = expect_frame::<STATUS_LEN>(bytes, Query::Status.id())?;
    Ok(Status {
        link: field(frame, LINK_AT)?,
        battery: field(frame, BATTERY_AT)?,
        charging: field(frame, CHARGING_AT)?,
    })
}

fn decode_audio(bytes: &[u8]) -> Result<AudioReadback, DecodeError> {
    let frame = expect_frame::<AUDIO_LEN>(bytes, Query::Audio.id())?;
    Ok(AudioReadback {
        equalizer: decode_curve(frame)?,
        mic_volume: field(frame, MIC_VOLUME_AT)?,
        sidetone: field(frame, SIDETONE_AT)?,
        volume_limiter: flag_field(frame, LIMITER_AT)?,
    })
}

fn decode_ux(bytes: &[u8]) -> Result<UxReadback, DecodeError> {
    let frame = expect_frame::<UX_LEN>(bytes, Query::Ux.id())?;
    Ok(UxReadback {
        auto_power_off: AutoPowerOff::nearest(byte(frame, AUTO_POWER_OFF_AT)?),
        mute_light: MuteLight::from_raw(byte(frame, MUTE_LIGHT_AT)?),
    })
}

fn decode_wireless(bytes: &[u8]) -> Result<WirelessMode, DecodeError> {
    let frame = expect_frame::<WIRELESS_LEN>(bytes, Query::Wireless.id())?;
    field(frame, WIRELESS_MODE_AT)
}
