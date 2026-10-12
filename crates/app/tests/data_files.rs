use std::path::{Path, PathBuf};

use arctis_app::application::APP_ID;

const SUFFIXES: [&str; 4] = [".desktop.in", ".metainfo.xml.in", ".gschema.xml", ".service.in"];

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

#[test]
fn every_data_file_is_named_after_the_app_id() {
    for suffix in SUFFIXES {
        let file = data_dir().join(format!("{APP_ID}{suffix}"));
        assert!(file.is_file(), "{}", file.display());
    }
}
