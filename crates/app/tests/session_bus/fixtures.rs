use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use arctis_core::intent::Intent;

pub const PATH: &str = "/io/github/marcelogm/ArctisCompanion";
pub const INTERFACE: &str = "io.github.marcelogm.ArctisCompanion.Headset1";

pub fn contract_xml() -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/dbus")
        .join(format!("{INTERFACE}.xml"));
    fs::read_to_string(file).expect("contract file")
}

pub type Intents = Rc<RefCell<Vec<Intent>>>;

pub fn recorder(log: &Intents) -> impl Fn(Intent) + 'static {
    let log = Rc::clone(log);
    move |intent| log.borrow_mut().push(intent)
}
