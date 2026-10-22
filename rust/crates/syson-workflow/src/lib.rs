// SPDX-License-Identifier: EPL-2.0
//! Local immutable snapshot adapter. This is a Rust port development facility,
//! not a replacement for the upstream PostgreSQL store or archive interchange.
#![forbid(unsafe_code)]
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use syson_model::Document;

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
        let path = self.path(key)?;
        let data = document.to_json()?;
        if data.len() > 16 * 1024 * 1024 {
            return Err(StoreError::TooLarge);
        }
        let staged = path.with_extension("staged");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)?;
        file.write_all(data.as_bytes())?;
        file.sync_all()?;
        // link is atomic and fails if the destination exists; unlike rename it
        // cannot silently overwrite a prior snapshot on Unix.
        fs::hard_link(&staged, &path)?;
        #[cfg(unix)]
        File::open(&self.root)?.sync_all()?;
        fs::remove_file(&staged)?;
        Ok(path)
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
