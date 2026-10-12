use std::io;
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::dbus::headset_object::HeadsetObject;
use crate::logging::LOG_DOMAIN;
use crate::runtime::{Runtime, stored_snapshot};
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
    use crate::dbus::headset_object::HeadsetObject;
    use crate::runtime::Runtime;

    #[derive(Default)]
    pub struct CompanionApplication {
        pub threads: Cell<Option<ThreadFactory>>,
        pub headset: RefCell<Option<Rc<HeadsetObject>>>,
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
            self.obj().register_headset(connection, path)
        }

        fn dbus_unregister(&self, connection: &gio::DBusConnection, path: &str) {
            self.headset.take();
            self.parent_dbus_unregister(connection, path);
        }

        fn startup(&self) {
            self.parent_startup();
            self.obj().install_actions();
            self.obj().start_runtime();
        }

        fn activate(&self) {
            self.parent_activate();
            self.obj().present();
        }

        fn shutdown(&self) {
            self.obj().drop_runtime();
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
    fn register_headset(
        &self,
        connection: &gio::DBusConnection,
        path: &str,
    ) -> Result<(), glib::Error> {
        let object = HeadsetObject::register(connection, path, stored_snapshot())?;
        self.imp().headset.replace(Some(object));
        Ok(())
    }

    fn start_runtime(&self) {
        let imp = self.imp();
        let Some(threads) = imp.threads.take() else {
            return;
        };
        match threads() {
            Ok(threads) => {
                let runtime = Runtime::start(self.upcast_ref(), threads, imp.headset.take());
                imp.runtime.replace(Some(runtime));
            }
            Err(error) => {
                glib::g_critical!(
                    LOG_DOMAIN,
                    "the device and audio threads did not start: {error}"
                );
                self.quit();
            }
        }
    }

    fn running_runtime(&self) -> Option<Rc<Runtime>> {
        self.imp().runtime.borrow().clone()
    }

    fn drop_runtime(&self) {
        self.imp().runtime.take();
    }

    fn present(&self) {
        if let Some(runtime) = self.running_runtime() {
            runtime.present(self.upcast_ref());
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
        match self.running_runtime() {
            Some(runtime) => runtime.quit(),
            None => self.quit(),
        }
    }

    fn show_battery(&self) {
        if let Some(runtime) = self.running_runtime() {
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
