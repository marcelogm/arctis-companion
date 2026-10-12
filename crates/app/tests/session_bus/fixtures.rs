use std::fs;
use std::path::Path;

pub const PATH: &str = "/io/github/marcelogm/ArctisCompanion";
pub const INTERFACE: &str = "io.github.marcelogm.ArctisCompanion.Headset1";

pub fn contract_xml() -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/dbus")
        .join(format!("{INTERFACE}.xml"));
    fs::read_to_string(file).expect("contract file")
}
