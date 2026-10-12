use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use arctis_core::companion::Snapshot;
use arctis_core::eq::preset::PresetId;
use arctis_core::intent::Intent;
use gtk::gio;
use gtk::glib;
use gtk::glib::prelude::*;

use super::properties::{PropertyMap, changed, property_values};
use super::wire_names::{error_message, error_name};
use crate::logging::warn;
use crate::ports::SnapshotView;

pub const INTERFACE: &str = "io.github.marcelogm.ArctisCompanion.Headset1";
const CONTRACT: &str =
    include_str!("../../../../data/dbus/io.github.marcelogm.ArctisCompanion.Headset1.xml");
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";
const UNKNOWN_METHOD: &str = "org.freedesktop.DBus.Error.UnknownMethod";

pub type IntentHandler = Rc<dyn Fn(Intent)>;

pub struct HeadsetObject {
    connection: gio::DBusConnection,
    path: String,
    registration: RefCell<Option<gio::RegistrationId>>,
    snapshot: RefCell<Rc<Snapshot>>,
    values: RefCell<PropertyMap>,
    intents: RefCell<Option<IntentHandler>>,
}

impl HeadsetObject {
    pub fn register(
        connection: &gio::DBusConnection,
        path: &str,
        snapshot: Rc<Snapshot>,
    ) -> Result<Rc<Self>, glib::Error> {
        let object = Rc::new(Self {
            connection: connection.clone(),
            path: path.to_owned(),
            registration: RefCell::default(),
            values: RefCell::new(property_values(&snapshot)),
            snapshot: RefCell::new(snapshot),
            intents: RefCell::default(),
        });
        let id = export(&object)?;
        object.registration.replace(Some(id));
        Ok(object)
    }

    pub fn on_intent(&self, handler: IntentHandler) {
        self.intents.replace(Some(handler));
    }

    pub fn unregister(&self) {
        if let Some(id) = self.registration.take()
            && let Err(error) = self.connection.unregister_object(id)
        {
            warn(&format!("Headset1 was not unregistered: {error}"));
        }
    }

    fn property(&self, name: &str) -> glib::Variant {
        self.values
            .borrow()
            .get(name)
            .cloned()
            .unwrap_or_else(|| "".to_variant())
    }

    fn call(
        &self,
        method: &str,
        parameters: &glib::Variant,
        invocation: gio::DBusMethodInvocation,
    ) {
        match (method, parameters.get::<(String,)>()) {
            ("ActivatePreset", Some((id,))) => self.activate(id, invocation),
            _ => invocation.return_dbus_error(UNKNOWN_METHOD, method),
        }
    }

    fn activate(&self, id: String, invocation: gio::DBusMethodInvocation) {
        let picked = self.snapshot.borrow().picked_path();
        match picked {
            Ok(_) => {
                invocation.return_value(None);
                self.emit(Intent::ActivatePreset(PresetId::new(id)));
            }
            Err(error) => invocation.return_dbus_error(error_name(error), error_message(error)),
        }
    }

    fn set(&self, name: &str, value: &glib::Variant) -> bool {
        match (name, value.get::<bool>()) {
            ("AutomaticPresets", Some(on)) => {
                self.emit(Intent::SetAutomaticPresets(on));
                true
            }
            _ => false,
        }
    }

    fn emit(&self, intent: Intent) {
        let handler = self.intents.borrow().clone();
        if let Some(handler) = handler {
            handler(intent);
        }
    }

    fn announce(&self, changes: &PropertyMap) {
        let entries: HashMap<String, glib::Variant> = changes
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect();
        let body = (INTERFACE, entries, Vec::<String>::new()).to_variant();
        let sent = self.connection.emit_signal(
            None,
            &self.path,
            PROPERTIES,
            "PropertiesChanged",
            Some(&body),
        );
        if let Err(error) = sent {
            warn(&format!("PropertiesChanged was not sent: {error}"));
        }
    }
}

impl Drop for HeadsetObject {
    fn drop(&mut self) {
        self.unregister();
    }
}

impl SnapshotView for HeadsetObject {
    fn publish(&self, snapshot: &Rc<Snapshot>) {
        let values = property_values(snapshot);
        let changes = changed(&self.values.borrow(), &values);
        self.snapshot.replace(Rc::clone(snapshot));
        self.values.replace(values);
        if !changes.is_empty() {
            self.announce(&changes);
        }
    }
}

fn export(object: &Rc<HeadsetObject>) -> Result<gio::RegistrationId, glib::Error> {
    let node = gio::DBusNodeInfo::for_xml(CONTRACT)?;
    let interface = node.lookup_interface(INTERFACE).ok_or_else(|| {
        glib::Error::new(
            gio::IOErrorEnum::NotFound,
            "Headset1 is not in its contract",
        )
    })?;
    let (on_call, on_get, on_set) = (
        Rc::downgrade(object),
        Rc::downgrade(object),
        Rc::downgrade(object),
    );
    object
        .connection
        .register_object(&object.path, &interface)
        .method_call(move |_, _, _, _, method, parameters, invocation| {
            called(&on_call, method, &parameters, invocation);
        })
        .property(move |_, _, _, _, name| read(&on_get, name))
        .set_property(move |_, _, _, _, name, value| written(&on_set, name, &value))
        .build()
}

fn called(
    object: &Weak<HeadsetObject>,
    method: &str,
    parameters: &glib::Variant,
    invocation: gio::DBusMethodInvocation,
) {
    match object.upgrade() {
        Some(object) => object.call(method, parameters, invocation),
        None => invocation.return_dbus_error(UNKNOWN_METHOD, method),
    }
}

fn read(object: &Weak<HeadsetObject>, name: &str) -> glib::Variant {
    object
        .upgrade()
        .map_or_else(|| "".to_variant(), |object| object.property(name))
}

fn written(object: &Weak<HeadsetObject>, name: &str, value: &glib::Variant) -> bool {
    object
        .upgrade()
        .is_some_and(|object| object.set(name, value))
}
