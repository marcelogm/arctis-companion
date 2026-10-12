use arctis_core::companion::adoption::AdoptionStatus;
use arctis_core::companion::app_settings::AppRules;
use arctis_core::companion::catalog::Catalog;
use arctis_core::companion::per_path::PATHS;
use arctis_core::companion::routing::Route;
use arctis_core::companion::selection::{
    ActivateError, ActivePreset, Candidate, ManualPick, PathSelection, PresetReason, RuledSet,
    Selection, candidates, picked_path,
};
use arctis_core::companion::timing::{RULE_ENTER, RULE_LEAVE};
use arctis_core::messages::audio::SoundPath;
use arctis_core::time::Millis;

use crate::support::{id, stream};

const T: Millis = Millis(1_000_000);

fn candidate(key: &str, preset: &str) -> Candidate {
    Candidate {
        key: key.to_owned(),
        name: key.to_owned(),
        preset: id(preset),
    }
}

fn joined(running: &[Candidate], at: Millis) -> RuledSet {
    let mut set = RuledSet::default();
    set.observe(running, at);
    let _ = set.advance(at.after(RULE_ENTER));
    set
}

fn newest(set: &RuledSet) -> Option<&str> {
    set.newest().map(|member| member.candidate.key.as_str())
}

#[test]
fn a_ruled_app_joins_after_one_second_of_playback() {
    let mut set = RuledSet::default();
    set.observe(&[candidate("firefox", "footsteps")], T);
    assert_eq!(set.next_deadline(), Some(T.after(RULE_ENTER)));
    assert!(!set.advance(T.after(RULE_ENTER - 1)));
    assert!(set.advance(T.after(RULE_ENTER)));
    assert_eq!(newest(&set), Some("firefox"));
    assert_eq!(set.next_deadline(), None);
}

#[test]
fn an_app_that_stops_within_one_second_never_joins() {
    let mut set = RuledSet::default();
    set.observe(&[candidate("firefox", "footsteps")], T);
    set.observe(&[], T.after(500));
    assert!(!set.advance(T.after(RULE_ENTER)));
    assert!(set.is_empty());
}

#[test]
fn a_member_leaves_after_ten_seconds_of_silence() {
    let mut set = joined(&[candidate("firefox", "footsteps")], T);
    let silent = T.after(2_000);
    set.observe(&[], silent);
    assert_eq!(set.next_deadline(), Some(silent.after(RULE_LEAVE)));
    assert!(!set.advance(silent.after(RULE_LEAVE - 1)));
    assert!(set.advance(silent.after(RULE_LEAVE)));
    assert!(set.is_empty());
}

#[test]
fn silence_seen_again_keeps_the_first_leave_deadline() {
    let mut set = joined(&[candidate("firefox", "footsteps")], T);
    let silent = T.after(2_000);
    set.observe(&[], silent);
    set.observe(&[], silent.after(5_000));
    assert_eq!(set.next_deadline(), Some(silent.after(RULE_LEAVE)));
    assert!(set.advance(silent.after(RULE_LEAVE)));
    assert_eq!(set.next_deadline(), None);
}

#[test]
fn clearing_reports_whether_members_were_dropped() {
    let mut set = joined(&[candidate("firefox", "footsteps")], T);
    assert!(set.clear());
    assert!(set.is_empty());
    assert!(!set.clear());
}

#[test]
fn pausing_and_resuming_within_ten_seconds_changes_nothing() {
    let firefox = [candidate("firefox", "footsteps")];
    let mut set = joined(&firefox, T);
    set.observe(&[], T.after(2_000));
    set.observe(&firefox, T.after(7_000));
    assert!(!set.advance(T.after(2_000 + RULE_LEAVE)));
    assert_eq!(newest(&set), Some("firefox"));
}

#[test]
fn the_newest_member_is_the_last_to_join() {
    let firefox = candidate("firefox", "footsteps");
    let spotify = candidate("spotify", "podcast");
    let mut set = joined(std::slice::from_ref(&firefox), T);
    set.observe(&[firefox.clone(), spotify.clone()], T.after(5_000));
    let _ = set.advance(T.after(6_000));
    assert_eq!(newest(&set), Some("spotify"));
    set.observe(&[firefox], T.after(7_000));
    let _ = set.advance(T.after(17_000));
    assert_eq!(newest(&set), Some("firefox"));
}

#[test]
fn a_member_follows_a_changed_rule() {
    let mut set = joined(&[candidate("firefox", "footsteps")], T);
    set.observe(&[candidate("firefox", "podcast")], T.after(3_000));
    assert_eq!(
        set.newest().map(|m| m.candidate.preset.clone()),
        Some(id("podcast"))
    );
}

