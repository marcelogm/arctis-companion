use gtk::prelude::*;

pub const FALLBACK_ICON: &str = "application-x-executable-symbolic";

#[must_use]
pub fn icon_name(theme: &gtk::IconTheme, identifier: &str) -> String {
    if theme.has_icon(identifier) {
        identifier.to_owned()
    } else {
        FALLBACK_ICON.to_owned()
    }
}

#[must_use]
pub fn display_icon_name(widget: &impl IsA<gtk::Widget>, identifier: &str) -> String {
    icon_name(&gtk::IconTheme::for_display(&widget.display()), identifier)
}
