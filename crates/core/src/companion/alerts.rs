use std::mem;

use super::timing::{BATTERY_HYSTERESIS, CRITICAL_BATTERY};
use crate::battery::sample::Reading;
use crate::settings::values::Percent;
use crate::view::text::{Arg, Text};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BatteryWarning {
    #[default]
    None,
    Low,
    Critical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlertId {
    LowBattery,
    CriticalBattery,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alert {
    pub id: AlertId,
    pub title: Text,
    pub body: Text,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlertChange {
    Notify(Alert),
    Withdraw(AlertId),
}

#[must_use]
pub fn warning_for(
    previous: BatteryWarning,
    reading: Reading,
    threshold: Percent,
) -> BatteryWarning {
    if reading.is_plugged() {
        return BatteryWarning::None;
    }
    if !reading.is_connected() {
        return previous;
    }
    let value = reading.percent.get();
    entered(value, threshold.get()).max(held(previous, value, threshold.get()))
}

const fn entered(value: u8, threshold: u8) -> BatteryWarning {
    if value <= CRITICAL_BATTERY {
        BatteryWarning::Critical
    } else if value <= threshold {
        BatteryWarning::Low
    } else {
        BatteryWarning::None
    }
}

fn held(previous: BatteryWarning, value: u8, threshold: u8) -> BatteryWarning {
    let below = |level: u8| value < level.saturating_add(BATTERY_HYSTERESIS);
    match previous {
        BatteryWarning::Critical if below(CRITICAL_BATTERY) => BatteryWarning::Critical,
        BatteryWarning::Critical | BatteryWarning::Low if below(threshold) => BatteryWarning::Low,
        _ => BatteryWarning::None,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Sent {
    low: bool,
    critical: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Alerts {
    warning: BatteryWarning,
    sent: Sent,
}

impl Alerts {
    #[must_use]
    pub const fn warning(&self) -> BatteryWarning {
        self.warning
    }

    pub fn update(&mut self, reading: Reading, threshold: Percent) -> Vec<AlertChange> {
        self.warning = warning_for(self.warning, reading, threshold);
        if reading.is_plugged() {
            return self.withdraw_all();
        }
        match self.warning {
            BatteryWarning::Critical => self.notify_critical(reading.percent),
            BatteryWarning::Low => self.notify_low(reading.percent),
            BatteryWarning::None => Vec::new(),
        }
    }

    fn withdraw_all(&mut self) -> Vec<AlertChange> {
        let sent = mem::take(&mut self.sent);
        [
            (sent.low, AlertId::LowBattery),
            (sent.critical, AlertId::CriticalBattery),
        ]
        .into_iter()
        .filter(|(was_sent, _)| *was_sent)
        .map(|(_, id)| AlertChange::Withdraw(id))
        .collect()
    }

    fn notify_low(&mut self, level: Percent) -> Vec<AlertChange> {
        if self.sent.low || self.sent.critical {
            return Vec::new();
        }
        self.sent.low = true;
        vec![AlertChange::Notify(low_alert(level))]
    }

    fn notify_critical(&mut self, level: Percent) -> Vec<AlertChange> {
        if self.sent.critical {
            return Vec::new();
        }
        let withdrawn = self
            .sent
            .low
            .then_some(AlertChange::Withdraw(AlertId::LowBattery));
        self.sent = Sent {
            low: false,
            critical: true,
        };
        withdrawn
            .into_iter()
            .chain([AlertChange::Notify(critical_alert(level))])
            .collect()
    }
}

fn low_alert(level: Percent) -> Alert {
    Alert {
        id: AlertId::LowBattery,
        title: Text::plain("Headset Battery Low"),
        body: Text::format(
            "{percent} remaining",
            vec![("percent", Arg::Percent(level))],
        ),
    }
}

fn critical_alert(level: Percent) -> Alert {
    Alert {
        id: AlertId::CriticalBattery,
        title: Text::plain("Headset Battery Critical"),
        body: Text::format(
            "{percent} remaining. Charge the headset soon.",
            vec![("percent", Arg::Percent(level))],
        ),
    }
}
