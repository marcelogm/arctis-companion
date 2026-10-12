#![forbid(unsafe_code)]

mod add_rule_dialog;
pub mod app_dialogs;
pub mod app_icons;
mod app_presets_page;
mod app_rule_row;
mod battery_chart;
mod battery_page;
mod eq_band_row;
mod eq_editor_page;
mod eq_graph;
mod eq_handle;
mod equalizer_group;
mod headset_page;
mod headset_rows;
mod headset_status;
pub mod intents;
mod permission_dialog;
mod preferences_dialog;
pub mod preset_files;
mod presets_page;
mod scale_row;
mod services;
mod status_pages;
pub mod text;
mod toasts;
pub mod widget_values;
mod window;

pub use add_rule_dialog::AddRuleDialog;
pub use app_presets_page::AppPresetsPage;
pub use app_rule_row::AppRuleRow;
pub use battery_chart::BatteryChart;
pub use battery_page::BatteryPage;
pub use eq_band_row::EqBandRow;
pub use eq_editor_page::EqEditorPage;
pub use eq_graph::{EqGraph, EqLegend, GraphAction};
pub use eq_handle::EqHandle;
pub use equalizer_group::EqualizerGroup;
pub use headset_page::HeadsetPage;
pub use headset_status::HeadsetStatus;
pub use permission_dialog::{PermissionDialog, RELOAD_COMMAND, UDEV_RULE};
pub use preferences_dialog::{PreferencesDialog, SHELL_EXTENSIONS_URI};
pub use presets_page::PresetsPage;
pub use scale_row::ScaleRow;
pub use services::{Services, details_dialog};
pub use status_pages::StatusView;
pub use toasts::ToastFeed;
pub use window::Window;

pub fn register_resources() -> Result<(), gtk::glib::Error> {
    gtk::gio::resources_register_include!("arctis-ui.gresource")
}
