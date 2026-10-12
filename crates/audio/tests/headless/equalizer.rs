use arctis_core::messages::audio::{AudioCommand, AudioEvent, SinkKind};

use crate::lab::{BLUEZ, EQ_SINK, Lab, USB, bluetooth, eventually, flat};
use crate::tools;

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
    assert_eq!(tools::configured_default().as_deref(), Some(EQ_SINK));
    lab.send(AudioCommand::RemoveEqualizer);
    lab.wait_for("bluez default again", |event| *event == bluetooth(false));
    eventually("the configured bluez default", || {
        tools::configured_default().as_deref() == Some(BLUEZ)
    });
    eventually("the equalizer sink gone", || {
        tools::node_id(EQ_SINK).is_none()
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
        tools::node_id(EQ_SINK).is_none()
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
        tools::node_id(EQ_SINK).is_none()
    });
}
