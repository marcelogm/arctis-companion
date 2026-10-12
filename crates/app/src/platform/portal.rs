use arctis_core::view::text::Text;
use arctis_ui::text::resolve;
use ashpd::desktop::background::{Background, BackgroundProxy, SetStatusOptions};
use gtk::glib;

use super::background::Portal;
use crate::logging::{LOG_DOMAIN, warn};

const COMMAND: [&str; 2] = ["arctis-companion", "--gapplication-service"];
const REASON: &str =
    "Keep battery warnings, battery history and app presets working with the window closed";

pub struct AshpdPortal;

impl Portal for AshpdPortal {
    fn request(&self, autostart: bool, answer: Box<dyn FnOnce(bool)>) {
        glib::spawn_future_local(async move {
            answer(request_background(autostart).await);
        });
    }

    fn set_status(&self, message: String) {
        glib::spawn_future_local(report_status(message));
    }
}

async fn request_background(autostart: bool) -> bool {
    let reason = resolve(&Text::plain(REASON));
    let request = Background::request()
        .reason(reason.as_str())
        .auto_start(autostart)
        .command(COMMAND)
        .dbus_activatable(false)
        .send()
        .await;
    match request.and_then(|request| request.response()) {
        Ok(answer) => answer.run_in_background(),
        Err(error) => {
            warn(&format!("background request failed: {error}"));
            false
        }
    }
}

async fn report_status(message: String) {
    if let Err(error) = send_status(&message).await {
        glib::g_debug!(LOG_DOMAIN, "background status not sent: {error}");
    }
}

async fn send_status(message: &str) -> ashpd::Result<()> {
    let proxy = BackgroundProxy::new().await?;
    proxy
        .set_status(SetStatusOptions::default().set_message(message))
        .await
}
