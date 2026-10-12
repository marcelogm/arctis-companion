use adw::prelude::*;
use arctis_ui::Window;

pub const APP_ID: &str = "io.github.marcelogm.ArctisCompanion";

#[must_use]
pub fn build() -> adw::Application {
    let application = adw::Application::builder().application_id(APP_ID).build();
    application.connect_startup(|_| register_resources());
    application.connect_activate(present_window);
    application
}

fn register_resources() {
    arctis_ui::register_resources().expect("bundled resources are valid");
}

fn present_window(application: &adw::Application) {
    match application.active_window() {
        Some(window) => window.present(),
        None => Window::new(application).present(),
    }
}
