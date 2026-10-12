use std::cell::RefCell;
use std::rc::{Rc, Weak};

use adw::prelude::*;
use arctis_core::companion::{Companion, Input};
use arctis_core::intent::Intent;
use gtk::{gio, glib};

use crate::alarm::{GlibAlarm, SystemClock};
use crate::application::APP_ID;
use crate::bridge::Bridge;
use crate::dbus::headset_object::HeadsetObject;
use crate::platform::background::BackgroundController;
use crate::platform::files::data_dir;
use crate::platform::lifetime::{AppHold, AppLifetime};
use crate::platform::notifications::GioNotifier;
use crate::platform::persisted::{file_stores, load_persisted};
use crate::platform::portal::AshpdPortal;
use crate::platform::settings_store::{GSettingsStore, SettingsWatch};
use crate::ports::{AudioPort, Clock, DevicePort, Ports};
use crate::threads::Threads;
use crate::window_slot::WindowSlot;

const BATTERY_PAGE: &str = "battery";

#[derive(Debug, PartialEq, Eq)]
pub enum Destination {
    Background(bool),
    Companion(Input),
}

#[must_use]
pub fn destination(intent: Intent) -> Destination {
    match intent {
        Intent::SetRunInBackground(on) => Destination::Background(on),
        other => Destination::Companion(Input::Intent(other)),
    }
}

pub struct Runtime {
    bridge: Rc<Bridge>,
    window: Rc<WindowSlot>,
    background: Rc<BackgroundController>,
    headset: RefCell<Option<Rc<HeadsetObject>>>,
    pumps: Vec<glib::JoinHandle<()>>,
    _watch: SettingsWatch,
}

struct Parts {
    bridge: Rc<Bridge>,
    pumps: Vec<glib::JoinHandle<()>>,
    watch: SettingsWatch,
    settings: gio::Settings,
}

impl Runtime {
    #[must_use]
    pub fn start(application: &adw::Application, threads: Threads) -> Rc<Self> {
        let parts = assemble(application, threads);
        let runtime = Rc::new_cyclic(|myself: &Weak<Self>| {
            let window = WindowSlot::new(intent_handler(myself), close_handler(myself));
            let hold = Rc::new(AppHold::new(application.upcast_ref()));
            let background = BackgroundController::new(
                parts.settings,
                Rc::new(AshpdPortal),
                hold,
                window.clone(),
            );
            Self {
                bridge: parts.bridge,
                window,
                background,
                headset: RefCell::default(),
                pumps: parts.pumps,
                _watch: parts.watch,
            }
        });
        runtime.bridge.attach(runtime.window.clone());
        runtime.bridge.attach(runtime.background.clone());
        runtime.background.start();
        runtime
    }

    pub fn export(
        self: &Rc<Self>,
        connection: &gio::DBusConnection,
        path: &str,
    ) -> Result<(), glib::Error> {
        let object = HeadsetObject::register(connection, path, self.bridge.snapshot())?;
        let runtime = Rc::downgrade(self);
        object.on_intent(move |intent| deliver(&runtime, intent));
        self.bridge.attach(object.clone());
        self.headset.replace(Some(object));
        Ok(())
    }

    pub fn unexport(&self) {
        if let Some(object) = self.headset.take() {
            object.unregister();
        }
    }

    pub fn present(&self, application: &adw::Application) {
        self.window.present(application);
    }

    pub fn show_battery(&self, application: &adw::Application) {
        self.window.show_page(application, BATTERY_PAGE);
    }

    pub fn quit(&self) {
        self.bridge.submit(Input::Quitting);
    }

    fn deliver(&self, intent: Intent) {
        match destination(intent) {
            Destination::Background(on) => self.background.set_enabled(on),
            Destination::Companion(input) => self.bridge.submit(input),
        }
    }

    fn closing(&self) -> glib::Propagation {
        if self.background.is_enabled() {
            return glib::Propagation::Proceed;
        }
        self.quit();
        glib::Propagation::Stop
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        for pump in &self.pumps {
            pump.abort();
        }
    }
}

fn deliver(runtime: &Weak<Runtime>, intent: Intent) {
    if let Some(runtime) = runtime.upgrade() {
        runtime.deliver(intent);
    }
}

fn intent_handler(runtime: &Weak<Runtime>) -> Rc<dyn Fn(Intent)> {
    let runtime = runtime.clone();
    Rc::new(move |intent| deliver(&runtime, intent))
}

fn close_handler(runtime: &Weak<Runtime>) -> Rc<dyn Fn() -> glib::Propagation> {
    let runtime = runtime.clone();
    Rc::new(move || {
        runtime
            .upgrade()
            .map_or(glib::Propagation::Proceed, |runtime| runtime.closing())
    })
}

fn assemble(application: &adw::Application, threads: Threads) -> Parts {
    let settings = gio::Settings::new(APP_ID);
    let store = GSettingsStore::new(settings.clone());
    let Threads {
        device,
        device_updates,
        audio,
        audio_events,
    } = threads;
    let bridge = bridge(application, &store, device, audio);
    let pumps = vec![
        bridge.pump_device(device_updates),
        bridge.pump_audio(audio_events),
    ];
    let watcher = Rc::downgrade(&bridge);
    let watch = store.watch(move |changed| {
        if let Some(bridge) = watcher.upgrade() {
            bridge.submit(Input::SettingsChanged(changed));
        }
    });
    Parts {
        bridge,
        pumps,
        watch,
        settings,
    }
}

fn bridge(
    application: &adw::Application,
    store: &GSettingsStore,
    device: Box<dyn DevicePort>,
    audio: Box<dyn AudioPort>,
) -> Rc<Bridge> {
    let clock = SystemClock;
    let mut stores = file_stores(&data_dir());
    let persisted = load_persisted(store.load(), &mut stores, clock.now());
    let ports = Ports {
        settings: Box::new(store.clone()),
        device,
        audio,
        notifier: Box::new(GioNotifier::new(application.clone().upcast())),
        alarm: Box::<GlibAlarm>::default(),
        clock: Box::new(clock),
        lifetime: Box::new(AppLifetime::new(application.upcast_ref())),
    };
    Bridge::new(Companion::new(persisted), ports, stores)
}
