use std::fs;
use std::path::Path;

use arctis_core::eq::{Filter, FilterKind};
use arctis_core::messages::device::{DeviceUpdate, Phase};
use arctis_core::session::timing::ENUMERATION_SETTLE;
use arctis_core::settings::headset_settings::HeadsetSettings;
use arctis_core::settings::values::{MicVolume, Percent, Sidetone};
use arctis_core::testkit::{Behaviour, FakeHeadset, Simulation};

pub const SETTLED: u64 = ENUMERATION_SETTLE + 1_000;

pub struct Arrow {
    pub name: &'static str,
    pub from: fn() -> Simulation,
    pub trigger: fn(&mut Simulation),
    pub to: Phase,
}

pub const fn arrow(
    name: &'static str,
    from: fn() -> Simulation,
    trigger: fn(&mut Simulation),
    to: Phase,
) -> Arrow {
    Arrow {
        name,
        from,
        trigger,
        to,
    }
}

pub fn assert_arrows(arrows: &[Arrow]) {
    let mut names: Vec<_> = arrows.iter().map(|arrow| arrow.name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), arrows.len(), "each arrow is named once");
    for arrow in arrows {
        let mut simulation = (arrow.from)();
        (arrow.trigger)(&mut simulation);
        assert_eq!(simulation.phase(), arrow.to, "{}", arrow.name);
    }
}

pub fn silent() -> Behaviour {
    Behaviour {
        reply_latency: None,
        ..Behaviour::default()
    }
}

pub fn absent() -> Simulation {
    Simulation::new(FakeHeadset::nova_5())
}

pub fn plugged(headset: FakeHeadset) -> Simulation {
    let mut simulation = Simulation::new(headset);
    simulation.plug();
    simulation
}

pub fn settling() -> Simulation {
    plugged(FakeHeadset::nova_5())
}

pub fn probing() -> Simulation {
    let mut simulation = settling();
    simulation.run_for(ENUMERATION_SETTLE);
    simulation
}

pub fn probing_silent() -> Simulation {
    let mut simulation = plugged(FakeHeadset::nova_5().with_behaviour(silent()));
    simulation.run_for(ENUMERATION_SETTLE);
    simulation
}

pub fn syncing() -> Simulation {
    let mut simulation = probing();
    simulation.run_for(1);
    simulation
}

pub fn syncing_silent() -> Simulation {
    let mut simulation = syncing();
    simulation.headset_mut().behaviour = silent();
    simulation
}

pub fn headset_off() -> Simulation {
    let mut simulation = plugged(FakeHeadset::nova_5().powered_off());
    simulation.run_until(Phase::HeadsetOff, SETTLED);
    simulation
}

pub fn ready() -> Simulation {
    ready_with(FakeHeadset::nova_5())
}

pub fn ready_with(headset: FakeHeadset) -> Simulation {
    let mut simulation = plugged(headset);
    simulation.run_until(Phase::Ready, SETTLED);
    simulation
}

pub fn observed(simulation: &Simulation) -> Option<HeadsetSettings> {
    simulation
        .updates()
        .iter()
        .rev()
        .find_map(|(_, update)| match update {
            DeviceUpdate::Observed(settings) => Some(settings.clone()),
            _ => None,
        })
}

pub fn published(simulation: &Simulation, wanted: impl Fn(&DeviceUpdate) -> bool) -> usize {
    simulation
        .updates()
        .iter()
        .filter(|(_, update)| wanted(update))
        .count()
}

pub fn sent(simulation: &Simulation, id: u8) -> usize {
    simulation
        .writes()
        .iter()
        .filter(|(_, frame)| frame.payload()[0] == id)
        .count()
}

pub fn with_sidetone(settings: &HeadsetSettings, level: u8) -> HeadsetSettings {
    HeadsetSettings {
        sidetone: sidetone(level),
        ..settings.clone()
    }
}

pub fn sidetone(level: u8) -> Sidetone {
    Sidetone::try_from(level).expect("sidetone")
}

pub fn mic(level: u8) -> MicVolume {
    MicVolume::try_from(level).expect("mic volume")
}

pub fn percent(value: u8) -> Percent {
    Percent::try_from(value).expect("percent")
}

pub fn golden(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name);
    let text = fs::read_to_string(path).expect("golden file is readable");
    text.split_whitespace()
        .map(|byte| u8::from_str_radix(byte, 16).expect("golden bytes are hex"))
        .collect()
}

pub fn filter(kind: FilterKind, hz: u16, half_db: i8, milli_q: u16) -> Filter {
    Filter::from_parts(kind, hz, half_db, milli_q).expect("filter in envelope")
}
