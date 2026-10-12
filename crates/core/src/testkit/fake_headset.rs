use super::replies;
use crate::eq::library::flat;
use crate::eq::{Curve, Filter};
use crate::protocol::command::{EQUALIZER, WIRELESS_MODE};
use crate::protocol::eq_wire::decode_curve;
use crate::protocol::event::{BATTERY, DeviceEvent, LINK, decode_event};
use crate::protocol::frame::PAYLOAD_LEN;
use crate::protocol::status::{Charging, Link, Status};
use crate::protocol::{Command, Frame, Query, encode};
use crate::session::Interface;
use crate::settings::headset_settings::{HeadsetSettings, SettingChange};
use crate::settings::values::{
    AutoPowerOff, MicVolume, MuteLight, Percent, Sidetone, WirelessMode,
};
use crate::time::Millis;

const NOVA_5: u16 = 0x2232;
const FWUPD_STATUS: [u8; 8] = [0xB0, 0x02, 0x02, 0x20, 0x03, 0x00, 0x00, 0x01];
const Q_STEP: u16 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    pub reply_latency: Option<u64>,
    pub readback_latency: u64,
    pub echo_delay: Option<u64>,
    pub event_interface: Interface,
    pub quantizes_q: bool,
    pub drops_writes: bool,
    pub sends_events: bool,
}

