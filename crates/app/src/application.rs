use adw::prelude::*;
use arctis_ui::Window;
use gtk::glib;

pub const APP_ID: &str = "io.github.marcelogm.ArctisCompanion";
const LOG_DOMAIN: &str = "arctis-companion";

#[must_use]
pub fn build() -> adw::Application {
    let application = adw::Application::builder().application_id(APP_ID).build();
    application.connect_activate(present_window);
    application
}

#[must_use]
pub fn run() -> glib::ExitCode {
    match arctis_ui::register_resources() {
        Ok(()) => build().run(),
        Err(error) => {
            glib::g_critical!(LOG_DOMAIN, "bundled resources failed to register: {error}");
            glib::ExitCode::FAILURE
        }
    }
}

fn present_window(application: &adw::Application) {
    match application.active_window() {
        Some(window) => window.present(),
        None => Window::new(application).present(),
    }
}
