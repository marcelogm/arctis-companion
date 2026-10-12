use adw::prelude::*;
use arctis_ui::{PermissionDialog, RELOAD_COMMAND, UDEV_RULE};
use gtk::glib;

use crate::support::{init, named};

fn dialog() -> PermissionDialog {
    init();
    glib::Object::new()
}

fn clipboard_text(widget: &impl IsA<gtk::Widget>) -> String {
    widget
        .clipboard()
        .content()
        .expect("local clipboard content")
        .value(glib::Type::STRING)
        .expect("text on the clipboard")
        .get::<String>()
        .expect("a string")
}

#[test]
fn the_rule_is_the_shipped_udev_rule() {
    assert_eq!(
        UDEV_RULE,
        include_str!("../../../../data/udev/70-arctis-companion.rules").trim_end()
    );
    assert!(UDEV_RULE.contains("TAG+=\"uaccess\""));
}

#[gtk::test]
fn the_dialog_shows_the_rule_and_the_command() {
    let dialog = dialog();
    assert_eq!(dialog.title(), "Permission Required");
    assert_eq!(named::<gtk::Label>(&dialog, "udev-rule").label(), UDEV_RULE);
    assert_eq!(
        named::<gtk::Label>(&dialog, "reload-command").label(),
        RELOAD_COMMAND
    );
}

#[gtk::test]
fn copy_buttons_put_their_text_on_the_clipboard() {
    let dialog = dialog();
    named::<gtk::Button>(&dialog, "copy-rule").emit_clicked();
    assert_eq!(clipboard_text(&dialog), UDEV_RULE);
    named::<gtk::Button>(&dialog, "copy-command").emit_clicked();
    assert_eq!(clipboard_text(&dialog), RELOAD_COMMAND);
}