impl Default for Behaviour {
    fn default() -> Self {
        Self {
            reply_latency: Some(1),
            readback_latency: 0,
            echo_delay: None,
            event_interface: Interface::Events,
            quantizes_q: false,
            drops_writes: false,
            sends_events: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Emission {
    pub at: Millis,
    pub interface: Interface,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct FakeHeadset {
    pub pid: u16,
    pub uniq: Option<String>,
    pub behaviour: Behaviour,
    settings: HeadsetSettings,
    landing: Vec<(Millis, SettingChange)>,
    status: Status,
    buffer: [u8; PAYLOAD_LEN],
    received: Vec<Frame>,
}

impl FakeHeadset {
    #[must_use]
    pub fn nova_5() -> Self {
        Self {
            pid: NOVA_5,
            uniq: None,
            behaviour: Behaviour::default(),
            settings: factory_settings(),
            landing: Vec::new(),
            status: Status {
                link: Link::Connected,
                battery: percent(37),
                charging: Charging::Discharging,
            },
            buffer: [0; PAYLOAD_LEN],
            received: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_pid(mut self, pid: u16) -> Self {
        self.pid = pid;
        self
    }

    #[must_use]
    pub fn with_behaviour(mut self, behaviour: Behaviour) -> Self {
        self.behaviour = behaviour;
        self
    }

    #[must_use]
    pub fn powered_off(mut self) -> Self {
        self.status.link = Link::HeadsetOff;
        self
    }

    #[must_use]
    pub fn with_battery(mut self, battery: Percent) -> Self {
        self.status.battery = battery;
        self
    }

    #[must_use]
    pub fn settings(&self) -> &HeadsetSettings {
        &self.settings
    }

    #[must_use]
    pub fn status(&self) -> Status {
        self.status
    }

    #[must_use]
    pub fn received(&self) -> &[Frame] {
        &self.received
    }

    pub fn set(&mut self, change: SettingChange) {
        self.settings.apply(change);
    }

    pub fn receive(&mut self, frame: &Frame, now: Millis) -> Vec<Emission> {
        self.received.push(frame.clone());
        self.land(now);
        let payload = frame.payload();
        match payload.first().copied().and_then(query_for) {
            Some(query) => self.reply(query, now),
            None => self.write(payload, now),
        }
    }

    pub fn link_event(&mut self, link: Link, now: Millis) -> Vec<Emission> {
        self.status.link = link;
        self.event(vec![LINK, link.raw()], now)
    }

    pub fn battery_event(&mut self, battery: Percent, now: Millis) -> Vec<Emission> {
        self.status.battery = battery;
        self.event(vec![BATTERY, battery.get()], now)
    }

    pub fn press(&mut self, change: SettingChange, now: Millis) -> Vec<Emission> {
        let bytes = event_bytes(&change);
        self.settings.apply(change);
        self.event(bytes, now)
    }

    pub fn claimed_by_other(&mut self) {
        self.fill(&FWUPD_STATUS);
    }

    #[must_use]
    pub fn other_claimant_reply() -> Vec<u8> {
        let mut bytes = FWUPD_STATUS.to_vec();
        bytes.resize(PAYLOAD_LEN, 0);
        bytes
    }

    fn reply(&mut self, query: Query, now: Millis) -> Vec<Emission> {
        let Some(latency) = self.behaviour.reply_latency else {
            return Vec::new();
        };
        let bytes = self.reply_bytes(query);
        self.fill(&bytes);
        vec![Emission {
            at: now.after(latency),
            interface: Interface::Command,
            bytes: self.buffer.to_vec(),
        }]
    }

    fn reply_bytes(&self, query: Query) -> Vec<u8> {
        match query {
            Query::Status => replies::status(self.status),
            Query::Firmware => replies::firmware(self.status.link.is_connected()),
            Query::Audio => replies::audio(&self.settings),
            Query::Ux => replies::ux(&self.settings, self.status.link.is_connected()),
            Query::Wireless => replies::wireless(&self.settings),
        }
    }

    fn write(&mut self, payload: &[u8], now: Millis) -> Vec<Emission> {
        let Some(change) = self.decode_setter(payload) else {
            return Vec::new();
        };
        if self.behaviour.drops_writes {
            return Vec::new();
        }
        let echo = self.echo(&change, now);
        self.landing
            .push((now.after(self.behaviour.readback_latency), change));
        self.land(now);
        echo
    }

    fn decode_setter(&self, payload: &[u8]) -> Option<SettingChange> {
        match *payload.first()? {
            EQUALIZER => decode_curve(payload)
                .ok()
                .map(|curve| SettingChange::Equalizer(self.stored(curve))),
            WIRELESS_MODE => WirelessMode::try_from(*payload.get(1)?)
                .ok()
                .map(SettingChange::WirelessMode),
            _ => match decode_event(payload)? {
                DeviceEvent::Setting(change) => Some(change),
                _ => None,
            },
        }
    }

    fn stored(&self, curve: Curve) -> Curve {
        if !self.behaviour.quantizes_q {
            return curve;
        }
        let filters = curve
            .filters()
            .iter()
            .map(|filter| quantized(*filter))
            .collect();
        Curve::new(filters).unwrap_or(curve)
    }

    fn echo(&self, change: &SettingChange, now: Millis) -> Vec<Emission> {
        let Some(delay) = self.behaviour.echo_delay else {
            return Vec::new();
        };
        if matches!(
            change,
            SettingChange::Equalizer(_) | SettingChange::WirelessMode(_)
        ) {
            return Vec::new();
        }
        vec![self.emission(event_bytes(change), now.after(delay))]
    }

    fn event(&self, bytes: Vec<u8>, now: Millis) -> Vec<Emission> {
        if !self.behaviour.sends_events {
            return Vec::new();
        }
        vec![self.emission(bytes, now)]
    }

    fn emission(&self, mut bytes: Vec<u8>, at: Millis) -> Emission {
        bytes.resize(PAYLOAD_LEN, 0);
        Emission {
            at,
            interface: self.behaviour.event_interface,
            bytes,
        }
    }

    fn land(&mut self, now: Millis) {
        let (landed, waiting) = self.landing.drain(..).partition(|(at, _)| *at <= now);
        self.landing = waiting;
        for (_, change) in landed {
            self.settings.apply(change);
        }
    }

    fn fill(&mut self, bytes: &[u8]) {
        let length = bytes.len().min(PAYLOAD_LEN);
        self.buffer[..length].copy_from_slice(&bytes[..length]);
    }
}

fn query_for(id: u8) -> Option<Query> {
    Query::ALL.into_iter().find(|query| query.id() == id)
}

fn event_bytes(change: &SettingChange) -> Vec<u8> {
    let frame = encode(&Command::from(change.clone()));
    frame.payload()[..2].to_vec()
}

fn quantized(filter: Filter) -> Filter {
    let milli = filter.q().milli();
    let rounded = (milli + Q_STEP / 2) / Q_STEP * Q_STEP;
    Filter::from_parts(
        filter.kind(),
        filter.hz().get(),
        filter.gain().half_db(),
        rounded,
    )
    .map_or(filter, |rounded| rounded.with_enabled(filter.is_enabled()))
}

fn factory_settings() -> HeadsetSettings {
    HeadsetSettings {
        mic_volume: MicVolume::try_from(15).expect("a factory value is in range"),
        sidetone: Sidetone::try_from(0).expect("a factory value is in range"),
        mute_light: MuteLight::High,
        volume_limiter: true,
        wireless_mode: WirelessMode::Standard,
        auto_power_off: AutoPowerOff::CHOICES[5],
        equalizer: flat().curve().clone(),
    }
}

fn percent(value: u8) -> Percent {
    Percent::try_from(value).expect("a factory value is in range")
}
