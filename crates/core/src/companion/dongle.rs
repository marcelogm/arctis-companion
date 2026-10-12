use super::adoption::{AdoptionStatus, DongleIdentity};
use crate::eq::curve::equivalent;
use crate::messages::device::Phase;
use crate::protocol::firmware::Firmware;
use crate::protocol::model::Model;
use crate::settings::headset_settings::{HeadsetSettings, Setting, SettingChange};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WriteFailure {
    pub serial: u32,
    pub setting: Setting,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DongleState {
    pub phase: Phase,
    pub identity: Option<DongleIdentity>,
    pub model: Option<Model>,
    pub firmware: Option<Firmware>,
    pub observed: Option<HeadsetSettings>,
    pub not_applied: Vec<Setting>,
    pub last_failure: Option<WriteFailure>,
}

impl DongleState {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            phase: Phase::Absent,
            identity: None,
            model: None,
            firmware: None,
            observed: None,
            not_applied: Vec::new(),
            last_failure: None,
        }
    }

    #[must_use]
    pub const fn is_present(&self) -> bool {
        is_present(self.phase)
    }

    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self.phase, Phase::Ready)
    }

    #[must_use]
    pub fn adoption(&self, adopted: Option<&DongleIdentity>) -> AdoptionStatus {
        match &self.identity {
            None => AdoptionStatus::Unknown,
            Some(identity) if adopted == Some(identity) => AdoptionStatus::Adopted,
            Some(_) => AdoptionStatus::NeverAdopted,
        }
    }

    pub fn set_phase(&mut self, phase: Phase) {
        if !is_present(phase) {
            self.forget();
        }
        self.phase = phase;
    }

    pub fn identify(&mut self, model: Model, uniq: Option<&str>, firmware: Firmware) {
        self.identity = Some(DongleIdentity::new(model.pid, uniq));
        self.model = Some(model);
        self.firmware = Some(firmware);
    }

    pub fn observe(&mut self, observed: HeadsetSettings, desired: &HeadsetSettings) {
        self.not_applied
            .retain(|setting| !same_value(&observed, desired, *setting));
        self.observed = Some(observed);
    }

    pub fn headset_changed(&mut self, change: &SettingChange) {
        if let Some(observed) = &mut self.observed {
            observed.apply(change.clone());
        }
    }

    pub fn write_failed(&mut self, setting: Setting) {
        if !self.not_applied.contains(&setting) {
            self.not_applied.push(setting);
        }
        let serial = self
            .last_failure
            .map_or(1, |failure| failure.serial.wrapping_add(1));
        self.last_failure = Some(WriteFailure { serial, setting });
    }

    fn forget(&mut self) {
        self.identity = None;
        self.model = None;
        self.firmware = None;
        self.observed = None;
        self.not_applied.clear();
    }
}

impl Default for DongleState {
    fn default() -> Self {
        Self::new()
    }
}

const fn is_present(phase: Phase) -> bool {
    !matches!(phase, Phase::Absent | Phase::PermissionDenied)
}

fn same_value(observed: &HeadsetSettings, desired: &HeadsetSettings, setting: Setting) -> bool {
    match setting {
        Setting::Equalizer => equivalent(&observed.equalizer, &desired.equalizer),
        other => observed.value_of(other) == desired.value_of(other),
    }
}
