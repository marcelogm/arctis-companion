use arctis_core::companion::alerts::{Alert, AlertId};
use arctis_ui::text::resolve;
use gtk::gio;
use gtk::gio::prelude::*;

use crate::ports::Notifier;

pub const SHOW_BATTERY_ACTION: &str = "app.show-battery";

pub struct GioNotifier {
    application: gio::Application,
}

impl GioNotifier {
    #[must_use]
    pub const fn new(application: gio::Application) -> Self {
        Self { application }
    }
}

impl Notifier for GioNotifier {
    fn notify(&self, alert: &Alert) {
        self.application
            .send_notification(Some(notification_id(alert.id)), &notification_for(alert));
    }

    fn withdraw(&self, id: AlertId) {
        self.application.withdraw_notification(notification_id(id));
    }
}

#[must_use]
pub const fn notification_id(id: AlertId) -> &'static str {
    match id {
        AlertId::LowBattery => "low-battery",
        AlertId::CriticalBattery => "critical-battery",
    }
}

#[must_use]
pub fn notification_for(alert: &Alert) -> gio::Notification {
    let notification = gio::Notification::new(&resolve(&alert.title));
    notification.set_body(Some(&resolve(&alert.body)));
    notification.set_default_action(SHOW_BATTERY_ACTION);
    notification
}
