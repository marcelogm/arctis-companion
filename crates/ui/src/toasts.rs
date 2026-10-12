use std::cell::Cell;

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

fn failure_toast(model: &ToastModel, sink: &IntentSink) -> adw::Toast {
    let toast = adw::Toast::builder()
        .title(resolve(&model.title))
        .button_label(resolve(&Text::plain("Retry")))
        .use_markup(false)
        .build();
    let retry = model.retry.clone();
    let sink = sink.clone();
    toast.connect_button_clicked(move |_| sink.emit(retry.clone()));
    toast
}
