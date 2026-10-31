// SPDX-License-Identifier: EPL-2.0
//! Local immutable snapshot adapter. This is a Rust port development facility,
//! not a replacement for the upstream PostgreSQL store or archive interchange.
#![forbid(unsafe_code)]
use std::{
    fs::{self, File},
    io,
    path::{Component, Path, PathBuf},
};
use syson_model::Document;

const SNAPSHOT_LIMIT: usize = 4 * 1024 * 1024;

mod snapshot_io;
pub use snapshot_io::{FsSnapshotIo, SnapshotIo};

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
    directory: File,
}
impl SnapshotStore {
    /// Linux x86_64 only. Require an absolute directory and reject root or
    /// ancestor symlinks. Callers must keep owned parent directories stable;
    /// metadata checks are not descriptor-relative openat or hostile-writer locks.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, StoreError> {
        supported()?;
        let root = root.as_ref();
        if !root.is_absolute()
            || !root
                .components()
                .all(|c| matches!(c, Component::RootDir | Component::Normal(_)))
        {
            return Err(invalid(
                "absolute snapshot root without parent components required",
            ));
        }
        let root: PathBuf = root.components().collect();
        directories(&root, true)?;
        fs::create_dir_all(&root)?;
        directories(&root, false)?;
        let admitted = fs::symlink_metadata(&root)?;
        let directory = open_nonblocking(&root)?;
        let opened = directory.metadata()?;
        if !opened.is_dir() || identity(&admitted)? != identity(&opened)? {
            return Err(invalid("snapshot directory changed during open"));
        }
        let store = Self { root, directory };
        store.check_root()?;
        Ok(store)
    }
    fn check_root(&self) -> Result<(), StoreError> {
        directories(&self.root, false)?;
        let meta = fs::symlink_metadata(&self.root)?;
        if !meta.is_dir() || identity(&meta)? != identity(&self.directory.metadata()?)? {
            return Err(invalid("snapshot directory identity changed"));
        }
        Ok(())
    }
    fn path(&self, key: &str) -> Result<PathBuf, StoreError> {
        if key.is_empty()
            || key.len() > 80
            || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err(StoreError::InvalidKey);
        }
        self.check_root()?;
        Ok(self.root.join(format!("{key}.emf.json")))
    }
    pub fn save_new(&self, key: &str, document: &Document) -> Result<PathBuf, StoreError> {
        self.save_new_with_io(key, document, &mut FsSnapshotIo)
    }
    /// Final symlinks and special files refuse. Opened identity and post-read
    /// metadata must agree; this is not a regular-disk-I/O deadline or a snapshot
    /// against writers that can change bytes and metadata concurrently.
    pub fn load(&self, key: &str) -> Result<Document, StoreError> {
        let data = read_regular(&self.path(key)?, || {})?;
        self.check_root()?;
        Ok(Document::from_json(&data)?)
    }
}
fn invalid(message: &'static str) -> StoreError {
    io::Error::new(io::ErrorKind::InvalidInput, message).into()
}
fn unsupported() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "snapshot admission requires Linux x86_64",
    )
}
fn supported() -> Result<(), StoreError> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Ok(())
    } else {
        Err(unsupported().into())
    }
}
fn directories(path: &Path, allow_missing: bool) -> Result<(), StoreError> {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => (),
            Ok(_) => {
                return Err(invalid(
                    "snapshot root and ancestors must be nonsymlink directories",
                ));
            }
            Err(e) if allow_missing && e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
fn identity(meta: &fs::Metadata) -> Result<(u64, u64), StoreError> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        use std::os::unix::fs::MetadataExt;
        Ok((meta.dev(), meta.ino()))
    }
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    {
        let _ = meta;
        Err(unsupported().into())
    }
}
fn open_nonblocking(path: &Path) -> io::Result<File> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // Linux x86_64 O_NONBLOCK | O_NOFOLLOW; not arbitrary-device race safety.
        fs::OpenOptions::new()
            .read(true)
            .custom_flags(0x800 | 0x20000)
            .open(path)
    }
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    {
        let _ = path;
        Err(unsupported())
    }
}
#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
fn read_regular(path: &Path, before_open: impl FnOnce()) -> Result<String, StoreError> {
    let _ = (path, before_open);
    Err(unsupported().into())
}
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn read_regular(path: &Path, before_open: impl FnOnce()) -> Result<String, StoreError> {
    use std::{io::Read, os::unix::fs::MetadataExt};
    let regular = |meta: &fs::Metadata| -> Result<(), StoreError> {
        if !meta.is_file() {
            return Err(invalid("regular nonsymlink snapshot required"));
        }
        if meta.len() > SNAPSHOT_LIMIT as u64 {
            return Err(StoreError::TooLarge);
        }
        Ok(())
    };
    let same = |a: &fs::Metadata, b: &fs::Metadata| {
        a.dev() == b.dev()
            && a.ino() == b.ino()
            && a.len() == b.len()
            && a.mtime() == b.mtime()
            && a.mtime_nsec() == b.mtime_nsec()
            && a.ctime() == b.ctime()
            && a.ctime_nsec() == b.ctime_nsec()
    };
    let admitted = fs::symlink_metadata(path)?;
    regular(&admitted)?;
    // Private deterministic test seam; production always supplies a no-op.
    before_open();
    let file = open_nonblocking(path)?;
    let opened = file.metadata()?;
    regular(&opened)?;
    if !same(&admitted, &opened) {
        return Err(invalid("snapshot changed during admission"));
    }
    let size = opened.len() as usize;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|_| invalid("snapshot allocation refused"))?;
    let mut input = (&file).take(size as u64 + 1);
    let mut chunk = [0; 8192];
    loop {
        let read = input.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        if read > size.saturating_sub(bytes.len()) {
            return Err(invalid("snapshot grew during read"));
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    let descriptor_after = file.metadata()?;
    let path_after = fs::symlink_metadata(path)?;
    regular(&descriptor_after)?;
    regular(&path_after)?;
    if !same(&opened, &descriptor_after) || !same(&descriptor_after, &path_after) {
        return Err(invalid("snapshot changed during read"));
    }
    String::from_utf8(bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "snapshot UTF8 required").into())
}

