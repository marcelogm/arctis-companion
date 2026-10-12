#[path = "common/schema.rs"]
mod schema;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use arctis_app::platform::background::{BackgroundController, BackgroundView, Hold, Portal};
use arctis_app::ports::SnapshotView;
use arctis_core::companion::routing::Route;
use arctis_core::messages::device::Phase;
use arctis_core::testkit::snapshots::{snapshot, with};
use arctis_core::view::preferences::Background;
use gtk::gio;
use gtk::gio::prelude::*;
use tempfile::TempDir;

use schema::{compiled_dir, memory_pair};

type Answer = Box<dyn FnOnce(bool)>;

#[derive(Default)]
struct FakePortal {
    requests: RefCell<Vec<bool>>,
    pending: RefCell<Vec<Answer>>,
    statuses: RefCell<Vec<String>>,
}

impl FakePortal {
    fn answer(&self, granted: bool) {
        let answer = self.pending.borrow_mut().remove(0);
        answer(granted);
    }
}

impl Portal for FakePortal {
    fn request(&self, autostart: bool, answer: Answer) {
        self.requests.borrow_mut().push(autostart);
        self.pending.borrow_mut().push(answer);
    }

    fn set_status(&self, message: String) {
        self.statuses.borrow_mut().push(message);
    }
}

#[derive(Default)]
struct FakeHold(Cell<bool>);

impl Hold for FakeHold {
    fn hold(&self) {
        self.0.set(true);
    }

    fn release(&self) {
        self.0.set(false);
    }
}

#[derive(Default)]
struct FakeView(RefCell<Vec<Background>>);

impl BackgroundView for FakeView {
    fn show_background(&self, state: Background) {
        self.0.borrow_mut().push(state);
    }
}

struct Setup {
    _dir: TempDir,
    settings: gio::Settings,
    portal: Rc<FakePortal>,
    hold: Rc<FakeHold>,
    view: Rc<FakeView>,
    controller: Rc<BackgroundController>,
}

fn setup(run_in_background: bool, asked_before: bool) -> Setup {
    let dir = compiled_dir();
    let (settings, _) = memory_pair(dir.path());
    settings
        .set_boolean("run-in-background", run_in_background)
        .expect("run in background");
    settings
        .set_boolean("background-requested", asked_before)
        .expect("requested");
    let (portal, hold, view) = (
        Rc::<FakePortal>::default(),
        Rc::<FakeHold>::default(),
        Rc::<FakeView>::default(),
    );
    let controller =
        BackgroundController::new(settings.clone(), portal.clone(), hold.clone(), view.clone());
    Setup {
        _dir: dir,
        settings,
        portal,
        hold,
        view,
        controller,
    }
}

#[test]
fn the_first_run_holds_and_asks_for_autostart() {
    let s = setup(true, false);
    s.controller.start();
    assert!(s.hold.0.get());
    assert_eq!(*s.view.0.borrow(), [Background::On]);
    assert_eq!(*s.portal.requests.borrow(), [true]);
}

#[test]
fn a_grant_is_remembered_and_not_asked_again() {
    let s = setup(true, false);
    s.controller.start();
    s.portal.answer(true);
    assert!(s.settings.boolean("background-requested"));
    assert!(s.hold.0.get());
    let again = BackgroundController::new(
        s.settings.clone(),
        s.portal.clone(),
        s.hold.clone(),
        s.view.clone(),
    );
    again.start();
    assert_eq!(s.portal.requests.borrow().len(), 1);
}

#[test]
fn a_denial_turns_run_in_background_off() {
    let s = setup(true, false);
    s.controller.start();
    s.portal.answer(false);
    assert!(!s.settings.boolean("run-in-background"));
    assert!(!s.hold.0.get());
    assert_eq!(s.view.0.borrow().last(), Some(&Background::Denied));
    assert!(!s.controller.is_enabled());
}

#[test]
fn switched_off_at_start_shows_off_without_holding() {
    let s = setup(false, true);
    s.controller.start();
    assert!(!s.hold.0.get());
    assert_eq!(*s.view.0.borrow(), [Background::Off]);
    assert!(s.portal.requests.borrow().is_empty());
}

#[test]
fn turning_it_on_asks_again() {
    let s = setup(false, true);
    s.controller.start();
    s.controller.set_enabled(true);
    assert!(s.settings.boolean("run-in-background"));
    assert!(s.hold.0.get());
    assert_eq!(s.view.0.borrow().last(), Some(&Background::On));
    assert_eq!(*s.portal.requests.borrow(), [true]);
}

#[test]
fn turning_it_off_releases_without_asking() {
    let s = setup(true, true);
    s.controller.start();
    s.controller.set_enabled(false);
    assert!(!s.settings.boolean("run-in-background"));
    assert!(!s.hold.0.get());
    assert_eq!(s.view.0.borrow().last(), Some(&Background::Off));
    assert!(s.portal.requests.borrow().is_empty());
}

#[test]
fn the_status_line_follows_the_snapshot_when_it_changes() {
    let s = setup(true, true);
    s.controller.publish(&Rc::new(snapshot()));
    s.controller.publish(&Rc::new(snapshot()));
    let mut absent = with(Phase::Absent, Route::None);
    absent.battery.reading = None;
    s.controller.publish(&Rc::new(absent));
    assert_eq!(
        *s.portal.statuses.borrow(),
        ["Headset connected, 73 %", "No headset"]
    );
}
