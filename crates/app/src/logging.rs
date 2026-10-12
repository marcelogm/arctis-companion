use arctis_core::ports::StoreError;
use gtk::glib;

pub const LOG_DOMAIN: &str = "arctis-companion";

pub fn warn(message: &str) {
    glib::g_warning!(LOG_DOMAIN, "{message}");
}

pub fn report(result: Result<(), StoreError>) {
    if let Err(error) = result {
        warn(&error.to_string());
    }
}
