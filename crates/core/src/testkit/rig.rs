use super::fake_headset::{Emission, FakeHeadset};
use super::simulation::{Flap, Simulation};
use crate::messages::device::{DeviceUpdate, Phase, SyncPolicy};
use crate::protocol::Frame;
use crate::protocol::status::Link;
use crate::settings::headset_settings::{HeadsetSettings, SettingChange};
use crate::settings::values::Percent;
use crate::time::Millis;

pub trait Rig: Sized {
    fn start(headset: FakeHeadset) -> Self;
    fn plug(&mut self);
    fn deny(&mut self);
    fn unplug(&mut self);
    fn desire(&mut self, settings: HeadsetSettings, policy: SyncPolicy);
    fn flap(&mut self, flap: Flap);
    fn remove_command_node(&mut self);
    fn fail_next_write(&mut self);
    fn press(&mut self, change: SettingChange);
    fn link(&mut self, link: Link);
    fn battery(&mut self, battery: Percent);
    fn emit(&mut self, emissions: Vec<Emission>);
    fn run_for(&mut self, duration: u64);
    fn now(&self) -> Millis;
    fn phase(&self) -> Phase;
    fn headset(&self) -> FakeHeadset;
    fn writes(&self) -> Vec<(Millis, Frame)>;
    fn updates(&self) -> Vec<(Millis, DeviceUpdate)>;
    fn clear_logs(&mut self);

    fn run_until(&mut self, phase: Phase, limit: u64) -> u64 {
        let start = self.now();
        while self.phase() != phase {
            assert!(
                self.now().since(start) < limit,
                "{phase:?} not reached within {limit} ms"
            );
            self.run_for(1);
        }
        self.now().since(start)
    }

    fn sent_ids(&self) -> Vec<u8> {
        self.writes()
            .iter()
            .map(|(_, frame)| frame.payload()[0])
            .collect()
    }

    fn published_phases(&self) -> Vec<Phase> {
        self.updates()
            .into_iter()
            .filter_map(|(_, update)| match update {
                DeviceUpdate::Phase(phase) => Some(phase),
                _ => None,
            })
            .collect()
    }
}

impl Rig for Simulation {
    fn start(headset: FakeHeadset) -> Self {
        Self::new(headset)
    }

    fn plug(&mut self) {
        Self::plug(self);
    }

    fn deny(&mut self) {
        Self::deny(self);
    }

    fn unplug(&mut self) {
        Self::unplug(self);
    }

    fn desire(&mut self, settings: HeadsetSettings, policy: SyncPolicy) {
        Self::desire(self, settings, policy);
    }

    fn flap(&mut self, flap: Flap) {
        Self::flap(self, flap);
    }

    fn remove_command_node(&mut self) {
        Self::remove_command_node(self);
    }

    fn fail_next_write(&mut self) {
        Self::fail_next_write(self);
    }

    fn press(&mut self, change: SettingChange) {
        Self::press(self, change);
    }

    fn link(&mut self, link: Link) {
        Self::link(self, link);
    }

    fn battery(&mut self, battery: Percent) {
        Self::battery(self, battery);
    }

    fn emit(&mut self, emissions: Vec<Emission>) {
        Self::emit(self, emissions);
    }

    fn run_for(&mut self, duration: u64) {
        Self::run_for(self, duration);
    }

    fn now(&self) -> Millis {
        Self::now(self)
    }

    fn phase(&self) -> Phase {
        Self::phase(self)
    }

    fn headset(&self) -> FakeHeadset {
        Self::headset(self).clone()
    }

    fn writes(&self) -> Vec<(Millis, Frame)> {
        Self::writes(self).to_vec()
    }

    fn updates(&self) -> Vec<(Millis, DeviceUpdate)> {
        Self::updates(self).to_vec()
    }

    fn clear_logs(&mut self) {
        Self::clear_logs(self);
    }
}
