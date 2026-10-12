use std::rc::Rc;

use arctis_app::dbus::headset_object::HeadsetObject;
use arctis_core::testkit::snapshots::snapshot;

use crate::bus::{Bus, Target, call};
use crate::fixtures::{INTERFACE, PATH, contract_xml};
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
