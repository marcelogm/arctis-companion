use adw::prelude::*;
use arctis_core::companion::routing::Route;
use arctis_core::messages::device::Phase;
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::with;
use arctis_core::view::headset_page::{HeadsetPageModel, headset_page};
use arctis_core::view::status_page::StatusPageModel;
use arctis_ui::StatusView;
use gtk::glib;

use crate::support::{first, init, named};

fn status_model(phase: Phase) -> StatusPageModel {
    match headset_page(&with(phase, Route::Dongle)) {
        HeadsetPageModel::Status(_, model) => *model,
        HeadsetPageModel::Headset(_) => panic!("expected a status page"),
    }
}

fn rendered(model: &StatusPageModel, action: Option<&str>) -> StatusView {
    init();
    let view: StatusView = glib::Object::new();
    view.render(model, action);
    view
}

#[gtk::test]
fn no_headset_shows_its_title_and_no_button() {
    let model = status_model(Phase::Absent);
    let view = rendered(&model, None);
    let page = first::<adw::StatusPage>(&view);
    assert_eq!(page.title(), "No Headset Connected");
    assert_eq!(
        page.description().as_deref(),
        Some(english(&model.description).as_str())
    );
    assert_eq!(page.icon_name().as_deref(), Some(model.icon));
    assert!(!named::<gtk::Button>(&view, "status-button").get_visible());
}

#[gtk::test]
fn permission_required_shows_its_button_and_action() {
    let view = rendered(
        &status_model(Phase::PermissionDenied),
        Some("win.permission-instructions"),
    );
    let button = named::<gtk::Button>(&view, "status-button");
    assert!(button.get_visible());
    assert_eq!(button.label().as_deref(), Some("Show Instructions"));
    assert_eq!(
        button.action_name().as_deref(),
        Some("win.permission-instructions")
    );
}

#[gtk::test]
fn a_new_model_replaces_the_button() {
    let view = rendered(
        &status_model(Phase::PermissionDenied),
        Some("win.permission-instructions"),
    );
    view.render(&status_model(Phase::Absent), None);
    let button = named::<gtk::Button>(&view, "status-button");
    assert!(!button.get_visible());
    assert_eq!(button.action_name(), None);
}
