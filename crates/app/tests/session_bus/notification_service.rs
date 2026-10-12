use std::cell::RefCell;
use std::rc::Rc;

use gtk::gio;
use gtk::glib;

use crate::bus::{Bus, wait_until};

const NAME: &str = "org.gtk.Notifications";
const PATH: &str = "/org/gtk/Notifications";
const XML: &str = r#"<node><interface name="org.gtk.Notifications"><method name="AddNotification"><arg type="s" direction="in"/><arg type="s" direction="in"/><arg type="a{sv}" direction="in"/></method><method name="RemoveNotification"><arg type="s" direction="in"/><arg type="s" direction="in"/></method></interface></node>"#;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shown {
    Added {
        id: String,
        title: String,
        default_action: String,
    },
    Removed(String),
}

pub struct NotificationService {
    connection: gio::DBusConnection,
    owner: Option<gio::OwnerId>,
    registration: Option<gio::RegistrationId>,
    log: Rc<RefCell<Vec<Shown>>>,
}

impl NotificationService {
    pub fn start(bus: &Bus) -> Self {
        let connection = bus.client();
        let log: Rc<RefCell<Vec<Shown>>> = Rc::default();
        let node = gio::DBusNodeInfo::for_xml(XML).expect("notification interface");
        let interface = node.lookup_interface(NAME).expect("interface");
        let writer = Rc::clone(&log);
        let registration = connection
            .register_object(PATH, &interface)
            .method_call(move |_, _, _, _, method, parameters, invocation| {
                answer(&writer, method, &parameters, invocation);
            })
            .build()
            .expect("notification server registered");
        let owner = own(&connection);
        Self {
            connection,
            owner: Some(owner),
            registration: Some(registration),
            log,
        }
    }

    pub fn shown(&self) -> Vec<Shown> {
        self.log.borrow().clone()
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

fn answer(
    log: &RefCell<Vec<Shown>>,
    method: &str,
    parameters: &glib::Variant,
    invocation: gio::DBusMethodInvocation,
) {
    log.borrow_mut().extend(record(method, parameters));
    invocation.return_value(None);
}

fn own(connection: &gio::DBusConnection) -> gio::OwnerId {
    let acquired = Rc::new(std::cell::Cell::new(false));
    let flag = Rc::clone(&acquired);
    let owner = gio::bus_own_name_on_connection(
        connection,
        NAME,
        gio::BusNameOwnerFlags::NONE,
        move |_, _| flag.set(true),
        |_, _| {},
    );
    wait_until("org.gtk.Notifications to be owned", || acquired.get());
    owner
}

fn record(method: &str, parameters: &glib::Variant) -> Option<Shown> {
    match method {
        "AddNotification" => {
            let (_, id, notification) = parameters.get::<(String, String, glib::VariantDict)>()?;
            let title = notification.lookup::<String>("title").ok()??;
            let default_action = notification.lookup::<String>("default-action").ok()??;
            Some(Shown::Added {
                id,
                title,
                default_action,
            })
        }
        "RemoveNotification" => {
            let (_, id) = parameters.get::<(String, String)>()?;
            Some(Shown::Removed(id))
        }
        _ => None,
    }
}
