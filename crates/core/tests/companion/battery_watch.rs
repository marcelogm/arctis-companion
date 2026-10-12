use arctis_core::battery::sample::{Sample, UnixSeconds};
use arctis_core::companion::alerts::{AlertChange, AlertId, BatteryWarning};
use arctis_core::companion::battery_watch::BatteryWatch;
use arctis_core::protocol::status::{Charging, Link};

use crate::support::START as T;
use crate::support::{discharging, percent, reading};

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
    let gap = watch
        .disconnect(T.after(60_000))
        .sample
        .expect("a closing row");
    assert_eq!(gap.at, UnixSeconds::from_millis(T.after(60_000)));
    assert_eq!(
        (gap.reading.percent, gap.reading.link),
        (percent(73), Link::HeadsetOff)
    );
    assert_eq!(watch.reading(), None);
    assert_eq!(watch.disconnect(T.after(120_000)).sample, None);
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
    assert_eq!(watch.disconnect(T.after(1_000)).sample, None);
}

#[test]
fn a_disconnect_withdraws_the_warning() {
    let mut watch = BatteryWatch::new(Vec::new());
    let _ = watch.observe(discharging(14), T, percent(15));
    let gone = watch.disconnect(T.after(1_000));
    assert_eq!(gone.alerts, [AlertChange::Withdraw(AlertId::LowBattery)]);
    assert_eq!(watch.warning(), BatteryWarning::None);
}
