#![forbid(unsafe_code)]

pub mod intents;
pub mod text;
pub mod widget_values;
mod window;

pub use window::Window;

pub fn register_resources() -> Result<(), gtk::glib::Error> {
    gtk::gio::resources_register_include!("arctis-ui.gresource")
}
