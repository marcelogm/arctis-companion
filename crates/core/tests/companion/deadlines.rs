use arctis_core::companion::action::Action;
use arctis_core::companion::deadlines::{Deadlines, earliest};

use crate::support::START as T;

#[test]
fn earliest_ignores_missing_deadlines() {
    assert_eq!(earliest([None, Some(T.after(5)), Some(T)]), Some(T));
    assert_eq!(earliest([None, None]), None);
}

#[test]
fn a_deadline_is_scheduled_once_until_it_fires() {
    let mut deadlines = Deadlines::default();
    assert_eq!(deadlines.schedule(Some(T)), Some(Action::WakeAt(T)));
    assert_eq!(deadlines.schedule(Some(T)), None);
    deadlines.woke();
    assert_eq!(deadlines.schedule(Some(T)), Some(Action::WakeAt(T)));
    assert_eq!(deadlines.schedule(None), None);
}