#[test]
fn the_active_preset_is_a_pick_then_the_newest_app_then_the_route_preset() {
    let mut selection = PathSelection::default();
    assert_eq!(
        selection.active(&id("flat")),
        ActivePreset::route(id("flat"))
    );
    selection.ruled = joined(&[candidate("firefox", "footsteps")], T);
    let app = ActivePreset {
        preset: id("footsteps"),
        reason: PresetReason::App("firefox".to_owned()),
    };
    assert_eq!(selection.active(&id("flat")), app);
    selection.manual = Some(ManualPick::lasting(id("bass-boost")));
    let manual = ActivePreset {
        preset: id("bass-boost"),
        reason: PresetReason::Manual(Some("firefox".to_owned())),
    };
    assert_eq!(selection.active(&id("flat")), manual);
}

#[test]
fn a_ruled_set_change_ends_the_manual_pick() {
    let mut selection = PathSelection {
        manual: Some(ManualPick::lasting(id("bass-boost"))),
        ..PathSelection::default()
    };
    selection.observe(&[candidate("firefox", "footsteps")], T);
    selection.advance(T.after(RULE_ENTER - 1));
    assert!(selection.manual.is_some());
    selection.advance(T.after(RULE_ENTER));
    assert_eq!(selection.manual, None);
}

#[test]
fn ending_a_preview_keeps_a_real_pick() {
    let pick = ManualPick::lasting(id("bass-boost"));
    let mut selection = PathSelection {
        manual: Some(pick.clone()),
        ..PathSelection::default()
    };
    selection.end_preview();
    assert_eq!(selection.manual, Some(pick));
    selection.manual = Some(ManualPick::preview(id("podcast")));
    selection.end_preview();
    assert_eq!(selection.manual, None);
}

#[test]
fn the_selection_wakes_for_the_earliest_path() {
    let mut selection = Selection::default();
    selection
        .get_mut(SoundPath::Bluetooth)
        .observe(&[candidate("spotify", "podcast")], T);
    selection
        .get_mut(SoundPath::Dongle)
        .observe(&[candidate("firefox", "footsteps")], T.after(500));
    assert_eq!(selection.next_deadline(), Some(T.after(RULE_ENTER)));
}

#[test]
fn ending_manual_picks_ends_them_on_both_paths() {
    let mut selection = Selection::default();
    for path in PATHS {
        selection.get_mut(path).manual = Some(ManualPick::lasting(id("podcast")));
    }
    selection.end_manual_picks();
    assert!(
        PATHS
            .iter()
            .all(|path| selection.get(*path).manual.is_none())
    );
}

#[test]
fn candidates_are_running_ruled_streams_on_the_path() {
    let rules: AppRules = [
        ("firefox", "footsteps"),
        ("spotify", "podcast"),
        ("mpv", "flat"),
    ]
    .into_iter()
    .map(|(app, preset)| (app.to_owned(), id(preset)))
    .collect();
    let streams = [
        stream("firefox", Some(SoundPath::Dongle), true),
        stream("firefox", Some(SoundPath::Dongle), true),
        stream("spotify", Some(SoundPath::Bluetooth), true),
        stream("mpv", Some(SoundPath::Dongle), false),
        stream("vlc", Some(SoundPath::Dongle), true),
    ];
    let found = candidates(&streams, &rules, &Catalog::default(), SoundPath::Dongle);
    let firefox = Candidate {
        key: "org.firefox.App".to_owned(),
        name: "firefox".to_owned(),
        preset: id("footsteps"),
    };
    assert_eq!(found, [firefox]);
}

#[test]
fn a_rule_naming_a_missing_preset_never_applies() {
    let rules: AppRules = [("firefox".to_owned(), id("deleted"))]
        .into_iter()
        .collect();
    let streams = [stream("firefox", Some(SoundPath::Dongle), true)];
    assert!(candidates(&streams, &rules, &Catalog::default(), SoundPath::Dongle).is_empty());
}

#[test]
fn the_picked_path_is_the_route_else_the_dongle() {
    use AdoptionStatus::{Adopted, NeverAdopted, Unknown};
    let cases = [
        (Route::Bluetooth, false, Unknown, Ok(SoundPath::Bluetooth)),
        (Route::Dongle, true, Adopted, Ok(SoundPath::Dongle)),
        (Route::None, true, Adopted, Ok(SoundPath::Dongle)),
        (
            Route::None,
            true,
            NeverAdopted,
            Err(ActivateError::NotReady),
        ),
        (Route::Dongle, true, Unknown, Err(ActivateError::NotReady)),
        (Route::None, false, Unknown, Err(ActivateError::NoRoute)),
    ];
    for (route, present, adoption, expected) in cases {
        assert_eq!(
            picked_path(route, present, adoption),
            expected,
            "{route:?} {present} {adoption:?}"
        );
    }
}
