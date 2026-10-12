use adw::prelude::*;
use arctis_app::application::{self, APP_ID};
use arctis_app::threads::Threads;

#[test]
fn application_uses_the_app_id() {
    assert_eq!(
        application::build(Threads::system)
            .application_id()
            .as_deref(),
        Some(APP_ID)
    );
}
