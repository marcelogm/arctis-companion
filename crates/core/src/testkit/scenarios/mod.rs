mod intent;
mod lifecycle;
mod rebinding;
mod verify;

use super::fake_headset::{Behaviour, FakeHeadset};
use super::rig::Rig;
use crate::messages::device::{DeviceUpdate, Phase};
use crate::protocol::{Command, encode};
use crate::session::timing::{ENUMERATION_SETTLE, RESCAN_ABSENT};
use crate::settings::headset_settings::{HeadsetSettings, Setting, SettingChange};
use crate::settings::values::Sidetone;

const DISCOVERY: u64 = RESCAN_ABSENT + 100;
const SETTLED: u64 = DISCOVERY + ENUMERATION_SETTLE + 1_000;

pub struct Scenario {
    pub name: &'static str,
    pub run: fn(),
}

const fn scenario(name: &'static str, run: fn()) -> Scenario {
    Scenario { name, run }
}

#[must_use]
pub fn catalogue<R: Rig>() -> Vec<Scenario> {
    let mut scenarios = lifecycle::scenarios::<R>();
    scenarios.extend(rebinding::scenarios::<R>());
    scenarios.extend(verify::scenarios::<R>());
    scenarios.extend(intent::scenarios::<R>());
    scenarios
}

pub fn run_all<R: Rig>() {
    for scenario in catalogue::<R>() {
        eprintln!("scenario: {}", scenario.name);
        (scenario.run)();
    }
}

fn plugged<R: Rig>(headset: FakeHeadset) -> R {
    let mut rig = R::start(headset);
    rig.plug();
    rig
}

fn ready<R: Rig>(behaviour: Behaviour) -> R {
    let mut rig = plugged::<R>(FakeHeadset::nova_5().with_behaviour(behaviour));
    rig.run_until(Phase::Ready, SETTLED);
    rig.clear_logs();
    rig
}

fn headset_off<R: Rig>(behaviour: Behaviour) -> R {
    let headset = FakeHeadset::nova_5()
        .with_behaviour(behaviour)
        .powered_off();
    let mut rig = plugged::<R>(headset);
    rig.run_until(Phase::HeadsetOff, SETTLED);
    rig.clear_logs();
    rig
}

fn with_sidetone(settings: &HeadsetSettings, level: u8) -> HeadsetSettings {
    HeadsetSettings {
        sidetone: sidetone(level),
        ..settings.clone()
    }
}

fn sidetone(level: u8) -> Sidetone {
    Sidetone::try_from(level).expect("a sidetone level in range")
}

fn setter(setting: Setting) -> u8 {
    let value = FakeHeadset::nova_5().settings().value_of(setting);
    encode(&Command::from(value)).payload()[0]
}

fn observed<R: Rig>(rig: &R) -> Option<HeadsetSettings> {
    rig.updates()
        .into_iter()
        .rev()
        .find_map(|(_, update)| match update {
            DeviceUpdate::Observed(settings) => Some(settings),
            _ => None,
        })
}

fn write_failures<R: Rig>(rig: &R) -> Vec<Setting> {
    rig.updates()
        .into_iter()
        .filter_map(|(_, update)| match update {
            DeviceUpdate::WriteFailed(setting) => Some(setting),
            _ => None,
        })
        .collect()
}

fn headset_changes<R: Rig>(rig: &R) -> Vec<SettingChange> {
    rig.updates()
        .into_iter()
        .filter_map(|(_, update)| match update {
            DeviceUpdate::HeadsetChanged(change) => Some(change),
            _ => None,
        })
        .collect()
}
