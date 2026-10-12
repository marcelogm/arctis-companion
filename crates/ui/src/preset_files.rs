use std::rc::Rc;

use arctis_core::l10n::msgid;
use arctis_core::view::text::Text;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::text::resolve;

const PATTERNS: [&str; 2] = ["*.txt", "*.json"];

pub struct PickedText {
    pub name: String,
    pub text: String,
}

pub type Render = Box<dyn FnOnce(&str) -> Option<String>>;
pub type Done = Box<dyn FnOnce(bool)>;
pub type Picked = Box<dyn FnOnce(PickedText)>;

pub struct Export {
    pub suggested_name: String,
    pub render: Render,
    pub done: Done,
}

pub trait PresetFiles {
    fn open(&self, parent: &gtk::Widget, done: Picked);
    fn save(&self, parent: &gtk::Widget, export: Export);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PortalFiles;

impl PresetFiles for PortalFiles {
    fn open(&self, parent: &gtk::Widget, done: Picked) {
        let dialog = preset_dialog(&Text::plain(msgid("Import Preset")));
        let window = parent.root().and_downcast::<gtk::Window>();
        glib::spawn_future_local(deliver(dialog, window, done));
    }

    fn save(&self, parent: &gtk::Widget, export: Export) {
        let dialog = preset_dialog(&Text::plain(msgid("Export Preset")));
        dialog.set_initial_name(Some(&export.suggested_name));
        let window = parent.root().and_downcast::<gtk::Window>();
        glib::spawn_future_local(async move {
            write(&dialog, window.as_ref(), export).await;
        });
    }
}

#[must_use]
pub fn portal_files() -> Rc<dyn PresetFiles> {
    Rc::new(PortalFiles)
}

fn preset_dialog(title: &Text) -> gtk::FileDialog {
    let filter = gtk::FileFilter::new();
    filter.set_name(Some(&resolve(&Text::plain(msgid("Presets")))));
    for pattern in PATTERNS {
        filter.add_pattern(pattern);
    }
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);
    gtk::FileDialog::builder()
        .title(resolve(title))
        .modal(true)
        .filters(&filters)
        .default_filter(&filter)
        .build()
}

async fn deliver(dialog: gtk::FileDialog, window: Option<gtk::Window>, done: Picked) {
    if let Some(picked) = pick(&dialog, window.as_ref()).await {
        done(picked);
    }
}

async fn pick(dialog: &gtk::FileDialog, window: Option<&gtk::Window>) -> Option<PickedText> {
    let file = dialog.open_future(window).await.ok()?;
    let (bytes, _) = file.load_contents_future().await.ok()?;
    Some(PickedText {
        name: file_name(&file),
        text: String::from_utf8_lossy(&bytes).into_owned(),
    })
}

async fn write(dialog: &gtk::FileDialog, window: Option<&gtk::Window>, export: Export) {
    let Ok(file) = dialog.save_future(window).await else {
        return;
    };
    let written = match (export.render)(&file_name(&file)) {
        Some(text) => store(&file, text).await,
        None => false,
    };
    (export.done)(written);
}

async fn store(file: &gio::File, text: String) -> bool {
    file.replace_contents_future(
        text.into_bytes(),
        None,
        false,
        gio::FileCreateFlags::REPLACE_DESTINATION,
    )
    .await
    .is_ok()
}

fn file_name(file: &gio::File) -> String {
    file.basename()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}
