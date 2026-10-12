use std::path::{Path, PathBuf};

use gettextrs::{LocaleCategory, bind_textdomain_codeset, bindtextdomain, setlocale, textdomain};

use crate::logging::warn;

pub const GETTEXT_PACKAGE: &str = "arctis-companion";

pub fn init() {
    if setlocale(LocaleCategory::LcAll, "").is_none() {
        warn("the locale is not supported; staying in English");
    }
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| locale_dir_beside(&exe))
    {
        report(bindtextdomain(GETTEXT_PACKAGE, dir).map(drop));
    }
    report(bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8").map(drop));
    report(textdomain(GETTEXT_PACKAGE).map(drop));
}

#[must_use]
pub fn locale_dir_beside(executable: &Path) -> Option<PathBuf> {
    let prefix = executable.parent()?.parent()?;
    Some(prefix.join("share").join("locale"))
}

fn report(result: std::io::Result<()>) {
    if let Err(error) = result {
        warn(&format!("translations unavailable: {error}"));
    }
}
