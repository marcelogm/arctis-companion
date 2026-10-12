use crate::lab::{BLUEZ, EQ_SINK_NAME, Lab, eventually, flat};
use crate::meter::level_db;
use crate::tone::Tone;
use crate::tools;

const UNITY_TOLERANCE_DB: f64 = 0.5;

fn close(found: &[f64], expected: f64) -> bool {
    !found.is_empty() && found.iter().all(|volume| (volume - expected).abs() < 1e-3)
}

#[test]
#[ignore = "needs the private PipeWire of crates/audio/tests/headless/run.sh"]
fn volume_and_mute_reach_the_bluez_node() {
    let lab = Lab::open();
    lab.insert(flat());
    tools::set_props(
        EQ_SINK_NAME,
        "{ channelVolumes = [ 0.25, 0.25 ], mute = true }",
    );
    eventually("the bluez node at 0.25 and muted", || {
        tools::volume_of(BLUEZ).is_some_and(|(channels, mute)| close(&channels, 0.25) && mute)
    });
    tools::set_props(
        EQ_SINK_NAME,
        "{ channelVolumes = [ 1.0, 1.0 ], mute = false }",
    );
    eventually("the bluez node back at 1.0", || {
        tools::volume_of(BLUEZ).is_some_and(|(channels, mute)| close(&channels, 1.0) && !mute)
    });
}

#[test]
#[ignore = "needs the private PipeWire of crates/audio/tests/headless/run.sh"]
fn the_equalizer_sink_processes_at_unity() {
    let lab = Lab::open();
    lab.insert(flat());
    let _tone = Tone::play(Vec::new(), Vec::new());
    let full = level_db(BLUEZ);
    tools::set_props(EQ_SINK_NAME, "{ channelVolumes = [ 0.5, 0.5 ] }");
    eventually("the bluez node at 0.5", || {
        tools::volume_of(BLUEZ).is_some_and(|(channels, _)| close(&channels, 0.5))
    });
    let lowered = level_db(BLUEZ);
    assert!(
        (full - lowered).abs() < UNITY_TOLERANCE_DB,
        "full {full:.2} dB, lowered {lowered:.2} dB"
    );
}

#[test]
#[ignore = "needs the private PipeWire of crates/audio/tests/headless/run.sh"]
fn inserting_keeps_the_bluez_volume() {
    let lab = Lab::open();
    tools::set_props(BLUEZ, "{ channelVolumes = [ 0.3, 0.3 ] }");
    eventually("the bluez node at 0.3", || {
        tools::volume_of(BLUEZ).is_some_and(|(channels, _)| close(&channels, 0.3))
    });
    lab.insert(flat());
    eventually("the equalizer sink at the bluez volume", || {
        tools::volume_of(EQ_SINK_NAME).is_some_and(|(channels, _)| close(&channels, 0.3))
    });
    assert!(tools::volume_of(BLUEZ).is_some_and(|(channels, _)| close(&channels, 0.3)));
}
