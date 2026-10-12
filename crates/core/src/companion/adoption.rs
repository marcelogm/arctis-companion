use super::app_settings::HeadsetValues;
use super::catalog::{Catalog, HEADSET_CURVE_NAME};
use crate::eq::preset::{Preset, PresetId};
use crate::settings::headset_settings::HeadsetSettings;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DongleIdentity {
    pid: u16,
    uniq: Option<String>,
}

impl DongleIdentity {
    #[must_use]
    pub fn new(pid: u16, uniq: Option<&str>) -> Self {
        Self {
            pid,
            uniq: uniq.filter(|serial| !serial.is_empty()).map(str::to_owned),
        }
    }

    #[must_use]
    pub fn to_setting(&self) -> String {
        match &self.uniq {
            Some(uniq) => format!("{:04x}:{uniq}", self.pid),
            None => format!("{:04x}", self.pid),
        }
    }

    #[must_use]
    pub fn from_setting(text: &str) -> Option<Self> {
        let (pid, uniq) = text
            .split_once(':')
            .map_or((text, None), |(pid, uniq)| (pid, Some(uniq)));
        let pid = u16::from_str_radix(pid, 16).ok()?;
        Some(Self::new(pid, uniq))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Adoption {
    pub values: HeadsetValues,
    pub preset: PresetId,
    pub new_preset: Option<Preset>,
}

#[must_use]
pub fn adopt(observed: &HeadsetSettings, catalog: &Catalog) -> Adoption {
    let values = HeadsetValues::from_settings(observed);
    if let Some(preset) = catalog.matching(&observed.equalizer) {
        return Adoption {
            values,
            preset,
            new_preset: None,
        };
    }
    let preset = Preset::custom(
        catalog.fresh_id(HEADSET_CURVE_NAME),
        HEADSET_CURVE_NAME.to_owned(),
        observed.equalizer.clone(),
    );
    Adoption {
        values,
        preset: preset.id().clone(),
        new_preset: Some(preset),
    }
}
