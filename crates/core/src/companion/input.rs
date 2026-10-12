use super::app_settings::AppSettings;
use crate::intent::Intent;
use crate::messages::audio::AudioEvent;
use crate::messages::device::DeviceUpdate;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Device(DeviceUpdate),
    Audio(AudioEvent),
    Intent(Intent),
    Woke,
    SettingsChanged(AppSettings),
    Quitting,
}
