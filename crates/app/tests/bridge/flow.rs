use std::cell::Cell;
use std::rc::{Rc, Weak};

use arctis_app::bridge::Bridge;
use arctis_app::ports::SnapshotView;
use arctis_core::companion::app_settings::PersistedChange;
use arctis_core::companion::{Input, Snapshot};
use arctis_core::intent::Intent;
use arctis_core::messages::audio::AudioEvent;
use arctis_core::messages::device::{DeviceUpdate, Phase};
use arctis_core::testkit::snapshots::id;

use crate::fakes::{BrokenPresets, Seen};
use crate::main_context::{drain, on_own_context};
use crate::rig::{Rig, mine, persisted, published};

#[test]
fn device_updates_are_pumped_into_the_companion() {
    on_own_context(|context| {
        let rig = Rig::new(persisted());
        let (sender, receiver) = async_channel::unbounded();
        let _pump = rig.bridge.pump_device(receiver);
        sender
            .send_blocking(DeviceUpdate::Phase(Phase::Settling))
            .expect("send");
        drain(context);
        assert_eq!(rig.bridge.snapshot().dongle.phase, Phase::Settling);
    });
}

#[test]
fn audio_events_are_pumped_into_the_companion() {
    on_own_context(|context| {
        let rig = Rig::new(persisted());
        let (sender, receiver) = async_channel::unbounded();
        let _pump = rig.bridge.pump_audio(receiver);
        sender
            .send_blocking(AudioEvent::ManagerDenied)
            .expect("send");
        drain(context);
        assert!(rig.bridge.snapshot().audio.manager_denied);
    });
}

struct Echo {
    bridge: Weak<Bridge>,
    armed: Cell<bool>,
    automatic_right_after_submit: Cell<Option<bool>>,
}

impl Echo {
    fn armed_after_attach(bridge: &Rc<Bridge>) -> Rc<Self> {
        let echo = Rc::new(Self {
            bridge: Rc::downgrade(bridge),
            armed: Cell::new(false),
            automatic_right_after_submit: Cell::new(None),
        });
        bridge.attach(echo.clone());
        echo.armed.set(true);
        echo
    }
}

impl SnapshotView for Echo {
    fn publish(&self, _snapshot: &Rc<Snapshot>) {
        if !self.armed.replace(false) {
            return;
        }
        if let Some(bridge) = self.bridge.upgrade() {
            bridge.submit(Input::Intent(Intent::SetAutomaticPresets(false)));
            let automatic = bridge.snapshot().presets.automatic;
            self.automatic_right_after_submit.set(Some(automatic));
        }
    }
}

fn settling() -> Input {
    Input::Device(DeviceUpdate::Phase(Phase::Settling))
}

#[test]
fn an_input_submitted_while_actions_run_waits_its_turn() {
    let alone = Rig::new(persisted()).send(settling());
    let rig = Rig::new(persisted());
    let echo = Echo::armed_after_attach(&rig.bridge);
    let seen = rig.send(settling());
    assert_eq!(
        echo.automatic_right_after_submit.get(),
        Some(true),
        "the nested input has not run yet"
    );
    assert_eq!(
        seen[..alone.len()],
        alone[..],
        "the first input finishes first"
    );
    let echoed = Seen::Persisted(PersistedChange::AutomaticPresets(false));
    assert!(seen[alone.len()..].contains(&echoed));
    assert!(!rig.bridge.snapshot().presets.automatic);
}

#[test]
fn a_failing_store_does_not_stop_the_other_actions() {
    let rig = Rig::with_presets(persisted(), Box::new(BrokenPresets));
    let seen = rig.send(Input::Intent(Intent::SavePreset(mine())));
    let snapshot = published(&seen).expect("still published");
    assert!(snapshot.presets.find(&id("mine")).is_some());
}
