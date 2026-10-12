use std::fs;
use std::path::{Path, PathBuf};

use arctis_core::eq::preset::{Preset, PresetId};
use arctis_core::ports::{PresetRepository, StoreError};

use super::files::{failure, files_with_extension, remove, write_atomically};
use crate::logging::warn;

const EXTENSION: &str = "json";

#[derive(Clone, Debug)]
pub struct FilePresets {
    dir: PathBuf,
}

impl FilePresets {
    #[must_use]
    pub const fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.dir
    }

    fn file_for(&self, id: &PresetId) -> Result<PathBuf, StoreError> {
        if is_slug(id.as_str()) {
            Ok(self.dir.join(format!("{}.{EXTENSION}", id.as_str())))
        } else {
            Err(StoreError(format!(
                "preset id {:?} is not a slug",
                id.as_str()
            )))
        }
    }
}

impl PresetRepository for FilePresets {
    fn load(&self) -> Result<Vec<Preset>, StoreError> {
        let files = files_with_extension(&self.dir, EXTENSION)?;
        Ok(files.iter().filter_map(|path| parse(path)).collect())
    }

    fn save(&mut self, preset: &Preset) -> Result<(), StoreError> {
        let path = self.file_for(preset.id())?;
        let text = serde_json::to_string_pretty(preset).map_err(|error| failure(&path, &error))?;
        write_atomically(&path, &text)
    }

    fn delete(&mut self, id: &PresetId) -> Result<(), StoreError> {
        remove(&self.file_for(id)?)
    }
}

fn is_slug(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn parse(path: &Path) -> Option<Preset> {
    let parsed = fs::read_to_string(path)
        .map_err(|error| failure(path, &error))
        .and_then(|text| serde_json::from_str(&text).map_err(|error| failure(path, &error)));
    parsed
        .map_err(|error| warn(&format!("skipping preset {error}")))
        .ok()
}
