use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use arctis_core::ports::StoreError;
use gtk::glib;

pub const APP_DIR: &str = "arctis-companion";

#[must_use]
pub fn data_dir() -> PathBuf {
    glib::user_data_dir().join(APP_DIR)
}

pub fn read_text(path: &Path) -> Result<Option<String>, StoreError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(failure(path, &error)),
    }
}

pub fn write_atomically(path: &Path, text: &str) -> Result<(), StoreError> {
    create_parent(path)?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, text).map_err(|error| failure(&temporary, &error))?;
    fs::rename(&temporary, path).map_err(|error| failure(path, &error))
}

pub fn append(path: &Path, text: &str) -> Result<(), StoreError> {
    create_parent(path)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| failure(path, &error))?;
    file.write_all(text.as_bytes())
        .map_err(|error| failure(path, &error))
}

pub fn remove(path: &Path) -> Result<(), StoreError> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(failure(path, &error)),
        _ => Ok(()),
    }
}

pub fn files_with_extension(dir: &Path, extension: &str) -> Result<Vec<PathBuf>, StoreError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(failure(dir, &error)),
    };
    Ok(entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|found| found == extension))
        .collect())
}

#[must_use]
pub fn failure(path: &Path, error: &impl ToString) -> StoreError {
    StoreError(format!("{}: {}", path.display(), error.to_string()))
}

fn create_parent(path: &Path) -> Result<(), StoreError> {
    match path.parent() {
        Some(dir) => fs::create_dir_all(dir).map_err(|error| failure(dir, &error)),
        None => Ok(()),
    }
}
