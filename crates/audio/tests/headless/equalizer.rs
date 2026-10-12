use std::thread;
use std::time::Duration;

use arctis_core::messages::audio::{AudioCommand, AudioEvent, SinkKind};

use crate::lab::{BLUEZ, EQ_SINK_NAME, Lab, USB, bluetooth, eventually, flat};
use crate::tools;

const QUIET: Duration = Duration::from_secs(1);
const SETTLED: Duration = Duration::from_secs(3);

#[test]
#[ignore = "needs the private PipeWire of crates/audio/tests/headless/run.sh"]
fn the_bluez_default_is_reported_on_start() {
    let lab = Lab::open();
    lab.wait_for("bluetooth seen", |event| {
        *event == AudioEvent::BluetoothNodeSeen
    });
    lab.wait_for("bluez default", |event| *event == bluetooth(false));
}

#[test]
#[ignore = "needs the private PipeWire of crates/audio/tests/headless/run.sh"]
fn inserting_makes_the_equalizer_the_default_and_removing_restores_bluez() {
    let lab = Lab::open();
    lab.insert(flat());
    assert_eq!(tools::configured_default().as_deref(), Some(EQ_SINK_NAME));
    lab.send(AudioCommand::RemoveEqualizer);
    lab.wait_for("bluez default again", |event| *event == bluetooth(false));
    eventually("the configured bluez default", || {
        tools::configured_default().as_deref() == Some(BLUEZ)
    });
    eventually("the equalizer sink gone", || {
        tools::node_id(EQ_SINK_NAME).is_none()
    });
}

#[test]
#[ignore = "needs the private PipeWire of crates/audio/tests/headless/run.sh"]
fn a_default_the_user_changed_while_inserted_is_left_alone() {
    let lab = Lab::open();
    lab.insert(flat());
    tools::set_configured_default(USB);
    lab.wait_for(
        "the user's default",
        |event| matches!(event, AudioEvent::DefaultSink(sink) if sink.terminal == SinkKind::Other),
    );
    lab.send(AudioCommand::RemoveEqualizer);
    eventually("the equalizer sink gone", || {
        tools::node_id(EQ_SINK_NAME).is_none()
    });
    assert_eq!(tools::configured_default().as_deref(), Some(USB));
}

#[test]
#[ignore = "needs the private PipeWire of crates/audio/tests/headless/run.sh"]
fn dropping_the_handle_restores_bluez() {
    let lab = Lab::open();
    lab.insert(flat());
    drop(lab);
    eventually("the configured bluez default", || {
        tools::configured_default().as_deref() == Some(BLUEZ)
    });
    eventually("the equalizer sink gone", || {
        tools::node_id(EQ_SINK_NAME).is_none()
    });
}

#[test]
#[ignore = "needs the private PipeWire of crates/audio/tests/headless/run.sh"]
fn the_adapter_connects_as_a_manager() {
    let _lab = Lab::open();
    eventually("one Manager client of this process", || {
        tools::manager_clients_of(std::process::id()) == 1
    });
}

#[test]
#[ignore = "needs the private PipeWire of crates/audio/tests/headless/run.sh"]
fn inserting_over_another_default_changes_nothing() {
    let lab = Lab::open();
    tools::set_configured_default(USB);
    lab.wait_for(
        "the USB default",
        |event| matches!(event, AudioEvent::DefaultSink(sink) if sink.terminal == SinkKind::Other),
    );
    lab.send(AudioCommand::InsertEqualizer(flat()));
    thread::sleep(QUIET);
    assert_eq!(tools::node_id(EQ_SINK_NAME), None);
    assert_eq!(tools::configured_default().as_deref(), Some(USB));
}

#[test]
#[ignore = "needs the private PipeWire of crates/audio/tests/headless/run.sh"]
fn inserting_reports_only_the_equalizer_route() {
    let lab = Lab::open();
    lab.wait_for("bluez default", |event| *event == bluetooth(false));
    lab.send(AudioCommand::InsertEqualizer(flat()));
    assert_eq!(lab.default_sinks_during(SETTLED), [bluetooth(true)]);
}

#[test]
#[ignore = "needs the private PipeWire of crates/audio/tests/headless/run.sh"]
fn removing_reports_only_the_bluez_route() {
    let lab = Lab::open();
    lab.insert(flat());
    lab.default_sinks_during(SETTLED);
    lab.send(AudioCommand::RemoveEqualizer);
    assert_eq!(lab.default_sinks_during(SETTLED), [bluetooth(false)]);
}