#[cfg(all(test, target_os = "linux", target_arch = "x86_64"))]
mod rr13_open_races {
    use super::*;
    use std::{
        process::{Command, Stdio},
        thread,
        time::{Duration, Instant},
    };
    fn run(name: &str, kind: &str) {
        if std::env::var("RR13_PRIVATE_CHILD").ok().as_deref() == Some(name) {
            let base =
                PathBuf::from(std::env::var_os("SYSON_TEST_DIR").expect("owned root required"))
                    .canonicalize()
                    .unwrap();
            assert!(base.starts_with("/workspace"));
            let dir = base.join(format!("rr13-race-{}", std::process::id()));
            fs::create_dir(&dir).unwrap();
            let target = dir.join("input.emf.json");
            let replacement = dir.join("replacement");
            fs::write(&target, b"original snapshot bytes").unwrap();
            match kind {
                "fifo" => assert!(
                    Command::new("/usr/bin/mkfifo")
                        .arg("--")
                        .arg(&replacement)
                        .env_clear()
                        .status()
                        .unwrap()
                        .success()
                ),
                "symlink" => {
                    fs::write(dir.join("referent"), b"replacement snapshot bytes").unwrap();
                    std::os::unix::fs::symlink("referent", &replacement).unwrap();
                }
                "regular" => fs::write(&replacement, b"replacement snapshot bytes").unwrap(),
                _ => unreachable!(),
            }
            let retained = dir.join("retained");
            let result = read_regular(&target, || {
                fs::rename(&target, &retained).unwrap();
                fs::rename(&replacement, &target).unwrap();
            });
            assert!(result.is_err(), "changed admitted file was accepted");
            assert_eq!(fs::read(retained).unwrap(), b"original snapshot bytes");
            return;
        }
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", name, "--nocapture", "--test-threads=1"])
            .env_clear()
            .env("RR13_PRIVATE_CHILD", name)
            .env(
                "SYSON_TEST_DIR",
                std::env::var_os("SYSON_TEST_DIR").unwrap(),
            )
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let start = Instant::now();
        let mut timed_out = false;
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            if start.elapsed() >= Duration::from_secs(3) {
                timed_out = true;
                let _ = child.kill();
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            !timed_out,
            "admission race child killed and waited after deadline"
        );
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("running 1 test") && stdout.contains("1 passed"),
            "no exact race case: {stdout}"
        );
    }
    #[test]
    fn regular_to_fifo_between_admission_and_open_refuses() {
        run(
            "rr13_open_races::regular_to_fifo_between_admission_and_open_refuses",
            "fifo",
        );
    }
    #[test]
    fn regular_to_symlink_between_admission_and_open_refuses() {
        run(
            "rr13_open_races::regular_to_symlink_between_admission_and_open_refuses",
            "symlink",
        );
    }
    #[test]
    fn regular_to_new_inode_between_admission_and_open_refuses() {
        run(
            "rr13_open_races::regular_to_new_inode_between_admission_and_open_refuses",
            "regular",
        );
    }
}
