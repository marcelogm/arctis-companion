use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib;

use crate::bus::{Bus, own_name};

const NAME: &str = "org.freedesktop.portal.Desktop";
const PATH: &str = "/org/freedesktop/portal/desktop";
const INTERFACE: &str = "org.freedesktop.portal.Background";
const XML: &str = r#"<node><interface name="org.freedesktop.portal.Background"><method name="RequestBackground"><arg type="s" name="parent_window" direction="in"/><arg type="a{sv}" name="options" direction="in"/><arg type="o" name="handle" direction="out"/></method><method name="SetStatus"><arg type="a{sv}" name="options" direction="in"/></method><property name="version" type="u" access="read"/></interface></node>"#;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub autostart: bool,
    pub commandline: Vec<String>,
}

#[derive(Default)]
pub struct PortalState {
    pub grant: Cell<bool>,
    pub requests: RefCell<Vec<Request>>,
    pub statuses: RefCell<Vec<String>>,
}

pub struct PortalService {
    connection: gio::DBusConnection,
    owner: Option<gio::OwnerId>,
    registration: Option<gio::RegistrationId>,
    pub state: Rc<PortalState>,
}

impl PortalService {
    pub fn start(bus: &Bus) -> Self {
        let connection = bus.client();
        let state = Rc::<PortalState>::default();
        let node = gio::DBusNodeInfo::for_xml(XML).expect("portal interface");
        let interface = node.lookup_interface(INTERFACE).expect("interface");
        let handler = Rc::clone(&state);
        let registration = connection
            .register_object(PATH, &interface)
            .method_call(
                move |connection, sender, _, _, method, parameters, invocation| {
                    answer(
                        &Call {
                            state: &handler,
                            connection: &connection,
                            sender: sender.unwrap_or_default(),
                        },
                        method,
                        &parameters,
                        invocation,
                    );
                },
            )
            .property(|_, _, _, _, _| 2_u32.to_variant())
            .build()
            .expect("portal registered");
        let owner = own_name(&connection, NAME);
        Self {
            connection,
            owner: Some(owner),
            registration: Some(registration),
            state,
        }
    }

    pub fn stop(mut self) {
        if let Some(owner) = self.owner.take() {
            gio::bus_unown_name(owner);
        }
        if let Some(registration) = self.registration.take() {
            self.connection
                .unregister_object(registration)
                .expect("unregistered");
        }
    }
}

struct Call<'a> {
    state: &'a PortalState,
    connection: &'a gio::DBusConnection,
    sender: &'a str,
}

fn answer(
    call: &Call,
    method: &str,
    parameters: &glib::Variant,
    invocation: gio::DBusMethodInvocation,
) {
    match method {
        "RequestBackground" => request_background(call, parameters, invocation),
        "SetStatus" => {
            let message = options(parameters, 0)
                .and_then(|dict| dict.lookup::<String>("message").ok().flatten());
            call.state.statuses.borrow_mut().extend(message);
            invocation.return_value(None);
        }
        _ => invocation.return_dbus_error("org.freedesktop.DBus.Error.UnknownMethod", method),
    }
}

fn options(parameters: &glib::Variant, index: usize) -> Option<glib::VariantDict> {
    parameters
        .try_child_value(index)?
        .get::<glib::VariantDict>()
}

fn request_background(
    call: &Call,
    parameters: &glib::Variant,
    invocation: gio::DBusMethodInvocation,
) {
    let dict = options(parameters, 1).unwrap_or_default();
    let token = dict
        .lookup::<String>("handle_token")
        .ok()
        .flatten()
        .unwrap_or_default();
    call.state.requests.borrow_mut().push(Request {
        autostart: dict
            .lookup::<bool>("autostart")
            .ok()
            .flatten()
            .unwrap_or(false),
        commandline: dict
            .lookup::<Vec<String>>("commandline")
            .ok()
            .flatten()
            .unwrap_or_default(),
    });
    let handle = request_path(call.sender, &token);
    let path = glib::variant::ObjectPath::try_from(handle.clone()).expect("object path");
    invocation.return_value(Some(&(path,).to_variant()));
    let granted = call.state.grant.get();
    let results: HashMap<String, glib::Variant> = HashMap::from([
        ("background".to_owned(), granted.to_variant()),
        ("autostart".to_owned(), granted.to_variant()),
    ]);
    call.connection
        .emit_signal(
            Some(call.sender),
            &handle,
            "org.freedesktop.portal.Request",
            "Response",
            Some(&(0_u32, results).to_variant()),
        )
        .expect("Response sent");
}

fn request_path(sender: &str, token: &str) -> String {
    let sender = sender.trim_start_matches(':').replace('.', "_");
    format!("{PATH}/request/{sender}/{token}")
}
