use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use arctis_core::companion::Snapshot;
use arctis_core::l10n::msgid;
use arctis_core::view::preferences::Background;
use arctis_core::view::text::{Arg, Text};
use arctis_ui::text::resolve;
use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib;

use super::settings_keys::{BACKGROUND_REQUESTED, RUN_IN_BACKGROUND};
use crate::logging::warn;
use crate::ports::SnapshotView;

pub trait Portal {
    fn request(&self, answer: Box<dyn FnOnce(bool)>);
    fn set_status(&self, message: String);
}

pub trait Hold {
    fn hold(&self);
    fn release(&self);
}

pub trait BackgroundView {
    fn show_background(&self, state: Background);
}

pub struct BackgroundController {
    settings: gio::Settings,
    portal: Rc<dyn Portal>,
    hold: Rc<dyn Hold>,
    view: Rc<dyn BackgroundView>,
    status: RefCell<Option<String>>,
    held: Cell<bool>,
    watch: RefCell<Option<glib::SignalHandlerId>>,
    myself: Weak<Self>,
}

impl BackgroundController {
    #[must_use]
    pub fn new(
        settings: gio::Settings,
        portal: Rc<dyn Portal>,
        hold: Rc<dyn Hold>,
        view: Rc<dyn BackgroundView>,
    ) -> Rc<Self> {
        let controller = Rc::new_cyclic(|myself| Self {
            settings,
            portal,
            hold,
            view,
            status: RefCell::default(),
            held: Cell::new(false),
            watch: RefCell::default(),
            myself: myself.clone(),
        });
        controller.follow_the_key();
        controller
    }

    fn follow_the_key(&self) {
        let controller = self.myself.clone();
        let handler = self
            .settings
            .connect_changed(Some(RUN_IN_BACKGROUND), move |_, _| {
                key_changed(&controller);
            });
        self.watch.replace(Some(handler));
    }

    fn key_changed(&self) {
        match (self.is_enabled(), self.held.get()) {
            (true, false) => self.keep_running(),
            (false, true) => self.stop_running(Background::Off),
            _ => {}
        }
    }

    pub fn start(&self) {
        if !self.is_enabled() {
            self.view.show_background(Background::Off);
            return;
        }
        self.keep_running();
        if !self.settings.boolean(BACKGROUND_REQUESTED) {
            self.ask();
        }
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.settings.boolean(RUN_IN_BACKGROUND)
    }

    pub fn set_enabled(&self, on: bool) {
        self.store(RUN_IN_BACKGROUND, on);
        if on {
            self.keep_running();
            self.ask();
        } else {
            self.stop_running(Background::Off);
        }
    }

    fn keep_running(&self) {
        self.held.set(true);
        self.hold.hold();
        self.view.show_background(Background::On);
    }

    fn stop_running(&self, state: Background) {
        self.held.set(false);
        self.hold.release();
        self.view.show_background(state);
    }

    fn ask(&self) {
        let controller = self.myself.clone();
        self.portal
            .request(Box::new(move |granted| answer(&controller, granted)));
    }

    fn answered(&self, granted: bool) {
        self.store(BACKGROUND_REQUESTED, true);
        if !granted && self.is_enabled() {
            self.store(RUN_IN_BACKGROUND, false);
            self.stop_running(Background::Denied);
        }
    }

    fn store(&self, key: &str, value: bool) {
        if let Err(error) = self.settings.set_boolean(key, value) {
            warn(&format!("could not save {key}: {error}"));
        }
    }
}

impl Drop for BackgroundController {
    fn drop(&mut self) {
        if let Some(handler) = self.watch.take() {
            self.settings.disconnect(handler);
        }
    }
}

impl SnapshotView for BackgroundController {
    fn publish(&self, snapshot: &Rc<Snapshot>) {
        let message = resolve(&status_text(snapshot));
        if self.status.borrow().as_deref() == Some(message.as_str()) {
            return;
        }
        self.status.replace(Some(message.clone()));
        self.portal.set_status(message);
    }
}

fn key_changed(controller: &Weak<BackgroundController>) {
    if let Some(controller) = controller.upgrade() {
        controller.key_changed();
    }
}

fn answer(controller: &Weak<BackgroundController>, granted: bool) {
    if let Some(controller) = controller.upgrade() {
        controller.answered(granted);
    }
}

#[must_use]
pub fn status_text(snapshot: &Snapshot) -> Text {
    let connected = snapshot
        .battery
        .reading
        .filter(|reading| reading.is_connected() && snapshot.dongle.is_present());
    match connected {
        Some(reading) => Text::format(
            msgid("Headset connected, {percent}"),
            vec![("percent", Arg::Percent(reading.percent))],
        ),
        None => Text::plain(msgid("No headset")),
    }
}
