use arctis_core::companion::engines::{Cause, Decision, EditorThrottle, Engines, Trigger};
use arctis_core::companion::selection::{ActivePreset, PresetReason};
use arctis_core::companion::timing::EDITOR_THROTTLE;
use arctis_core::messages::audio::SoundPath;
use arctis_core::time::Millis;

use crate::support::id;

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
