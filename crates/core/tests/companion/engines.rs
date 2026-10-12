use arctis_core::battery::sample::{Sample, UnixSeconds};
use arctis_core::companion::action::Action;
use arctis_core::companion::alerts::{AlertChange, BatteryWarning};
use arctis_core::companion::battery_watch::BatteryWatch;
use arctis_core::companion::deadlines::{Deadlines, earliest};
use arctis_core::companion::engines::{Cause, Decision, EditorThrottle, Engines, Trigger};
use arctis_core::companion::selection::{ActivePreset, PresetReason};
use arctis_core::companion::timing::EDITOR_THROTTLE;
use arctis_core::messages::audio::SoundPath;
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::time::Millis;

use crate::support::{discharging, id, percent, reading};

const T: Millis = Millis(1_760_000_000_000);

fn app(preset: &str) -> ActivePreset {
    ActivePreset {
        preset: id(preset),
        reason: PresetReason::App("firefox".to_owned()),
    }
}

const fn at(offset: u64, cause: Cause) -> Trigger {
    Trigger {
        now: T.after(offset),
        cause,
    }
}

fn after_switch(path: SoundPath) -> Engines {
    let mut engines = Engines::default();
    engines.record(path, ActivePreset::route(id("flat")), Some(T));
    engines
}

#[test]
fn the_first_target_is_applied() {
    let decision = Engines::default().decide(
        SoundPath::Dongle,
        &app("footsteps"),
        at(0, Cause::Automatic),
    );
    assert_eq!(decision, Decision::Apply);
}

#[test]
fn the_applied_target_is_kept() {
    let mut engines = after_switch(SoundPath::Dongle);
    let flat = ActivePreset::route(id("flat"));
    assert_eq!(
        engines.decide(SoundPath::Dongle, &flat, at(1, Cause::Automatic)),
        Decision::Keep
    );
}

#[test]
fn the_same_preset_for_another_reason_is_relabelled() {
    let mut engines = after_switch(SoundPath::Dongle);
    assert_eq!(
        engines.decide(SoundPath::Dongle, &app("flat"), at(1, Cause::Automatic)),
        Decision::Relabel
    );
}

#[test]
fn an_automatic_switch_waits_for_the_path_budget() {
    let cases = [(SoundPath::Dongle, 30_000), (SoundPath::Bluetooth, 2_000)];
    for (path, budget) in cases {
        let mut engines = after_switch(path);
        assert_eq!(
            engines.decide(path, &app("footsteps"), at(budget - 1, Cause::Automatic)),
            Decision::Hold
        );
        assert_eq!(engines.next_deadline(), Some(T.after(budget)));
        assert_eq!(
            engines.decide(path, &app("footsteps"), at(budget, Cause::Automatic)),
            Decision::Apply
        );
        assert_eq!(engines.next_deadline(), None);
    }
}

#[test]
fn a_user_switch_ignores_the_budget() {
    let mut engines = after_switch(SoundPath::Dongle);
    assert_eq!(
        engines.decide(SoundPath::Dongle, &app("footsteps"), at(1, Cause::User)),
        Decision::Apply
    );
}

#[test]
fn a_switch_that_wrote_nothing_spends_no_budget() {
    let mut engines = Engines::default();
    engines.record(SoundPath::Dongle, ActivePreset::route(id("flat")), None);
    assert_eq!(
        engines.decide(
            SoundPath::Dongle,
            &app("footsteps"),
            at(1, Cause::Automatic)
        ),
        Decision::Apply
    );
}

#[test]
fn editor_writes_are_throttled() {
    let mut throttle = EditorThrottle::default();
    assert!(!throttle.take_due(T));
    throttle.request();
    assert!(throttle.take_due(T));
    throttle.request();
    assert!(!throttle.take_due(T.after(100)));
    assert_eq!(throttle.next_deadline(), Some(T.after(EDITOR_THROTTLE)));
    assert!(throttle.take_due(T.after(EDITOR_THROTTLE)));
    assert_eq!(throttle.next_deadline(), None);
}

#[test]
fn a_cancelled_edit_is_not_written() {
    let mut throttle = EditorThrottle::default();
    throttle.request();
    throttle.cancel();
    assert!(!throttle.take_due(T));
}

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

#[test]
fn the_battery_watch_records_changes_and_raises_alerts() {
    let mut watch = BatteryWatch::new(Vec::new());
    let first = watch.observe(discharging(15), T, percent(15));
    assert_eq!(
        first.sample.map(|s| s.at),
        Some(UnixSeconds::from_millis(T))
    );
    assert!(matches!(first.alerts.as_slice(), [AlertChange::Notify(_)]));
    let repeat = watch.observe(discharging(15), T.after(60_000), percent(15));
    assert_eq!((repeat.sample, repeat.alerts.len()), (None, 0));
    assert_eq!(watch.history().len(), 1);
    assert_eq!(watch.warning(), BatteryWarning::Low);
}

#[test]
fn the_battery_watch_drops_history_older_than_eight_days() {
    let old = Sample {
        at: UnixSeconds::from_millis(T).minus(9 * 24 * 60 * 60),
        reading: discharging(90),
    };
    let mut watch = BatteryWatch::new(vec![old]);
    let _ = watch.observe(discharging(80), T, percent(15));
    assert_eq!(watch.history().len(), 1);
}

#[test]
fn a_new_threshold_re_evaluates_the_last_reading() {
    let mut watch = BatteryWatch::new(Vec::new());
    assert!(watch.threshold_changed(percent(20)).is_empty());
    let _ = watch.observe(discharging(18), T, percent(15));
    assert!(matches!(
        watch.threshold_changed(percent(20)).as_slice(),
        [AlertChange::Notify(_)]
    ));
}

#[test]
fn a_disconnect_closes_the_line_once() {
    let mut watch = BatteryWatch::new(Vec::new());
    let _ = watch.observe(discharging(73), T, percent(15));
    let gap = watch.disconnect(T.after(60_000)).expect("a closing row");
    assert_eq!(gap.at, UnixSeconds::from_millis(T.after(60_000)));
    assert_eq!(
        (gap.reading.percent, gap.reading.link),
        (percent(73), Link::HeadsetOff)
    );
    assert_eq!(watch.reading(), None);
    assert_eq!(watch.disconnect(T.after(120_000)), None);
    assert_eq!(watch.history().len(), 2);
}

#[test]
fn a_reading_already_without_a_link_needs_no_closing_row() {
    let mut watch = BatteryWatch::new(Vec::new());
    let _ = watch.observe(
        reading(0, Charging::Unknown, Link::HeadsetOff),
        T,
        percent(15),
    );
    assert_eq!(watch.disconnect(T.after(1_000)), None);
}
