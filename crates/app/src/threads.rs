use std::io;

use arctis_audio::AudioHandle;
use arctis_core::messages::audio::{AudioCommand, AudioEvent};
use arctis_core::messages::device::{Change, DeviceUpdate};
use arctis_hid::{DeviceHandle, SysfsRoot};

use crate::logging::warn;
use crate::ports::{AudioPort, DevicePort};

pub struct Threads {
    pub device: Box<dyn DevicePort>,
    pub device_updates: async_channel::Receiver<DeviceUpdate>,
    pub audio: Box<dyn AudioPort>,
    pub audio_events: async_channel::Receiver<AudioEvent>,
}

impl Threads {
    pub fn system() -> io::Result<Self> {
        let device = DeviceHandle::spawn(SysfsRoot::system())?;
        let audio = AudioHandle::spawn()?;
        Ok(Self {
            device_updates: device.updates.clone(),
            audio_events: audio.events.clone(),
            device: Box::new(device),
            audio: Box::new(audio),
        })
    }
}

impl DevicePort for DeviceHandle {
    fn send(&self, change: Change) {
        if !self.changes.send(change) {
            warn("the device thread has stopped");
        }
    }
}

impl AudioPort for AudioHandle {
    fn send(&self, command: AudioCommand) {
        if !self.commands.send(command) {
            warn("the audio thread has stopped");
        }
    }
}
