use arctis_core::eq::reference;
use gettextrs::gettext;

const METAINFO: &str = "/io/github/marcelogm/ArctisCompanion/ui/metainfo.xml";
const SHORTCUTS: &str = "/io/github/marcelogm/ArctisCompanion/ui/shortcuts_dialog.ui";
const SHORTCUTS_ID: &str = "shortcuts_dialog";

#[must_use]
pub fn about_dialog() -> adw::AboutDialog {
    let about = adw::AboutDialog::from_appdata(METAINFO, None);
    about.add_acknowledgement_section(
        Some(&gettext(reference::CREDITS_TITLE)),
        &reference::CREDITS,
    );
    about.add_legal_section(
        "AutoEq",
        Some(reference::AUTOEQ_COPYRIGHT),
        gtk::License::MitX11,
        None,
    );
    about
}

#[must_use]
pub fn shortcuts_builder() -> gtk::Builder {
    gtk::Builder::from_resource(SHORTCUTS)
}

#[must_use]
pub fn shortcuts_dialog() -> adw::ShortcutsDialog {
    shortcuts_builder()
        .object::<adw::ShortcutsDialog>(SHORTCUTS_ID)
        .expect("the shortcuts resource defines its dialog")
}
