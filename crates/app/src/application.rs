use std::io;
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::logging::LOG_DOMAIN;
use crate::runtime::Runtime;
use crate::threads::Threads;

pub const APP_ID: &str = "io.github.marcelogm.ArctisCompanion";
const QUIT_ACCELERATOR: &str = "<Control>q";

pub type ThreadFactory = fn() -> io::Result<Threads>;

mod imp {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use adw::subclass::prelude::*;
    use gtk::{gio, glib};

    use super::ThreadFactory;
    use crate::runtime::Runtime;

    #[derive(Default)]
    pub struct CompanionApplication {
        pub threads: Cell<Option<ThreadFactory>>,
        pub runtime: RefCell<Option<Rc<Runtime>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CompanionApplication {
        const NAME: &'static str = "ArctisCompanionApplication";
        type Type = super::CompanionApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for CompanionApplication {}

    impl ApplicationImpl for CompanionApplication {
        fn dbus_register(
            &self,
            connection: &gio::DBusConnection,
            path: &str,
        ) -> Result<(), glib::Error> {
            self.parent_dbus_register(connection, path)?;
            self.obj().export(connection, path)
        }

        fn dbus_unregister(&self, connection: &gio::DBusConnection, path: &str) {
            self.obj().unexport();
            self.parent_dbus_unregister(connection, path);
        }

        fn startup(&self) {
            self.parent_startup();
            self.obj().install_actions();
            let _ = self.obj().runtime();
        }

        fn activate(&self) {
            self.parent_activate();
            self.obj().present();
        }

        fn shutdown(&self) {
            self.obj().stop_runtime();
            self.parent_shutdown();
        }
    }

    impl GtkApplicationImpl for CompanionApplication {}

    impl AdwApplicationImpl for CompanionApplication {}
}

glib::wrapper! {
    pub struct CompanionApplication(ObjectSubclass<imp::CompanionApplication>)
        @extends adw::Application, gtk::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl CompanionApplication {
    fn runtime(&self) -> Option<Rc<Runtime>> {
        let imp = self.imp();
        if let Some(runtime) = imp.runtime.borrow().as_ref() {
            return Some(Rc::clone(runtime));
        }
        let threads = imp.threads.take()?;
        match threads() {
            Ok(threads) => {
                let runtime = Runtime::start(self.upcast_ref(), threads);
                imp.runtime.replace(Some(Rc::clone(&runtime)));
                Some(runtime)
            }
            Err(error) => {
                glib::g_critical!(
                    LOG_DOMAIN,
                    "the device and audio threads did not start: {error}"
                );
                self.quit();
                None
            }
        }
    }

    fn export(&self, connection: &gio::DBusConnection, path: &str) -> Result<(), glib::Error> {
        self.runtime()
            .map_or(Ok(()), |runtime| runtime.export(connection, path))
    }

    fn unexport(&self) {
        if let Some(runtime) = self.imp().runtime.borrow().as_ref() {
            runtime.unexport();
        }
    }

    fn present(&self) {
        if let Some(runtime) = self.runtime() {
            runtime.present(self.upcast_ref());
        }
    }

    fn stop_runtime(&self) {
        if let Some(runtime) = self.imp().runtime.take() {
            runtime.unexport();
        }
    }

    fn install_actions(&self) {
        let quit = gio::ActionEntry::builder("quit")
            .activate(|application: &Self, _, _| application.quit_gracefully())
            .build();
        let battery = gio::ActionEntry::builder("show-battery")
            .activate(|application: &Self, _, _| application.show_battery())
            .build();
        self.add_action_entries([quit, battery]);
        self.set_accels_for_action("app.quit", &[QUIT_ACCELERATOR]);
    }

    fn quit_gracefully(&self) {
        match self.runtime() {
            Some(runtime) => runtime.quit(),
            None => self.quit(),
        }
    }

    fn show_battery(&self) {
        if let Some(runtime) = self.runtime() {
            runtime.show_battery(self.upcast_ref());
        }
    }
}

#[must_use]
pub fn build(threads: ThreadFactory) -> CompanionApplication {
    let application: CompanionApplication = glib::Object::builder()
        .property("application-id", APP_ID)
        .build();
    application.imp().threads.set(Some(threads));
    application
}

#[must_use]
pub fn run() -> glib::ExitCode {
    match arctis_ui::register_resources() {
        Ok(()) => build(Threads::system).run(),
        Err(error) => {
            glib::g_critical!(LOG_DOMAIN, "bundled resources failed to register: {error}");
            glib::ExitCode::FAILURE
        }
    }
}
