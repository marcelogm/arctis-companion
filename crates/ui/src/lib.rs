#![forbid(unsafe_code)]

pub mod app_dialogs;
pub mod app_icons;
mod equalizer_group;
mod headset_page;
mod headset_rows;
mod headset_status;
pub mod intents;
mod permission_dialog;
mod preferences_dialog;
mod scale_row;
mod status_pages;
pub mod text;
mod toasts;
pub mod widget_values;
mod window;

pub use equalizer_group::EqualizerGroup;
pub use headset_page::HeadsetPage;
pub use headset_status::HeadsetStatus;
pub use permission_dialog::{PermissionDialog, RELOAD_COMMAND, UDEV_RULE};
pub use preferences_dialog::{PreferencesDialog, SHELL_EXTENSIONS_URI};
pub use scale_row::ScaleRow;
pub use status_pages::StatusView;
pub use toasts::ToastFeed;
pub use window::Window;

pub fn register_resources() -> Result<(), gtk::glib::Error> {
    gtk::gio::resources_register_include!("arctis-ui.gresource")
}
