use arctis_app::platform::notifications::GioNotifier;
use arctis_core::protocol::status::Charging;
use gtk::gio;
use gtk::gio::prelude::*;

use crate::bus::{Bus, wait_until};
use crate::fixtures::{quiet_bridge, status};
use crate::notification_service::{NotificationService, Shown};

const TEST_APP_ID: &str = "io.github.marcelogm.ArctisCompanion.NotificationTest";

fn registered_application() -> gio::Application {
    let application = gio::Application::new(Some(TEST_APP_ID), gio::ApplicationFlags::empty());
    application
        .register(None::<&gio::Cancellable>)
        .expect("registered");
    application
}

fn added(id: &str, title: &str) -> Shown {
    Shown::Added {
        id: id.to_owned(),
        title: title.to_owned(),
        default_action: "app.show-battery".to_owned(),
    }
}

fn removed(id: &str) -> Shown {
    Shown::Removed(id.to_owned())
}

fn shown_after(service: &NotificationService, what: &str, count: usize) -> Vec<Shown> {
    wait_until(what, || service.shown().len() == count);
    service.shown()
}

pub fn battery_alerts_are_notified_then_withdrawn(bus: &Bus) {
    let service = NotificationService::start(bus);
    let bridge = quiet_bridge(Box::new(GioNotifier::new(registered_application())));
    bridge.submit(status(14, Charging::Discharging));
    let shown = shown_after(&service, "the low-battery notification", 1);
    assert_eq!(shown, [added("low-battery", "Headset Battery Low")]);
    bridge.submit(status(4, Charging::Discharging));
    let shown = shown_after(&service, "the critical notification", 3);
    assert_eq!(
        shown[1..],
        [
            removed("low-battery"),
            added("critical-battery", "Headset Battery Critical")
        ]
    );
    bridge.submit(status(4, Charging::Charging));
    let shown = shown_after(&service, "the withdrawal", 4);
    assert_eq!(shown[3], removed("critical-battery"));
    service.stop();
}
