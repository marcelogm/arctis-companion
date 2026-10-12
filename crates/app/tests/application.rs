use adw::prelude::*;
use arctis_app::application::{self, APP_ID};

#[test]
fn application_uses_the_app_id() {
    assert_eq!(
        application::build().application_id().as_deref(),
        Some(APP_ID)
    );
}
