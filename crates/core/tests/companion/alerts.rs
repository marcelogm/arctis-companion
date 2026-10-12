use arctis_core::companion::alerts::{AlertChange, AlertId, Alerts, BatteryWarning, warning_for};
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::testkit::english::english;

use crate::support::{discharging, percent, reading};

fn ids(changes: &[AlertChange]) -> Vec<(bool, AlertId)> {
    changes
        .iter()
        .map(|change| match change {
            AlertChange::Notify(alert) => (true, alert.id),
            AlertChange::Withdraw(id) => (false, *id),
        })
        .collect()
}

#[test]
fn warning_levels_follow_the_threshold_with_hysteresis() {
    use BatteryWarning as W;
    let cases = [
        (W::None, 16, W::None),
        (W::None, 15, W::Low),
        (W::None, 6, W::Low),
        (W::None, 5, W::Critical),
        (W::Low, 17, W::Low),
        (W::Low, 18, W::None),
        (W::Low, 5, W::Critical),
        (W::Critical, 7, W::Critical),
        (W::Critical, 8, W::Low),
        (W::Critical, 17, W::Low),
        (W::Critical, 18, W::None),
    ];
    for (previous, value, expected) in cases {
        let warning = warning_for(previous, discharging(value), percent(15));
        assert_eq!(warning, expected, "{previous:?} at {value} %");
    }
}

#[test]
fn charging_clears_the_warning() {
    for charging in [Charging::Charging, Charging::PluggedIn] {
        let plugged = reading(3, charging, Link::Connected);
        assert_eq!(
            warning_for(BatteryWarning::Critical, plugged, percent(15)),
            BatteryWarning::None
        );
    }
}

#[test]
fn a_reading_without_a_link_keeps_the_warning() {
    let off = reading(0, Charging::Unknown, Link::HeadsetOff);
    assert_eq!(
        warning_for(BatteryWarning::Low, off, percent(15)),
        BatteryWarning::Low
    );
}

#[test]
fn the_threshold_is_the_users() {
    assert_eq!(
        warning_for(BatteryWarning::None, discharging(20), percent(20)),
        BatteryWarning::Low
    );
}

#[test]
fn low_battery_notifies_once_per_discharge() {
    let mut alerts = Alerts::default();
    assert_eq!(
        ids(&alerts.update(discharging(15), percent(15))),
        [(true, AlertId::LowBattery)]
    );
    for value in [14, 17, 15] {
        assert!(
            alerts.update(discharging(value), percent(15)).is_empty(),
            "{value} %"
        );
    }
}

#[test]
fn critical_battery_replaces_the_low_notification() {
    let mut alerts = Alerts::default();
    let _ = alerts.update(discharging(15), percent(15));
    assert_eq!(
        ids(&alerts.update(discharging(5), percent(15))),
        [
            (false, AlertId::LowBattery),
            (true, AlertId::CriticalBattery)
        ]
    );
    assert_eq!(alerts.warning(), BatteryWarning::Critical);
    assert!(alerts.update(discharging(9), percent(15)).is_empty());
}

#[test]
fn charging_withdraws_what_was_sent_and_starts_a_new_discharge() {
    let mut alerts = Alerts::default();
    let _ = alerts.update(discharging(4), percent(15));
    let plugged = reading(4, Charging::Charging, Link::Connected);
    assert_eq!(
        ids(&alerts.update(plugged, percent(15))),
        [(false, AlertId::CriticalBattery)]
    );
    assert_eq!(
        ids(&alerts.update(discharging(15), percent(15))),
        [(true, AlertId::LowBattery)]
    );
}

#[test]
fn alert_texts_name_the_level_and_the_percent() {
    let mut alerts = Alerts::default();
    let changes = alerts.update(discharging(14), percent(15));
    let Some(AlertChange::Notify(alert)) = changes.first() else {
        panic!("a low battery notification");
    };
    assert_eq!(english(&alert.title), "Headset Battery Low");
    assert_eq!(english(&alert.body), "14 % remaining");
}

#[test]
fn a_recharge_the_app_did_not_see_starts_a_new_discharge() {
    let mut alerts = Alerts::default();
    let _ = alerts.update(discharging(14), percent(15));
    assert_eq!(
        ids(&alerts.update(discharging(90), percent(15))),
        [(false, AlertId::LowBattery)]
    );
    assert_eq!(
        ids(&alerts.update(discharging(14), percent(15))),
        [(true, AlertId::LowBattery)]
    );
}

#[test]
fn a_reset_withdraws_what_was_sent_and_clears_the_warning() {
    let mut alerts = Alerts::default();
    let _ = alerts.update(discharging(14), percent(15));
    assert_eq!(ids(&alerts.reset()), [(false, AlertId::LowBattery)]);
    assert_eq!(alerts.warning(), BatteryWarning::None);
    assert!(alerts.reset().is_empty());
}
