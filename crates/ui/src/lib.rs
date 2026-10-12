#![forbid(unsafe_code)]

mod headset_status;
pub mod intents;
mod scale_row;
mod status_pages;
pub mod text;
pub mod widget_values;
mod window;

pub use headset_status::HeadsetStatus;
pub use scale_row::ScaleRow;
pub use status_pages::StatusView;
pub use window::Window;

pub fn register_resources() -> Result<(), gtk::glib::Error> {
    gtk::gio::resources_register_include!("arctis-ui.gresource")
}
