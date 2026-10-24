// SPDX-License-Identifier: EPL-2.0
//! Local immutable snapshot adapter. This is a Rust port development facility,
//! not a replacement for the upstream PostgreSQL store or archive interchange.
#![forbid(unsafe_code)]
use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};
use syson_model::Document;

mod snapshot_io;
pub use snapshot_io::{FsSnapshotIo, SnapshotIo};

#[cfg(feature = "raw-factory-candidate")]
mod initialized_creation;
#[cfg(feature = "raw-factory-candidate")]
pub use initialized_creation::{
    CreationInitialization, InitializedCreationError, InitializedCreationReport,
    stage_initialized_creation,
};

#[derive(Debug)]
pub enum StoreError {
    Io(io::Error),
    Model(syson_model::ModelError),
    InvalidKey,
    TooLarge,
}
impl From<io::Error> for StoreError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<syson_model::ModelError> for StoreError {
    fn from(e: syson_model::ModelError) -> Self {
        Self::Model(e)
    }
}
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for StoreError {}
/// A caller-owned directory of immutable EMF JSON snapshots. Keys are ASCII
/// alphanumeric or hyphens, never paths. Existing files are never overwritten.
/// Unfinished .staged files are ignored by load; they are retained for diagnosis.
pub struct SnapshotStore {
    root: PathBuf,
}
impl SnapshotStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, StoreError> {
        fs::create_dir_all(root.as_ref())?;
        Ok(Self {
            root: root.as_ref().to_owned(),
        })
    }
    fn path(&self, key: &str) -> Result<PathBuf, StoreError> {
        if key.is_empty()
            || key.len() > 80
            || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err(StoreError::InvalidKey);
        }
        Ok(self.root.join(format!("{key}.emf.json")))
    }
    pub fn save_new(&self, key: &str, document: &Document) -> Result<PathBuf, StoreError> {
        self.save_new_with_io(key, document, &mut FsSnapshotIo)
    }
    pub fn load(&self, key: &str) -> Result<Document, StoreError> {
        let mut data = String::new();
        // Bound input before allocation; this adapter's limit is not an upstream limit.
        const LIMIT: u64 = 16 * 1024 * 1024;
        File::open(self.path(key)?)?
            .take(LIMIT + 1)
            .read_to_string(&mut data)?;
        if data.len() as u64 > LIMIT {
            return Err(StoreError::TooLarge);
        }
        Ok(Document::from_json(&data)?)
    }
}
