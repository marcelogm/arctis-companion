use std::time::Duration;

use arctis_core::companion::Input;
use arctis_core::companion::adoption::DongleIdentity;
use arctis_core::companion::timing::QUIT_GRACE;
use arctis_core::eq::curve::Curve;
use arctis_core::messages::audio::{AudioEvent, DefaultSink, SinkKind, SoundPath};
use arctis_core::messages::device::{DeviceUpdate, Phase};
use arctis_core::protocol::firmware::Firmware;
use arctis_core::settings::headset_settings::HeadsetSettings;
use arctis_core::testkit::snapshots::{nova_5, values};

use crate::fakes::Seen;
use crate::rig::{Rig, curve, device_curves, firefox, with_rule};

const NOVA_5: u16 = 0x2232;

fn observed(preset: &str) -> Input {
    Input::Device(DeviceUpdate::Observed(observed_settings(curve(preset))))
}

fn observed_settings(equalizer: Curve) -> HeadsetSettings {
    values().with_curve(equalizer)
}

fn connect_adopted(rig: &Rig) {
    let _ = rig.send(Input::Device(DeviceUpdate::Phase(Phase::Syncing)));
    let _ = rig.send(Input::Device(DeviceUpdate::Identity {
        model: nova_5(),
        uniq: None,
        firmware: Firmware {
            receiver: None,
            headset: None,
        },
    }));
    let _ = rig.send(observed("flat"));
    let _ = rig.send(Input::Device(DeviceUpdate::Phase(Phase::Ready)));
}

fn app_preset_on_the_headset() -> Rig {
    let mut saved = with_rule("firefox", "bass-boost");
    saved.settings.adopted_identity = Some(DongleIdentity::new(NOVA_5, None));
    let rig = Rig::new(saved);
    connect_adopted(&rig);
    let sink = DefaultSink {
        terminal: SinkKind::NovaDongle,
        via_equalizer: false,
        via_easy_effects: false,
    };
    let _ = rig.send(Input::Audio(AudioEvent::DefaultSink(sink)));
    let _ = rig.send(Input::Audio(AudioEvent::Streams(vec![firefox(Some(
        SoundPath::Dongle,
    ))])));
    rig.clock.advance(1_000);
    rig.alarm.fire();
    assert_eq!(
        device_curves(&rig.take()),
        [curve("bass-boost")],
        "the app preset is written"
    );
    let _ = rig.send(observed("bass-boost"));
    rig.clock.advance(100);
    rig
}

#[test]
fn quitting_sends_the_route_preset_and_waits_for_it() {
    let rig = app_preset_on_the_headset();
    let seen = rig.send(Input::Quitting);
    assert_eq!(device_curves(&seen), [curve("flat")]);
    assert!(seen.contains(&Seen::Scheduled(Duration::from_millis(QUIT_GRACE))));
    assert!(!seen.contains(&Seen::ReadyToQuit));
}

#[test]
fn quit_ends_when_the_route_preset_is_verified() {
    let rig = app_preset_on_the_headset();
    let _ = rig.send(Input::Quitting);
    rig.clock.advance(300);
    assert!(rig.send(observed("flat")).contains(&Seen::ReadyToQuit));
}

#[test]
fn quit_ends_after_the_grace_without_a_verify() {
    let rig = app_preset_on_the_headset();
    let _ = rig.send(Input::Quitting);
    rig.clock.advance(QUIT_GRACE);
    rig.alarm.fire();
    assert!(rig.take().contains(&Seen::ReadyToQuit));
}
