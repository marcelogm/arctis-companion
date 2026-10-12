use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use arctis_app::bridge::Bridge;
use arctis_app::ports::SnapshotView;
use arctis_core::companion::app_settings::PersistedChange;
use arctis_core::companion::{Input, Snapshot};
use arctis_core::intent::Intent;
use arctis_core::messages::audio::AudioEvent;
use arctis_core::messages::device::{DeviceUpdate, Phase};
use arctis_core::testkit::snapshots::id;
use gtk::glib;

use crate::fakes::{BrokenPresets, Seen};
use crate::rig::{Rig, mine, persisted, published};

fn on_own_context(test: impl FnOnce(&glib::MainContext)) {
    let context = glib::MainContext::new();
    context
        .with_thread_default(|| test(&context))
        .expect("the test owns its main context");
}

fn drain(context: &glib::MainContext) {
    while context.iteration(false) {}
}

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
    bridge: RefCell<Weak<Bridge>>,
    sent: Cell<bool>,
}

impl SnapshotView for Echo {
    fn publish(&self, _snapshot: &Rc<Snapshot>) {
        if self.sent.replace(true) {
            return;
        }
        if let Some(bridge) = self.bridge.borrow().upgrade() {
            bridge.submit(Input::Intent(Intent::SetAutomaticPresets(false)));
        }
    }
}

#[test]
fn an_input_submitted_while_actions_run_waits_its_turn() {
    let rig = Rig::new(persisted());
    let echo = Rc::new(Echo {
        bridge: RefCell::new(Rc::downgrade(&rig.bridge)),
        sent: Cell::new(true),
    });
    rig.bridge.attach(echo.clone());
    echo.sent.set(false);
    let seen = rig.send(Input::Device(DeviceUpdate::Phase(Phase::Settling)));
    assert!(seen.contains(&Seen::Persisted(PersistedChange::AutomaticPresets(false))));
    assert!(!rig.bridge.snapshot().presets.automatic);
}

#[test]
fn a_failing_store_does_not_stop_the_other_actions() {
    let rig = Rig::with_presets(persisted(), Box::new(BrokenPresets));
    let seen = rig.send(Input::Intent(Intent::SavePreset(mine())));
    let snapshot = published(&seen).expect("still published");
    assert!(snapshot.presets.find(&id("mine")).is_some());
}
