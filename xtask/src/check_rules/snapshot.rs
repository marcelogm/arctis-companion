use std::fs;
use std::path::Path;

use crate::git;

#[derive(Clone, Copy)]
pub enum Snapshot<'r> {
    WorkingTree(&'r Path),
    Index(&'r Path),
}

impl Snapshot<'_> {
    pub fn read(self, file: &str) -> Option<String> {
        match self {
            Self::WorkingTree(root) => fs::read_to_string(root.join(file)).ok(),
            Self::Index(root) => git::staged_blob(root, file),
        }
    }

    pub fn files_to_check(self) -> Result<Vec<String>, String> {
        match self {
            Self::WorkingTree(root) => git::tracked_files(root),
            Self::Index(root) => git::staged_files(root),
        }
    }

    pub fn tracked_files(self) -> Result<Vec<String>, String> {
        match self {
            Self::WorkingTree(root) | Self::Index(root) => git::tracked_files(root),
        }
    }
}
