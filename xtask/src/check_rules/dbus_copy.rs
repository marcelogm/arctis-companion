use std::path::Path;

use super::snapshot::Snapshot;
use crate::report::Report;

const SOURCE: &str = "data/dbus/io.github.marcelogm.ArctisCompanion.Headset1.xml";
const NAME: &str = "io.github.marcelogm.ArctisCompanion.Headset1.xml";

pub fn check(snapshot: Snapshot, tracked: &[String]) -> Report {
    let source = snapshot.read(SOURCE);
    let diverging = tracked
        .iter()
        .filter(|file| is_extension_copy(file))
        .filter(|file| snapshot.read(file) != source);
    Report::errors(diverging.map(|file| format!("{file} must be byte-equal to {SOURCE}")))
}

fn is_extension_copy(file: &str) -> bool {
    file.starts_with("extension/") && Path::new(file).file_name().is_some_and(|name| name == NAME)
}
