use std::cell::RefCell;
use std::rc::Rc;

use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib;

use super::background::Hold;
use crate::ports::Lifetime;

pub struct AppHold {
    application: gio::Application,
    guard: RefCell<Option<gio::ApplicationHoldGuard>>,
}

impl AppHold {
    #[must_use]
    pub fn new(application: &gio::Application) -> Self {
        Self {
            application: application.clone(),
            guard: RefCell::default(),
        }
    }
}

impl Hold for AppHold {
    fn hold(&self) {
        if self.guard.borrow().is_none() {
            self.guard.replace(Some(self.application.hold()));
        }
    }

    fn release(&self) {
        self.guard.take();
    }
}

pub struct QuitWhenAlone {
    hold: Rc<dyn Hold>,
    alone: Box<dyn Fn() -> bool>,
    quit: Box<dyn Fn()>,
}

impl QuitWhenAlone {
    #[must_use]
    pub fn new(hold: Rc<dyn Hold>, alone: Box<dyn Fn() -> bool>, quit: Box<dyn Fn()>) -> Self {
        Self { hold, alone, quit }
    }
}

impl Hold for QuitWhenAlone {
    fn hold(&self) {
        self.hold.hold();
    }

    fn release(&self) {
        if (self.alone)() {
            (self.quit)();
        } else {
            self.hold.release();
        }
    }
}

pub struct AppLifetime {
    application: glib::WeakRef<gio::Application>,
}

impl AppLifetime {
    #[must_use]
    pub fn new(application: &gio::Application) -> Self {
        Self {
            application: application.downgrade(),
        }
    }
}

impl Lifetime for AppLifetime {
    fn ready_to_quit(&self) {
        if let Some(application) = self.application.upgrade() {
            application.quit();
        }
    }
}
