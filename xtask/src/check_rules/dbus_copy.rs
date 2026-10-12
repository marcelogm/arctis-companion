use std::fs;
use std::path::Path;

use crate::report::Report;

const SOURCE: &str = "data/dbus/io.github.marcelogm.ArctisCompanion.Headset1.xml";
const NAME: &str = "io.github.marcelogm.ArctisCompanion.Headset1.xml";

pub fn check(root: &Path, files: &[String]) -> Report {
    let source = fs::read(root.join(SOURCE)).ok();
    let diverging = files
        .iter()
        .filter(|file| is_extension_copy(file))
        .filter(|file| fs::read(root.join(file.as_str())).ok() != source);
    Report::errors(diverging.map(|file| format!("{file} must be byte-equal to {SOURCE}")))
}

fn is_extension_copy(file: &str) -> bool {
    file.starts_with("extension/") && Path::new(file).file_name().is_some_and(|name| name == NAME)
}
