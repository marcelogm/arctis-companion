use std::cell::Cell;

use arctis_core::l10n::msgid;
use arctis_core::view::headset_page::ToastModel;
use arctis_core::view::text::Text;

use crate::intents::IntentSink;
use crate::text::resolve;

#[derive(Debug, Default)]
pub struct ToastFeed {
    shown: Cell<Option<u32>>,
}

impl ToastFeed {
    #[must_use]
    pub fn toast_for_new_failure(
        &self,
        model: Option<&ToastModel>,
        sink: &IntentSink,
    ) -> Option<adw::Toast> {
        let model = model?;
        let previous = self.shown.replace(Some(model.serial));
        (previous != Some(model.serial)).then(|| failure_toast(model, sink))
    }
}

#[must_use]
pub fn plain_toast(title: &Text) -> adw::Toast {
    adw::Toast::builder()
        .title(resolve(title))
        .use_markup(false)
        .build()
}

#[must_use]
pub fn button_toast(title: &Text, label: &Text, on_click: impl Fn() + 'static) -> adw::Toast {
    let toast = plain_toast(title);
    toast.set_button_label(Some(&resolve(label)));
    toast.connect_button_clicked(move |_| on_click());
    toast
}

fn failure_toast(model: &ToastModel, sink: &IntentSink) -> adw::Toast {
    let retry = model.retry.clone();
    let sink = sink.clone();
    button_toast(&model.title, &Text::plain(msgid("Retry")), move || {
        sink.emit(retry.clone());
    })
}
