use std::thread;
use std::time::{Duration, Instant};

use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib;

use crate::fixtures::{INTERFACE, PATH};

const PATIENCE: Duration = Duration::from_secs(5);
const CALL_TIMEOUT_MS: i32 = 5_000;

pub struct Bus {
    test: gio::TestDBus,
    session: gio::DBusConnection,
}

pub struct Target<'a> {
    pub name: &'a str,
    pub path: &'a str,
    pub interface: &'a str,
}

impl Bus {
    pub fn start() -> Self {
        let test = gio::TestDBus::new(gio::TestDBusFlags::NONE);
        test.up();
        let session = gio::bus_get_sync(gio::BusType::Session, None::<&gio::Cancellable>)
            .expect("session bus");
        Self { test, session }
    }

    pub fn session(&self) -> gio::DBusConnection {
        self.session.clone()
    }

    pub fn client(&self) -> gio::DBusConnection {
        let address = self.test.bus_address().expect("bus address");
        let flags = gio::DBusConnectionFlags::AUTHENTICATION_CLIENT
            | gio::DBusConnectionFlags::MESSAGE_BUS_CONNECTION;
        gio::DBusConnection::for_address_sync(&address, flags, None, None::<&gio::Cancellable>)
            .expect("client connection")
    }

    pub fn stop(self) {
        drop(self.session);
        self.test.stop();
    }
}

pub fn wait_until(what: &str, done: impl Fn() -> bool) {
    let context = glib::MainContext::default();
    let deadline = Instant::now() + PATIENCE;
    while !done() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        context.iteration(false);
        thread::sleep(Duration::from_millis(2));
    }
}

pub fn settle() {
    let context = glib::MainContext::default();
    let deadline = Instant::now() + Duration::from_millis(200);
    while Instant::now() < deadline {
        context.iteration(false);
        thread::sleep(Duration::from_millis(2));
    }
}

pub fn call(
    connection: &gio::DBusConnection,
    target: &Target,
    method: &str,
    parameters: Option<&glib::Variant>,
) -> Result<glib::Variant, glib::Error> {
    let reply = connection.call_future(
        Some(target.name),
        target.path,
        target.interface,
        method,
        parameters,
        None,
        gio::DBusCallFlags::NONE,
        CALL_TIMEOUT_MS,
    );
    glib::MainContext::default().block_on(reply)
}

pub fn property(connection: &gio::DBusConnection, name: &str, property: &str) -> glib::Variant {
    let target = Target {
        name,
        path: PATH,
        interface: "org.freedesktop.DBus.Properties",
    };
    let reply = call(
        connection,
        &target,
        "Get",
        Some(&(INTERFACE, property).to_variant()),
    )
    .expect("property read");
    let (value,) = reply.get::<(glib::Variant,)>().expect("(v)");
    value
}
