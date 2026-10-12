use std::path::Path;

use arctis_app::application::APP_ID;
use gtk::gio;

pub fn memory_pair(dir: &Path) -> (gio::Settings, gio::Settings) {
    let source =
        gio::SettingsSchemaSource::from_directory(dir, None, false).expect("schema source");
    let schema = source.lookup(APP_ID, false).expect("app schema");
    let backend = gio::memory_settings_backend_new();
    (
        gio::Settings::new_full(&schema, Some(&backend), None),
        gio::Settings::new_full(&schema, Some(&backend), None),
    )
}
