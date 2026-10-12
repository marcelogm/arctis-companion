use std::path::{Path, PathBuf};

use arctis_core::battery::log_format::{parse_log, render_line, render_log};
use arctis_core::battery::sample::{Sample, UnixSeconds};
use arctis_core::ports::{BatteryRepository, StoreError};

use super::files::{append, read_text, write_atomically};

#[derive(Clone, Debug)]
pub struct FileBattery {
    path: PathBuf,
}

impl FileBattery {
    #[must_use]
    pub const fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl BatteryRepository for FileBattery {
    fn load(&self) -> Result<Vec<Sample>, StoreError> {
        Ok(read_text(&self.path)?
            .map(|text| parse_log(&text))
            .unwrap_or_default())
    }

    fn append(&mut self, sample: &Sample) -> Result<(), StoreError> {
        append(&self.path, &render_line(sample))
    }

    fn retain_since(&mut self, cutoff: UnixSeconds) -> Result<(), StoreError> {
        let kept: Vec<Sample> = self
            .load()?
            .into_iter()
            .filter(|sample| sample.at >= cutoff)
            .collect();
        write_atomically(&self.path, &render_log(&kept))
    }
}
