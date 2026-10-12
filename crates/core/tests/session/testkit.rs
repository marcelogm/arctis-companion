use arctis_core::eq::{Curve, FilterKind};
use arctis_core::protocol::{Command, Query, encode};
use arctis_core::session::Interface;
use arctis_core::settings::headset_settings::SettingChange;
use arctis_core::settings::values::Percent;
use arctis_core::testkit::{Behaviour, FakeHeadset, frame_from_report};
use arctis_core::time::Millis;

use crate::support::{filter, golden, sidetone};

fn ask(headset: &mut FakeHeadset, query: Query) -> Vec<u8> {
    let emissions = headset.receive(&encode(&Command::from(query)), Millis(0));
    emissions.into_iter().next().expect("a reply").bytes
}

#[test]
fn replies_with_the_headset_on_match_the_captures() {
    let mut headset = FakeHeadset::nova_5();
    assert_eq!(
        ask(&mut headset, Query::Firmware),
        golden("reply_10_headset_on.hex")
    );
    assert_eq!(
        ask(&mut headset, Query::Status),
        golden("reply_b0_link3_headset_on.hex")
    );
    assert_eq!(
        ask(&mut headset, Query::Audio),
        golden("reply_20_flat_headset_on.hex")
    );
    assert_eq!(
        ask(&mut headset, Query::Ux),
        golden("reply_a0_headset_on.hex")
    );
}

#[test]
fn the_wireless_reply_keeps_the_audio_reply_tail() {
    let mut headset = FakeHeadset::nova_5();
    ask(&mut headset, Query::Audio);
    assert_eq!(
        ask(&mut headset, Query::Wireless),
        golden("reply_36_speed_headset_on.hex")
    );
}

#[test]
fn replies_with_the_headset_off_match_the_captures() {
    let mut headset = FakeHeadset::nova_5()
        .powered_off()
        .with_battery(Percent::try_from(32).expect("percent"));
    ask(&mut headset, Query::Audio);
    assert_eq!(
        ask(&mut headset, Query::Ux),
        golden("reply_a0_headset_off.hex")
    );
    assert_eq!(
        ask(&mut headset, Query::Status),
        golden("reply_b0_link2_headset_off.hex")
    );
    ask(&mut headset, Query::Audio);
    assert_eq!(
        ask(&mut headset, Query::Firmware),
        golden("reply_10_headset_off.hex")
    );
}

#[test]
fn another_claimant_leaves_its_status_tail_behind() {
    let mut headset = FakeHeadset::nova_5();
    ask(&mut headset, Query::Audio);
    headset.claimed_by_other();
    assert_eq!(
        ask(&mut headset, Query::Wireless),
        golden("reply_36_tail_from_fwupd_b0.hex")
    );
}

#[test]
fn a_setter_changes_the_register_it_names() {
    let mut headset = FakeHeadset::nova_5();
    headset.receive(&encode(&Command::SetSidetone(sidetone(6))), Millis(0));
    assert_eq!(headset.settings().sidetone, sidetone(6));
}

#[test]
fn a_write_lands_after_the_readback_latency() {
    let mut headset = FakeHeadset::nova_5().with_behaviour(Behaviour {
        readback_latency: 300,
        ..Behaviour::default()
    });
    headset.receive(&encode(&Command::SetSidetone(sidetone(6))), Millis(0));
    ask(&mut headset, Query::Audio);
    assert_eq!(headset.settings().sidetone, sidetone(0));
    headset.receive(&encode(&Command::from(Query::Audio)), Millis(300));
    assert_eq!(headset.settings().sidetone, sidetone(6));
}

#[test]
fn a_quantizing_headset_stores_q_in_steps_of_ten() {
    let mut headset = FakeHeadset::nova_5().with_behaviour(Behaviour {
        quantizes_q: true,
        ..Behaviour::default()
    });
    let curve = Curve::from_array([filter(FilterKind::Peaking, 1_000, 4, 1_414)]);
    headset.receive(&encode(&Command::SetEqualizer(curve)), Millis(0));
    assert_eq!(headset.settings().equalizer.filters()[0].q().milli(), 1_410);
}

#[test]
fn an_echoing_headset_reports_the_written_value_as_an_event() {
    let mut headset = FakeHeadset::nova_5().with_behaviour(Behaviour {
        echo_delay: Some(20),
        ..Behaviour::default()
    });
    let emissions = headset.receive(&encode(&Command::SetSidetone(sidetone(6))), Millis(100));
    assert_eq!(emissions.len(), 1);
    assert_eq!(emissions[0].at, Millis(120));
    assert_eq!(emissions[0].interface, Interface::Events);
    assert_eq!(emissions[0].bytes[..2], [0x39, 0x06]);
}

#[test]
fn a_press_changes_the_register_and_sends_an_event() {
    let mut headset = FakeHeadset::nova_5();
    let emissions = headset.press(SettingChange::Sidetone(sidetone(3)), Millis(5));
    assert_eq!(headset.settings().sidetone, sidetone(3));
    assert_eq!(emissions[0].bytes[..2], [0x39, 0x03]);
}

#[test]
fn a_silent_headset_answers_nothing() {
    let mut headset = FakeHeadset::nova_5().with_behaviour(Behaviour {
        reply_latency: None,
        ..Behaviour::default()
    });
    let emissions = headset.receive(&encode(&Command::from(Query::Status)), Millis(0));
    assert!(emissions.is_empty());
}

#[test]
fn a_written_report_reads_back_as_its_frame() {
    let frame = encode(&Command::from(Query::Status));
    assert_eq!(frame_from_report(frame.as_bytes()), Some(frame));
}

#[test]
fn a_report_of_the_wrong_shape_is_refused() {
    assert_eq!(frame_from_report(&[0xB0; 65]), None);
    assert_eq!(frame_from_report(&[0; 64]), None);
}
