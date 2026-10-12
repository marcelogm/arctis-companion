use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::{Rc, Weak};
use std::time::Duration;

use arctis_core::battery::sample::Sample;
use arctis_core::companion::seen_apps::SeenApp;
use arctis_core::companion::{Action, Companion, Input, PresetChange, Snapshot};
use arctis_core::time::Millis;

use crate::logging::report;
use crate::ports::{Ports, SnapshotView, Stores};

pub struct Bridge {
    companion: RefCell<Companion>,
    ports: Ports,
    stores: RefCell<Stores>,
    views: RefCell<Vec<Rc<dyn SnapshotView>>>,
    latest: RefCell<Rc<Snapshot>>,
    inbox: RefCell<VecDeque<Input>>,
    draining: Cell<bool>,
    myself: Weak<Self>,
}

impl Bridge {
    #[must_use]
    pub fn new(companion: Companion, ports: Ports, stores: Stores) -> Rc<Self> {
        let latest = companion.snapshot();
        Rc::new_cyclic(|myself| Self {
            companion: RefCell::new(companion),
            ports,
            stores: RefCell::new(stores),
            views: RefCell::default(),
            latest: RefCell::new(latest),
            inbox: RefCell::default(),
            draining: Cell::new(false),
            myself: myself.clone(),
        })
    }

    #[must_use]
    pub fn snapshot(&self) -> Rc<Snapshot> {
        Rc::clone(&self.latest.borrow())
    }

    pub fn attach(&self, view: Rc<dyn SnapshotView>) {
        view.publish(&self.snapshot());
        self.views.borrow_mut().push(view);
    }

    pub fn submit(&self, input: Input) {
        self.inbox.borrow_mut().push_back(input);
        if self.draining.replace(true) {
            return;
        }
        while let Some(input) = self.next_input() {
            self.handle(input);
        }
        self.draining.set(false);
    }

    fn next_input(&self) -> Option<Input> {
        self.inbox.borrow_mut().pop_front()
    }

    fn handle(&self, input: Input) {
        let now = self.ports.clock.now();
        let actions = self.companion.borrow_mut().handle(input, now);
        for action in actions {
            self.perform(action, now);
        }
    }

    fn perform(&self, action: Action, now: Millis) {
        match action {
            Action::Persist(change) => self.ports.settings.persist(&change),
            Action::StorePreset(change) => self.store_preset(&change),
            Action::RecordBattery(sample) => self.record_battery(&sample),
            Action::RecordSeenApp(app) => self.record_seen_app(&app),
            Action::SendDevice(change) => self.ports.device.send(change),
            Action::SendAudio(command) => self.ports.audio.send(command),
            Action::Publish(snapshot) => self.publish(&snapshot),
            Action::Notify(alert) => self.ports.notifier.notify(&alert),
            Action::Withdraw(id) => self.ports.notifier.withdraw(id),
            Action::WakeAt(at) => self.wake_at(at, now),
            Action::ReadyToQuit => self.ports.lifetime.ready_to_quit(),
        }
    }

    fn record_battery(&self, sample: &Sample) {
        report(self.stores.borrow_mut().battery.append(sample));
    }

    fn record_seen_app(&self, app: &SeenApp) {
        report(self.stores.borrow_mut().seen_apps.record(app));
    }

    fn store_preset(&self, change: &PresetChange) {
        let mut stores = self.stores.borrow_mut();
        report(match change {
            PresetChange::Save(preset) => stores.presets.save(preset),
            PresetChange::Delete(id) => stores.presets.delete(id),
        });
    }

    fn publish(&self, snapshot: &Rc<Snapshot>) {
        self.latest.replace(Rc::clone(snapshot));
        let views = self.views.borrow().clone();
        for view in views {
            view.publish(snapshot);
        }
    }

    fn wake_at(&self, at: Millis, now: Millis) {
        let bridge = self.myself.clone();
        let delay = Duration::from_millis(at.since(now));
        self.ports
            .alarm
            .schedule(delay, Box::new(move || wake(&bridge)));
    }
}

fn wake(bridge: &Weak<Bridge>) {
    if let Some(bridge) = bridge.upgrade() {
        bridge.submit(Input::Woke);
    }
}
