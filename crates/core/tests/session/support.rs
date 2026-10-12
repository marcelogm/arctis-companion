use std::fs;
use std::path::Path;

use arctis_core::eq::{Filter, FilterKind};
use arctis_core::settings::values::Sidetone;

pub fn golden(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name);
    let text = fs::read_to_string(path).expect("golden file is readable");
    text.split_whitespace()
        .map(|byte| u8::from_str_radix(byte, 16).expect("golden bytes are hex"))
        .collect()
}

pub fn filter(kind: FilterKind, hz: u16, half_db: i8, milli_q: u16) -> Filter {
    Filter::from_parts(kind, hz, half_db, milli_q).expect("filter in envelope")
}

pub fn sidetone(level: u8) -> Sidetone {
    Sidetone::try_from(level).expect("sidetone")
}
