use std::rc::{Rc, Weak};

use adw::prelude::*;
use arctis_core::companion::{Companion, Input, Snapshot};
use arctis_core::intent::Intent;
use gtk::{gio, glib};

use crate::alarm::{GlibAlarm, SystemClock};
use crate::application::APP_ID;
use crate::bridge::Bridge;
use crate::dbus::headset_object::HeadsetObject;
use crate::platform::background::BackgroundController;
use crate::platform::files::data_dir;
use crate::platform::lifetime::{AppHold, AppLifetime, QuitWhenAlone};
use crate::platform::notifications::GioNotifier;
use crate::platform::persisted::{file_stores, load_persisted, settings_only};
use crate::platform::portal::AshpdPortal;
use crate::platform::settings_store::{GSettingsStore, SettingsWatch};
use crate::ports::{AudioPort, Clock, DevicePort, Ports};
use crate::threads::Threads;
use crate::window_slot::{OnClose, WindowSlot};

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
    pub fn start(
        application: &adw::Application,
        threads: Threads,
        headset: Option<Rc<HeadsetObject>>,
    ) -> Rc<Self> {
        let parts = assemble(application, threads);
        let runtime = Rc::new_cyclic(|myself: &Weak<Self>| {
            let window = WindowSlot::new(intent_handler(myself), close_handler(myself));
            let background = BackgroundController::new(
                parts.settings,
                Rc::new(AshpdPortal),
                hold(application, &window, myself),
                window.clone(),
            );
            Self {
                bridge: parts.bridge,
                window,
                background,
                pumps: parts.pumps,
                _watch: parts.watch,
            }
        });
        runtime.bridge.attach(runtime.window.clone());
        runtime.bridge.attach(runtime.background.clone());
        if let Some(headset) = headset {
            headset.on_intent(intent_handler(&Rc::downgrade(&runtime)));
            runtime.bridge.attach(headset);
        }
        runtime.background.start();
        runtime
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

    fn closing(&self) -> OnClose {
        if self.background.is_enabled() {
            return OnClose::ForgetWindow;
        }
        self.quit();
        OnClose::HideUntilQuit
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        for pump in &self.pumps {
            pump.abort();
        }
    }
}

#[must_use]
pub fn stored_snapshot() -> Rc<Snapshot> {
    let store = GSettingsStore::new(gio::Settings::new(APP_ID));
    Companion::new(settings_only(store.load())).snapshot()
}

fn intent_handler(runtime: &Weak<Runtime>) -> Rc<dyn Fn(Intent)> {
    let runtime = runtime.clone();
    Rc::new(move |intent| {
        if let Some(runtime) = runtime.upgrade() {
            runtime.deliver(intent);
        }
    })
}

fn hold(
    application: &adw::Application,
    window: &Rc<WindowSlot>,
    runtime: &Weak<Runtime>,
) -> Rc<QuitWhenAlone> {
    let slot = Rc::downgrade(window);
    let runtime = runtime.clone();
    Rc::new(QuitWhenAlone::new(
        Rc::new(AppHold::new(application.upcast_ref())),
        Box::new(move || slot.upgrade().is_none_or(|slot| slot.window().is_none())),
        Box::new(move || {
            if let Some(runtime) = runtime.upgrade() {
                runtime.quit();
            }
        }),
    ))
}

fn close_handler(runtime: &Weak<Runtime>) -> Rc<dyn Fn() -> OnClose> {
    let runtime = runtime.clone();
    Rc::new(move || {
        runtime
            .upgrade()
            .map_or(OnClose::ForgetWindow, |runtime| runtime.closing())
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
