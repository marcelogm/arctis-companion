use arctis_core::eq::{Curve, FilterKind};
use arctis_core::messages::device::{DeviceUpdate, Phase, SyncPolicy};
use arctis_core::protocol::status::Link;
use arctis_core::session::timing::{COMMAND_SPACING, REPLY_TIMEOUT, VERIFY_BACKOFF};
use arctis_core::settings::headset_settings::{HeadsetSettings, Setting};
use arctis_core::settings::values::{AutoPowerOff, MuteLight};
use arctis_core::testkit::{Behaviour, FakeHeadset, Simulation};

use crate::support::{SETTLED, filter, mic, observed, published, ready_with, sent, with_sidetone};

pub fn ready_behaving(behaviour: Behaviour) -> Simulation {
    let mut simulation = ready_with(FakeHeadset::nova_5().with_behaviour(behaviour));
    simulation.clear_logs();
    simulation
}

pub fn desire(simulation: &mut Simulation, settings: &HeadsetSettings) {
    simulation.desire(settings.clone(), SyncPolicy::Enforce);
}

fn sidetone_bytes(simulation: &Simulation) -> Vec<u8> {
    simulation
        .writes()
        .iter()
        .map(|(_, frame)| frame.payload())
        .filter(|payload| payload[0] == 0x39)
        .map(|payload| payload[1])
        .collect()
}

fn write_failures(simulation: &Simulation) -> Vec<Setting> {
    simulation
        .updates()
        .iter()
        .filter_map(|(_, update)| match update {
            DeviceUpdate::WriteFailed(setting) => Some(*setting),
            _ => None,
        })
        .collect()
}

#[test]
fn a_setter_is_followed_by_its_readback_after_the_first_backoff() {
    let mut simulation = ready_behaving(Behaviour::default());
    let desired = with_sidetone(simulation.headset().settings(), 6);
    desire(&mut simulation, &desired);
    simulation.run_for(1_000);
    let writes = simulation.writes();
    assert_eq!(simulation.sent_ids(), [0x39, 0x20]);
    assert_eq!(writes[1].0.since(writes[0].0), VERIFY_BACKOFF[0]);
    assert_eq!(observed(&simulation), Some(desired));
    assert!(write_failures(&simulation).is_empty());
}

#[test]
fn a_value_visible_only_at_the_400_ms_readback_passes() {
    let mut simulation = ready_behaving(Behaviour {
        readback_latency: VERIFY_BACKOFF[2] - 10,
        ..Behaviour::default()
    });
    let desired = with_sidetone(simulation.headset().settings(), 6);
    desire(&mut simulation, &desired);
    simulation.run_for(1_000);
    assert_eq!(simulation.sent_ids(), [0x39, 0x20, 0x20, 0x20]);
    let start = simulation.writes()[0].0;
    let offsets: Vec<u64> = simulation.writes()[1..]
        .iter()
        .map(|(at, _)| at.since(start))
        .collect();
    assert_eq!(offsets, VERIFY_BACKOFF);
    assert_eq!(observed(&simulation), Some(desired));
    assert!(write_failures(&simulation).is_empty());
}

#[test]
fn a_mismatch_is_retried_once_then_reported() {
    let mut simulation = ready_behaving(Behaviour {
        drops_writes: true,
        ..Behaviour::default()
    });
    let before = simulation.headset().settings().clone();
    desire(&mut simulation, &with_sidetone(&before, 6));
    simulation.run_for(10_000);
    let attempt = [0x39, 0x20, 0x20, 0x20];
    assert_eq!(simulation.sent_ids(), [attempt, attempt].concat());
    assert_eq!(write_failures(&simulation), [Setting::Sidetone]);
    assert_eq!(observed(&simulation), Some(before));
}

#[test]
fn a_failed_setting_waits_for_the_next_desired_change() {
    let mut simulation = ready_behaving(Behaviour {
        drops_writes: true,
        ..Behaviour::default()
    });
    let desired = with_sidetone(simulation.headset().settings(), 6);
    desire(&mut simulation, &desired);
    simulation.run_for(10_000);
    simulation.clear_logs();
    simulation.run_for(60_000);
    assert!(simulation.writes().is_empty());
    desire(&mut simulation, &desired);
    simulation.run_for(1);
    assert_eq!(simulation.sent_ids(), [0x39]);
}

