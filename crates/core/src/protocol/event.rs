use super::command::{AUTO_POWER_OFF, MIC_VOLUME, MUTE_LIGHT, SIDETONE, VOLUME_LIMITER};
use super::frame::flag;
use super::status::{Charging, ChatMix, Link};
use crate::settings::headset_settings::SettingChange;
use crate::settings::values::{AutoPowerOff, MuteLight, Percent};

const CHAT_MIX: u8 = 0x45;
const BATTERY: u8 = 0xB7;
const LINK: u8 = 0xB9;
const CHARGING: u8 = 0xBB;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceEvent {
    Setting(SettingChange),
    ChatMix(ChatMix),
    Battery(Percent),
    Charging(Charging),
    Link(Link),
}

#[must_use]
pub fn decode_event(bytes: &[u8]) -> Option<DeviceEvent> {
    match *bytes.first()? {
        BATTERY => value(bytes, 1).map(DeviceEvent::Battery),
        CHARGING => value(bytes, 1).map(DeviceEvent::Charging),
        LINK => value(bytes, 1).map(DeviceEvent::Link),
        CHAT_MIX => chat_mix(bytes).map(DeviceEvent::ChatMix),
        id => setting_change(id, bytes).map(DeviceEvent::Setting),
    }
}

fn value<T: TryFrom<u8>>(bytes: &[u8], offset: usize) -> Option<T> {
    T::try_from(*bytes.get(offset)?).ok()
}

fn chat_mix(bytes: &[u8]) -> Option<ChatMix> {
    Some(ChatMix {
        game: value(bytes, 1)?,
        chat: value(bytes, 2)?,
    })
}

fn setting_change(id: u8, bytes: &[u8]) -> Option<SettingChange> {
    let raw = *bytes.get(1)?;
    match id {
        VOLUME_LIMITER => flag(raw).map(SettingChange::VolumeLimiter),
        MIC_VOLUME => value(bytes, 1).map(SettingChange::MicVolume),
        SIDETONE => value(bytes, 1).map(SettingChange::Sidetone),
        AUTO_POWER_OFF => Some(SettingChange::AutoPowerOff(AutoPowerOff::nearest(raw))),
        MUTE_LIGHT => Some(SettingChange::MuteLight(MuteLight::from_raw(raw))),
        _ => None,
    }
}
