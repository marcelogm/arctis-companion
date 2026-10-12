use std::rc::Rc;

use adw::prelude::*;
use arctis_core::l10n::msgid;
use arctis_core::view::text::Text;

use crate::intents::IntentSink;
use crate::preset_files::PresetFiles;
use crate::text::resolve;
use crate::toasts::{button_toast, plain_toast};

const CLOSE: &str = "close";

#[derive(Clone)]
pub struct Services {
    pub sink: IntentSink,
    pub toasts: adw::ToastOverlay,
    pub files: Rc<dyn PresetFiles>,
}

impl Services {
    pub fn toast(&self, title: &Text) {
        self.toasts.add_toast(plain_toast(title));
    }

    pub fn undo_toast(&self, title: &Text, undo: impl Fn() + 'static) -> adw::Toast {
        let toast = button_toast(title, &Text::plain(msgid("Undo")), undo);
        self.toasts.add_toast(toast.clone());
        toast
    }

    pub fn urgent_undo_toast(&self, title: &Text, undo: impl Fn() + 'static) {
        let toast = button_toast(title, &Text::plain(msgid("Undo")), undo);
        toast.set_priority(adw::ToastPriority::High);
        self.toasts.add_toast(toast);
    }

    pub fn toast_with_details(&self, title: &Text, details: Vec<Text>) {
        let overlay = self.toasts.clone();
        let toast = button_toast(title, &Text::plain(msgid("Details")), move || {
            details_dialog(&details).present(Some(&overlay));
        });
        self.toasts.add_toast(toast);
    }
}

#[must_use]
pub fn details_dialog(lines: &[Text]) -> adw::AlertDialog {
    let body: Vec<String> = lines.iter().map(resolve).collect();
    let dialog = adw::AlertDialog::builder()
        .heading(resolve(&Text::plain(msgid("Changed to Fit the Headset"))))
        .body(body.join("\n"))
        .body_use_markup(false)
        .close_response(CLOSE)
        .build();
    dialog.add_response(CLOSE, &resolve(&Text::plain(msgid("_Close"))));
    dialog
}
