use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::intent::Intent;
use arctis_core::view::preferences::Background;
use arctis_ui::Window;
use gtk::glib;

use crate::platform::background::BackgroundView;
use crate::ports::SnapshotView;

const SHOW_PAGE: &str = "win.show-page";

type IntentHandler = Rc<dyn Fn(Intent)>;
type CloseHandler = Rc<dyn Fn() -> OnClose>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnClose {
    ForgetWindow,
    HideUntilQuit,
}

pub struct WindowSlot {
    window: RefCell<Option<Window>>,
    latest: RefCell<Option<Rc<Snapshot>>>,
    background: Cell<Background>,
    intents: IntentHandler,
    on_close: CloseHandler,
}

impl WindowSlot {
    #[must_use]
    pub fn new(intents: IntentHandler, on_close: CloseHandler) -> Rc<Self> {
        Rc::new(Self {
            window: RefCell::default(),
            latest: RefCell::default(),
            background: Cell::new(Background::default()),
            intents,
            on_close,
        })
    }

    #[must_use]
    pub fn window(&self) -> Option<Window> {
        self.window.borrow().clone()
    }

    pub fn present(self: &Rc<Self>, application: &adw::Application) {
        self.window_for(application).present();
    }

    pub fn show_page(self: &Rc<Self>, application: &adw::Application, tag: &str) {
        let window = self.window_for(application);
        window.present();
        if let Err(error) = WidgetExt::activate_action(&window, SHOW_PAGE, Some(&tag.to_variant()))
        {
            crate::logging::warn(&format!("{SHOW_PAGE} {tag}: {error}"));
        }
    }

    fn window_for(self: &Rc<Self>, application: &adw::Application) -> Window {
        if let Some(window) = self.window() {
            return window;
        }
        let window = self.build(application);
        self.window.replace(Some(window.clone()));
        window
    }

    fn build(self: &Rc<Self>, application: &adw::Application) -> Window {
        let window = Window::new(application);
        let intents = Rc::clone(&self.intents);
        window.on_intent(move |intent| intents(intent));
        window.set_background(self.background.get());
        if let Some(snapshot) = self.latest.borrow().as_ref() {
            window.render(snapshot);
        }
        let slot = Rc::downgrade(self);
        window.connect_close_request(move |window| {
            slot.upgrade()
                .map_or(glib::Propagation::Proceed, |slot| slot.closing(window))
        });
        window
    }

    fn closing(&self, window: &Window) -> glib::Propagation {
        match (self.on_close)() {
            OnClose::ForgetWindow => {
                self.window.replace(None);
                glib::Propagation::Proceed
            }
            OnClose::HideUntilQuit => {
                window.set_visible(false);
                glib::Propagation::Stop
            }
        }
    }
}

impl SnapshotView for WindowSlot {
    fn publish(&self, snapshot: &Rc<Snapshot>) {
        self.latest.replace(Some(Rc::clone(snapshot)));
        if let Some(window) = self.window() {
            window.render(snapshot);
        }
    }
}

impl BackgroundView for WindowSlot {
    fn show_background(&self, state: Background) {
        self.background.set(state);
        if let Some(window) = self.window() {
            window.set_background(state);
        }
    }
}
