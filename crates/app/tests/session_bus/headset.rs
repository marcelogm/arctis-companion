use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use arctis_app::dbus::headset_object::HeadsetObject;
use arctis_app::ports::SnapshotView;
use arctis_core::companion::Snapshot;
use arctis_core::companion::adoption::AdoptionStatus;
use arctis_core::companion::alerts::BatteryWarning;
use arctis_core::companion::routing::Route;
use arctis_core::intent::Intent;
use arctis_core::messages::device::Phase;
use arctis_core::testkit::snapshots::{id, percent, snapshot, with};
use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib;

use crate::bus::{Bus, Target, call, property, settle, wait_until};
use crate::fixtures::{INTERFACE, Intents, PATH, contract_xml, recorder};
use crate::introspection::canonical_members;

pub fn matches_the_contract(bus: &Bus) {
    let session = bus.session();
    let object = HeadsetObject::register(&session, PATH, Rc::new(snapshot())).expect("exported");
    let name = session.unique_name().expect("unique name");
    let target = Target {
        name: &name,
        path: PATH,
        interface: "org.freedesktop.DBus.Introspectable",
    };
    let reply = call(&bus.client(), &target, "Introspect", None).expect("introspection");
    let (live,) = reply.get::<(String,)>().expect("(s)");
    assert_eq!(
        canonical_members(&live, INTERFACE),
        canonical_members(&contract_xml(), INTERFACE)
    );
    object.unregister();
}

fn low_battery() -> Snapshot {
    let mut low = snapshot();
    if let Some(reading) = low.battery.reading.as_mut() {
        reading.percent = percent(14);
    }
    low.battery.warning = BatteryWarning::Low;
    low
}

type Signals = Rc<RefCell<Vec<glib::Variant>>>;

fn listen(client: &gio::DBusConnection, sender: &str) -> (gio::SignalSubscription, Signals) {
    let signals = Signals::default();
    let log = Rc::clone(&signals);
    let subscription = client.subscribe_to_signal(
        Some(sender),
        Some("org.freedesktop.DBus.Properties"),
        Some("PropertiesChanged"),
        Some(PATH),
        None,
        gio::DBusSignalFlags::NONE,
        move |signal| log.borrow_mut().push(signal.parameters.clone()),
    );
    settle();
    (subscription, signals)
}

pub fn follows_the_snapshot(bus: &Bus) {
    let session = bus.session();
    let name = session.unique_name().expect("unique name");
    let object = HeadsetObject::register(&session, PATH, Rc::new(snapshot())).expect("exported");
    let client = bus.client();
    let (_subscription, signals) = listen(&client, &name);
    object.publish(&Rc::new(low_battery()));
    wait_until("PropertiesChanged", || !signals.borrow().is_empty());
    let first = signals.borrow()[0].clone();
    let (interface, changed, invalidated) = first
        .get::<(String, HashMap<String, glib::Variant>, Vec<String>)>()
        .expect("(sa{sv}as)");
    assert_eq!(interface, INTERFACE);
    let mut names: Vec<&str> = changed.keys().map(String::as_str).collect();
    names.sort_unstable();
    assert_eq!(names, ["BatteryLevel", "BatteryWarning"]);
    assert!(invalidated.is_empty());
    assert_eq!(
        property(&client, &name, "BatteryLevel").get::<i32>(),
        Some(14)
    );
    object.publish(&Rc::new(low_battery()));
    settle();
    assert_eq!(
        signals.borrow().len(),
        1,
        "an unchanged snapshot sends nothing"
    );
    object.unregister();
}

fn activate(bus: &Bus, state: Snapshot) -> (Result<glib::Variant, glib::Error>, Vec<Intent>) {
    let session = bus.session();
    let name = session.unique_name().expect("unique name");
    let object = HeadsetObject::register(&session, PATH, Rc::new(state)).expect("exported");
    let intents = Intents::default();
    object.on_intent(recorder(&intents));
    let target = Target {
        name: &name,
        path: PATH,
        interface: INTERFACE,
    };
    let result = call(
        &bus.client(),
        &target,
        "ActivatePreset",
        Some(&("bass-boost",).to_variant()),
    );
    object.unregister();
    let intents = intents.borrow().clone();
    (result, intents)
}

fn refused_with(result: &Result<glib::Variant, glib::Error>, error: &str) -> bool {
    result
        .as_ref()
        .is_err_and(|refusal| refusal.message().contains(error))
}

pub fn activate_preset_follows_the_picked_path(bus: &Bus) {
    let (result, intents) = activate(bus, with(Phase::Absent, Route::None));
    assert!(refused_with(&result, "Error.NoRoute"), "{result:?}");
    assert!(intents.is_empty());
    let mut never_read = with(Phase::HeadsetOff, Route::None);
    never_read.dongle.adoption = AdoptionStatus::NeverAdopted;
    let (result, intents) = activate(bus, never_read);
    assert!(refused_with(&result, "Error.NotReady"), "{result:?}");
    assert!(intents.is_empty());
    let (result, intents) = activate(bus, snapshot());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(intents, [Intent::ActivatePreset(id("bass-boost"))]);
}

pub fn automatic_presets_is_writable(bus: &Bus) {
    let session = bus.session();
    let name = session.unique_name().expect("unique name");
    let object = HeadsetObject::register(&session, PATH, Rc::new(snapshot())).expect("exported");
    let intents = Intents::default();
    object.on_intent(recorder(&intents));
    let target = Target {
        name: &name,
        path: PATH,
        interface: "org.freedesktop.DBus.Properties",
    };
    let value = (INTERFACE, "AutomaticPresets", false.to_variant()).to_variant();
    call(&bus.client(), &target, "Set", Some(&value)).expect("set");
    assert_eq!(*intents.borrow(), [Intent::SetAutomaticPresets(false)]);
    object.unregister();
}
