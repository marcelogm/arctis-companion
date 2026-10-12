use adw::prelude::*;
use arctis_ui::app_dialogs::{about_dialog, shortcuts_builder, shortcuts_dialog};

use crate::support::init;

#[gtk::test]
fn about_comes_from_the_metainfo() {
    init();
    let about = about_dialog();
    assert_eq!(about.application_name(), "Arctis Companion");
    assert_eq!(about.version(), env!("CARGO_PKG_VERSION"));
    assert_eq!(about.developer_name(), "marcelogm");
    assert_eq!(
        about.issue_url(),
        "https://github.com/marcelogm/arctis-companion/issues"
    );
}

#[gtk::test]
fn the_shortcuts_dialog_lists_three_shortcuts() {
    init();
    let _ = shortcuts_dialog();
    let builder = shortcuts_builder();
    let mut accelerators: Vec<String> = builder
        .objects()
        .into_iter()
        .filter_map(|object| object.downcast::<adw::ShortcutsItem>().ok())
        .map(|item| item.accelerator().to_string())
        .collect();
    accelerators.sort();
    assert_eq!(
        accelerators,
        ["<Control>comma", "<Control>question", "<Control>w"]
    );
}