#[test]
fn a_failed_setting_is_retried_on_the_next_syncing() {
    let mut simulation = ready_behaving(Behaviour {
        drops_writes: true,
        ..Behaviour::default()
    });
    let desired = with_sidetone(simulation.headset().settings(), 6);
    desire(&mut simulation, &desired);
    simulation.run_for(10_000);
    simulation.power_off();
    simulation.run_for(100);
    simulation.headset_mut().behaviour.drops_writes = false;
    simulation.clear_logs();
    simulation.power_on();
    simulation.run_until(Phase::Ready, SETTLED);
    assert_eq!(sent(&simulation, 0x39), 1);
    assert_eq!(simulation.headset().settings(), &desired);
}

#[test]
fn a_headset_that_quantizes_q_passes_verify() {
    let mut simulation = ready_behaving(Behaviour {
        quantizes_q: true,
        ..Behaviour::default()
    });
    let curve = Curve::from_array([filter(FilterKind::Peaking, 1_000, 4, 1_414)]);
    let desired = HeadsetSettings {
        equalizer: curve,
        ..simulation.headset().settings().clone()
    };
    desire(&mut simulation, &desired);
    simulation.run_for(1_000);
    assert_eq!(simulation.sent_ids(), [0x33, 0x20]);
    assert!(write_failures(&simulation).is_empty());
    let stored = &simulation.headset().settings().equalizer;
    assert_eq!(stored.filters()[0].q().milli(), 1_410);
}

#[test]
fn a_slider_drag_writes_only_the_latest_value() {
    let mut simulation = ready_behaving(Behaviour::default());
    let base = simulation.headset().settings().clone();
    for level in 3..=6 {
        desire(&mut simulation, &with_sidetone(&base, level));
    }
    simulation.run_until_sent(0x39, 1_000);
    for level in 7..=9 {
        desire(&mut simulation, &with_sidetone(&base, level));
    }
    simulation.run_for(1_000);
    assert_eq!(sidetone_bytes(&simulation), [6, 9]);
}

#[test]
fn a_reconcile_batches_writes_then_reads_back_once_per_query_kind() {
    let mut simulation = ready_behaving(Behaviour::default());
    let desired = HeadsetSettings {
        mic_volume: mic(7),
        mute_light: MuteLight::Low,
        auto_power_off: AutoPowerOff::NEVER,
        ..with_sidetone(simulation.headset().settings(), 6)
    };
    desire(&mut simulation, &desired);
    simulation.run_for(2_000);
    assert_eq!(simulation.sent_ids(), [0x37, 0x39, 0xAE, 0xA3, 0x20, 0xA0]);
    let readbacks = &simulation.writes()[4..];
    assert_eq!(readbacks[1].0.since(readbacks[0].0), COMMAND_SPACING);
    assert_eq!(observed(&simulation), Some(desired));
}

#[test]
fn consecutive_writes_keep_the_command_spacing() {
    let mut simulation = ready_behaving(Behaviour::default());
    let desired = HeadsetSettings {
        mic_volume: mic(7),
        mute_light: MuteLight::Low,
        ..with_sidetone(simulation.headset().settings(), 6)
    };
    desire(&mut simulation, &desired);
    simulation.run_for(2_000);
    let times: Vec<_> = simulation.writes().iter().map(|(at, _)| *at).collect();
    assert!(
        times
            .windows(2)
            .all(|pair| pair[1].since(pair[0]) >= COMMAND_SPACING)
    );
    assert_eq!(sent(&simulation, 0x20), 1);
    let observations = published(&simulation, |update| {
        matches!(update, DeviceUpdate::Observed(_))
    });
    assert_eq!(observations, 1);
}

#[test]
fn an_unanswered_readback_after_an_ordinary_write_means_headset_off() {
    let mut simulation = ready_behaving(Behaviour::default());
    let desired = with_sidetone(simulation.headset().settings(), 6);
    desire(&mut simulation, &desired);
    simulation.run_until_sent(0x39, 1_000);
    simulation.headset_mut().behaviour.reply_latency = None;
    simulation.run_for(VERIFY_BACKOFF[0] + 2 * REPLY_TIMEOUT);
    assert_eq!(simulation.phase(), Phase::HeadsetOff);
}

#[test]
fn a_link_drop_during_an_ordinary_write_means_headset_off() {
    let mut simulation = ready_behaving(Behaviour::default());
    let desired = with_sidetone(simulation.headset().settings(), 6);
    desire(&mut simulation, &desired);
    simulation.run_until_sent(0x39, 1_000);
    simulation.link(Link::HeadsetOff);
    simulation.run_for(1);
    assert_eq!(simulation.phase(), Phase::HeadsetOff);
}
