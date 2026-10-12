use super::alerts::{AlertChange, Alerts, BatteryWarning};
use crate::battery::log_format::retain_recent;
use crate::battery::sample::{Reading, Sample, Sampler, UnixSeconds};
use crate::protocol::status::Link;
use crate::settings::values::Percent;
use crate::time::Millis;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BatteryUpdate {
    pub sample: Option<Sample>,
    pub alerts: Vec<AlertChange>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatteryWatch {
    sampler: Sampler,
    history: Vec<Sample>,
    alerts: Alerts,
    reading: Option<Reading>,
}

impl BatteryWatch {
    #[must_use]
    pub fn new(history: Vec<Sample>) -> Self {
        Self {
            sampler: Sampler::starting_from(&history),
            history,
            alerts: Alerts::default(),
            reading: None,
        }
    }

    pub fn observe(&mut self, reading: Reading, now: Millis, threshold: Percent) -> BatteryUpdate {
        self.reading = Some(reading);
        BatteryUpdate {
            sample: self.record(reading, now),
            alerts: self.alerts.update(reading, threshold),
        }
    }

    pub fn threshold_changed(&mut self, threshold: Percent) -> Vec<AlertChange> {
        let Some(reading) = self.reading else {
            return Vec::new();
        };
        self.alerts.update(reading, threshold)
    }

    pub fn disconnect(&mut self, now: Millis) -> BatteryUpdate {
        BatteryUpdate {
            sample: self.close_line(now),
            alerts: self.alerts.reset(),
        }
    }

    fn close_line(&mut self, now: Millis) -> Option<Sample> {
        let last = self
            .reading
            .take()
            .filter(|reading| reading.is_connected())?;
        let closing = Reading {
            link: Link::HeadsetOff,
            ..last
        };
        self.record(closing, now)
    }

    fn record(&mut self, reading: Reading, now: Millis) -> Option<Sample> {
        let at = UnixSeconds::from_millis(now);
        let sample = self.sampler.observe(reading, at)?;
        self.history = retain_recent(&self.history, at);
        self.history.push(sample);
        Some(sample)
    }

    #[must_use]
    pub const fn reading(&self) -> Option<Reading> {
        self.reading
    }

    #[must_use]
    pub fn history(&self) -> &[Sample] {
        &self.history
    }

    #[must_use]
    pub const fn warning(&self) -> BatteryWarning {
        self.alerts.warning()
    }
}
