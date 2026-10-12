use std::io::Cursor;

use pipewire::spa;
use spa::param::audio::{AudioFormat, AudioInfoRaw, MAX_CHANNELS};
use spa::pod::serialize::PodSerializer;
use spa::pod::{Object, Value};
use spa::sys::{
    SPA_AUDIO_CHANNEL_FL, SPA_AUDIO_CHANNEL_FR, SPA_PARAM_EnumFormat, SPA_TYPE_OBJECT_Format,
};

use crate::samples::CHANNELS;

#[must_use]
pub fn pod_bytes(value: &Value) -> Vec<u8> {
    PodSerializer::serialize(Cursor::new(Vec::new()), value)
        .map(|(cursor, _)| cursor.into_inner())
        .unwrap_or_default()
}

#[must_use]
pub fn format_pod(rate: Option<u32>) -> Vec<u8> {
    let mut info = AudioInfoRaw::new();
    info.set_format(AudioFormat::F32LE);
    info.set_channels(u32::try_from(CHANNELS).unwrap_or_default());
    info.set_position(stereo_positions());
    if let Some(rate) = rate {
        info.set_rate(rate);
    }
    pod_bytes(&Value::Object(Object {
        type_: SPA_TYPE_OBJECT_Format,
        id: SPA_PARAM_EnumFormat,
        properties: info.into(),
    }))
}

fn stereo_positions() -> [u32; MAX_CHANNELS] {
    let mut position = [0; MAX_CHANNELS];
    position[0] = SPA_AUDIO_CHANNEL_FL;
    position[1] = SPA_AUDIO_CHANNEL_FR;
    position
}
