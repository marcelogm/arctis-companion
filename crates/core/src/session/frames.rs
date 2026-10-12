use super::input::Interface;
use super::phase::Stage;
use super::reducer::Session;
use super::syncing::Readings;
use crate::messages::device::DeviceUpdate;
use crate::protocol::Reply;
use crate::protocol::event::{DeviceEvent, decode_event};
use crate::protocol::status::{Link, Status};
use crate::settings::headset_settings::SettingChange;

impl Session {
    pub(super) fn command_frame(&mut self, bytes: &[u8]) {
        if self.stage.drains_command_frames() {
            return;
        }
        match self.exchange.answer(bytes) {
            Some(reply) => self.on_reply(reply),
            None => self.on_event(bytes),
        }
    }

    pub(super) fn unsolicited(&mut self, interface: Interface, bytes: &[u8]) {
        let drained = match interface {
            Interface::Command => self.stage.drains_command_frames(),
            Interface::Events => !self.stage.is_present(),
        };
        if !drained {
            self.on_event(bytes);
        }
    }

    fn on_reply(&mut self, reply: Reply) {
        match reply {
            Reply::Status(status) => self.on_status(status),
            other => self.on_readback(&other),
        }
    }

    fn on_status(&mut self, status: Status) {
        self.status = Some(status);
        self.last_status_at = self.now;
        self.publish_status(status);
        self.on_link(status.link);
    }

    fn on_readback(&mut self, reply: &Reply) {
        if let Stage::Syncing(readings) = &mut self.stage
            && !readings.is_complete()
        {
            readings.absorb(reply.clone());
            self.after_read();
        }
    }

    fn after_read(&mut self) {
        let Stage::Syncing(readings) = &self.stage else {
            return;
        };
        let Some((settings, firmware)) = readings.finish() else {
            return;
        };
        self.observed = Some(settings);
        if let Some(dongle) = self.dongle.clone() {
            self.publish(DeviceUpdate::Identity {
                model: dongle.model,
                uniq: dongle.uniq,
                firmware,
            });
        }
        self.publish_observed();
    }

    fn on_event(&mut self, bytes: &[u8]) {
        match decode_event(bytes) {
            Some(DeviceEvent::Setting(change)) => self.on_setting_event(change),
            Some(DeviceEvent::Battery(battery)) => {
                self.last_status_at = self.now;
                self.update_status(|status| status.battery = battery);
            }
            Some(DeviceEvent::Charging(charging)) => {
                self.update_status(|status| status.charging = charging);
            }
            Some(DeviceEvent::Link(link)) => self.on_link_event(link),
            Some(DeviceEvent::ChatMix(_)) | None => {}
        }
    }

    fn on_setting_event(&mut self, change: SettingChange) {
        if let Some(observed) = &mut self.observed {
            observed.apply(change.clone());
        }
        self.publish(DeviceUpdate::HeadsetChanged(change));
    }

    fn on_link_event(&mut self, link: Link) {
        self.update_status(|status| status.link = link);
        if let Stage::HeadsetOff { next_poll } = &mut self.stage
            && link.is_connected()
        {
            *next_poll = self.now;
            return;
        }
        self.on_link(link);
    }

    fn on_link(&mut self, link: Link) {
        match (&self.stage, link.is_connected()) {
            (Stage::Probing | Stage::HeadsetOff { .. }, true) => {
                self.enter(Stage::Syncing(Readings::default()));
            }
            (Stage::Probing | Stage::Syncing(_) | Stage::Ready, false) => self.headset_off(),
            _ => {}
        }
    }

    fn update_status(&mut self, change: impl FnOnce(&mut Status)) {
        if let Some(mut status) = self.status {
            change(&mut status);
            self.status = Some(status);
            self.publish_status(status);
        }
    }

    fn publish_status(&mut self, status: Status) {
        self.publish(DeviceUpdate::Status {
            battery: status.battery,
            charging: status.charging,
            link: status.link,
        });
    }
}
