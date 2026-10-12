use std::path::{Path, PathBuf};

use arctis_core::companion::seen_apps::{SeenApp, remember};
use arctis_core::ports::{SeenAppsRepository, StoreError};

use super::files::{failure, read_text, write_atomically};
use crate::logging::warn;

#[derive(Clone, Debug)]
pub struct FileSeenApps {
    path: PathBuf,
}

impl FileSeenApps {
    #[must_use]
    pub const fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl FileSeenApps {
    fn known(&self) -> Result<Vec<SeenApp>, StoreError> {
        Ok(read_text(&self.path)?.map_or_else(Vec::new, |text| self.parsed_or_replaced(&text)))
    }

    fn parsed_or_replaced(&self, text: &str) -> Vec<SeenApp> {
        serde_json::from_str(text).unwrap_or_else(|error| {
            warn(&format!(
                "replacing unreadable {}: {error}",
                self.path.display()
            ));
            Vec::new()
        })
    }
}

impl SeenAppsRepository for FileSeenApps {
    fn load(&self) -> Result<Vec<SeenApp>, StoreError> {
        match read_text(&self.path)? {
            Some(text) => serde_json::from_str(&text).map_err(|error| failure(&self.path, &error)),
            None => Ok(Vec::new()),
        }
    }

    fn record(&mut self, app: &SeenApp) -> Result<(), StoreError> {
        let apps = remember(&self.known()?, app.clone());
        let text =
            serde_json::to_string_pretty(&apps).map_err(|error| failure(&self.path, &error))?;
        write_atomically(&self.path, &text)
    }
}
