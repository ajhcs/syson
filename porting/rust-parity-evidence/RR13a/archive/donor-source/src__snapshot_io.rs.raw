// SPDX-License-Identifier: EPL-2.0
//! Injectable I/O for the local immutable snapshot adapter.
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};
use syson_model::Document;

use crate::{SNAPSHOT_LIMIT, SnapshotStore, StoreError, open_nonblocking};

/// Trusted caller-supplied filesystem operations for deterministic fault testing.
/// Implementations must preserve the named operation's semantics. In particular,
/// `publish` must not replace an existing destination. An error may follow a
/// side effect; neither this trait nor `StoreError::Io` guarantees nonpublication.
/// This interface does not model crashes, power loss, or a storage server.
pub trait SnapshotIo {
    fn create_staged(&mut self, path: &Path) -> io::Result<File>;
    fn write_staged(&mut self, file: &mut File, data: &[u8]) -> io::Result<()>;
    fn sync_staged(&mut self, file: &File) -> io::Result<()>;
    fn publish(&mut self, staged: &Path, destination: &Path) -> io::Result<()>;
    fn sync_directory(&mut self, directory: &Path) -> io::Result<()>;
    fn remove_staged(&mut self, staged: &Path) -> io::Result<()>;
}

/// The original adapter's standard-library filesystem operations.
pub struct FsSnapshotIo;
impl SnapshotIo for FsSnapshotIo {
    fn create_staged(&mut self, path: &Path) -> io::Result<File> {
        OpenOptions::new().write(true).create_new(true).open(path)
    }
    fn write_staged(&mut self, file: &mut File, data: &[u8]) -> io::Result<()> {
        file.write_all(data)
    }
    fn sync_staged(&mut self, file: &File) -> io::Result<()> {
        file.sync_all()
    }
    fn publish(&mut self, staged: &Path, destination: &Path) -> io::Result<()> {
        fs::hard_link(staged, destination)
    }
    fn sync_directory(&mut self, directory: &Path) -> io::Result<()> {
        let file = open_nonblocking(directory)?;
        if !file.metadata()?.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "snapshot directory required",
            ));
        }
        file.sync_all()
    }
    fn remove_staged(&mut self, staged: &Path) -> io::Result<()> {
        fs::remove_file(staged)
    }
}

impl SnapshotStore {
    /// Save through caller-supplied trusted I/O, using the same sequence as
    /// `save_new`. Does not attempt cleanup after failure; completed operations
    /// may have side effects. An error after removal can leave no staged file.
    /// An `Io` error does not reveal whether the destination was published:
    /// load/inspect it before deciding what recovery requires. A visible file
    /// alone does not prove durability.
    pub fn save_new_with_io(
        &self,
        key: &str,
        document: &Document,
        io: &mut impl SnapshotIo,
    ) -> Result<PathBuf, StoreError> {
        let path = self.path(key)?;
        // RR04 counts canonical bytes/depth before reserving complete output.
        // Keep one write_staged call so trusted I/O fault semantics stay intact.
        let data = document.to_json_bounded(SNAPSHOT_LIMIT, || false)?;
        self.check_root()?;
        let staged = path.with_extension("staged");
        let mut file = io.create_staged(&staged)?;
        io.write_staged(&mut file, data.as_bytes())?;
        io.sync_staged(&file)?;
        // Same atomic, no-replace publication as the original local adapter.
        io.publish(&staged, &path)?;
        #[cfg(unix)]
        io.sync_directory(&self.root)?;
        io.remove_staged(&staged)?;
        self.check_root()?;
        Ok(path)
    }
}
