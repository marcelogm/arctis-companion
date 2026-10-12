use std::io::Cursor;

use pipewire::spa;
use spa::pod::deserialize::PodDeserializer;
use spa::pod::serialize::PodSerializer;
use spa::pod::{Object, Pod, Property, PropertyFlags, Value, ValueArray};
use spa::sys::{
    SPA_PARAM_Props, SPA_PROP_channelVolumes, SPA_PROP_mute, SPA_PROP_softMute,
    SPA_PROP_softVolumes, SPA_TYPE_OBJECT_Props,
};

pub const CHANNEL_VOLUMES: u32 = SPA_PROP_channelVolumes;
pub const MUTE: u32 = SPA_PROP_mute;
pub const SOFT_VOLUMES: u32 = SPA_PROP_softVolumes;
pub const SOFT_MUTE: u32 = SPA_PROP_softMute;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VolumeChange {
    pub channels: Option<Vec<f32>>,
    pub mute: Option<bool>,
}

impl VolumeChange {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.channels.is_none() && self.mute.is_none()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Volume {
    pub channels: Vec<f32>,
    pub mute: bool,
}

impl Volume {
    #[must_use]
    pub fn merged(&self, change: &VolumeChange) -> Self {
        Self {
            channels: change
                .channels
                .clone()
                .unwrap_or_else(|| self.channels.clone()),
            mute: change.mute.unwrap_or(self.mute),
        }
    }
}

#[derive(Debug, Default)]
pub struct VolumeMirror {
    latest: Volume,
    forwarded: Option<Volume>,
}

impl VolumeMirror {
    pub fn apply(&mut self, change: VolumeChange) -> Option<Volume> {
        if let Some(channels) = change.channels {
            self.latest.channels = channels;
        }
        if let Some(mute) = change.mute {
            self.latest.mute = mute;
        }
        let fresh =
            !self.latest.channels.is_empty() && self.forwarded.as_ref() != Some(&self.latest);
        fresh.then(|| {
            self.forwarded = Some(self.latest.clone());
            self.latest.clone()
        })
    }
}

#[must_use]
pub fn volume_change(pod: &Pod) -> VolumeChange {
    match PodDeserializer::deserialize_any_from(pod.as_bytes()) {
        Ok((_, Value::Object(object))) => object
            .properties
            .into_iter()
            .fold(VolumeChange::default(), absorb),
        _ => VolumeChange::default(),
    }
}

fn absorb(mut change: VolumeChange, property: Property) -> VolumeChange {
    match (property.key, property.value) {
        (CHANNEL_VOLUMES, Value::ValueArray(ValueArray::Float(channels))) => {
            change.channels = Some(channels);
        }
        (MUTE, Value::Bool(mute)) => change.mute = Some(mute),
        _ => {}
    }
    change
}

#[must_use]
pub fn props_pod(volume: &Volume) -> Vec<u8> {
    serialize_props(vec![
        property(
            CHANNEL_VOLUMES,
            Value::ValueArray(ValueArray::Float(volume.channels.clone())),
        ),
        property(MUTE, Value::Bool(volume.mute)),
    ])
}

fn serialize_props(properties: Vec<Property>) -> Vec<u8> {
    let props = Value::Object(Object {
        type_: SPA_TYPE_OBJECT_Props,
        id: SPA_PARAM_Props,
        properties,
    });
    PodSerializer::serialize(Cursor::new(Vec::new()), &props)
        .map(|(cursor, _)| cursor.into_inner())
        .unwrap_or_default()
}

fn property(key: u32, value: Value) -> Property {
    Property {
        key,
        flags: PropertyFlags::empty(),
        value,
    }
}
