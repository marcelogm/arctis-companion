use std::cell::Cell;
use std::rc::Rc;

use arctis_app::platform::background::Portal;
use arctis_app::platform::portal::AshpdPortal;

use crate::bus::{Bus, wait_until};
use crate::portal_service::{PortalService, Request};

fn ask(portal: &AshpdPortal) -> bool {
    let answer = Rc::new(Cell::new(None));
    let slot = Rc::clone(&answer);
    portal.request(Box::new(move |granted| slot.set(Some(granted))));
    wait_until("the portal answer", || answer.get().is_some());
    answer.get().unwrap_or(false)
}

pub fn asks_for_background_with_autostart(bus: &Bus) {
    let service = PortalService::start(bus);
    let portal = AshpdPortal;
    service.state.grant.set(true);
    assert!(ask(&portal));
    service.state.grant.set(false);
    assert!(!ask(&portal));
    let expected = Request {
        autostart: true,
        commandline: vec![
            "arctis-companion".to_owned(),
            "--gapplication-service".to_owned(),
        ],
    };
    assert_eq!(
        *service.state.requests.borrow(),
        [expected.clone(), expected]
    );
    portal.set_status("No headset".to_owned());
    wait_until("the status line", || {
        !service.state.statuses.borrow().is_empty()
    });
    assert_eq!(*service.state.statuses.borrow(), ["No headset"]);
    service.stop();
}

pub fn an_absent_portal_counts_as_denied(_bus: &Bus) {
    assert!(!ask(&AshpdPortal));
}
