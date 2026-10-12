use std::fs;
use std::path::{Path, PathBuf};

use arctis_core::eq::{Curve, Filter, FilterKind};

pub fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

pub fn golden(name: &str) -> Vec<u8> {
    let text = fs::read_to_string(golden_dir().join(name)).expect("golden file is readable");
    text.split_whitespace()
        .map(|byte| u8::from_str_radix(byte, 16).expect("golden bytes are hex"))
        .collect()
}

const FACTORY_HZ: [u16; 10] = [32, 64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];

pub fn filter(kind: FilterKind, hz: u16, half_db: i8, milli_q: u16) -> Filter {
    Filter::from_parts(kind, hz, half_db, milli_q).expect("filter in envelope")
}

pub fn factory_flat() -> Curve {
    let filters = FACTORY_HZ
        .iter()
        .map(|&hz| filter(FilterKind::Peaking, hz, 0, 1410))
        .collect();
    Curve::new(filters).expect("ten filters")
}
