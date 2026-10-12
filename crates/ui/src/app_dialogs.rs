const METAINFO: &str = "/io/github/marcelogm/ArctisCompanion/ui/metainfo.xml";
const SHORTCUTS: &str = "/io/github/marcelogm/ArctisCompanion/ui/shortcuts_dialog.ui";

#[must_use]
pub fn about_dialog() -> adw::AboutDialog {
    adw::AboutDialog::from_appdata(METAINFO, None)
}

#[must_use]
pub fn shortcuts_dialog() -> adw::ShortcutsDialog {
    gtk::Builder::from_resource(SHORTCUTS)
        .object::<adw::ShortcutsDialog>("shortcuts_dialog")
        .unwrap_or_default()
}
