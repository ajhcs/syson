// SPDX-License-Identifier: EPL-2.0
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};
use syson_model::{Document, Editor, NodeIds};
use syson_workflow::{FsSnapshotIo, SnapshotIo, SnapshotStore, StoreError};
static NEXT: AtomicU64 = AtomicU64::new(0);
const FIFO_TEST: &str = "fifo_load_refuses_without_a_blocking_open";
fn directory() -> PathBuf {
    let base =
        PathBuf::from(std::env::var_os("SYSON_TEST_DIR").expect("owned workspace root required"));
    let base = base.canonicalize().unwrap();
    assert!(base.is_absolute() && base.starts_with("/workspace"));
    let dir = base.join(format!(
        "rr13-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).unwrap();
    dir
}
fn fifo(path: &Path) {
    assert!(path.is_absolute() && path.starts_with("/workspace"));
    assert!(!path.exists());
    assert!(
        Command::new("/usr/bin/mkfifo")
            .arg("--")
            .arg(path)
            .env_clear()
            .status()
            .unwrap()
            .success()
    );
}
#[test]
fn fifo_load_refuses_without_a_blocking_open() {
    if let Some(file) = std::env::var_os("RR13_CHILD_FIFO") {
        let file = PathBuf::from(file);
        assert!(file.is_absolute() && file.starts_with("/workspace"));
        let store = SnapshotStore::open(file.parent().unwrap()).unwrap();
        assert!(
            store.load("input").is_err(),
            "FIFO was admitted as a snapshot"
        );
        return;
    }
    let dir = directory();
    let file = dir.join("input.emf.json");
    fifo(&file);
    child_case(FIFO_TEST, "RR13_CHILD_FIFO", &file);
    assert!(fs::symlink_metadata(file).unwrap().file_type().is_fifo());
}
use std::os::unix::fs::FileTypeExt;

fn child_case(name: &str, variable: &str, file: &Path) {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture", "--test-threads=1"])
        .current_dir(file.parent().unwrap())
        .env_clear()
        .env(variable, file)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let start = Instant::now();
    loop {
        if child.try_wait().unwrap().is_some() {
            let output = child.wait_with_output().unwrap();
            assert!(
                output.status.success(),
                "child failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("running 1 test") && stdout.contains("1 passed"),
                "child case was not executed: {stdout}"
            );
            break;
        }
        if start.elapsed() >= Duration::from_secs(3) {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "snapshot operation exceeded the owned child deadline; child killed and waited: {}",
                output.status
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn regular_reopen_preserves_canonical_bytes_and_never_overwrites_a_destination() {
    let dir = directory();
    let document = Document::from_json(r#"{"json":{"version":"1.0","encoding":"utf-8"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[],"vendor":{"$serde_json::private::Number":"123","number":900719925474099312345}}"#).unwrap();
    let store = SnapshotStore::open(&dir).unwrap();
    let saved = store.save_new("normal", &document).unwrap();
    let bytes = fs::read(&saved).unwrap();
    assert_eq!(
        SnapshotStore::open(&dir).unwrap().load("normal").unwrap(),
        document
    );
    assert!(store.save_new("normal", &Document::empty()).is_err());
    assert_eq!(fs::read(&saved).unwrap(), bytes);
    assert!(dir.join("normal.emf.staged").is_file());
}
#[test]
fn root_and_ancestor_symlinks_and_parent_components_refuse_before_creation() {
    let dir = directory();
    let real = dir.join("real");
    fs::create_dir(&real).unwrap();
    let alias = dir.join("alias");
    std::os::unix::fs::symlink(&real, &alias).unwrap();
    assert!(SnapshotStore::open(&alias).is_err());
    assert!(SnapshotStore::open(alias.join("new-root")).is_err());
    assert!(!real.join("new-root").exists());
    assert!(SnapshotStore::open(dir.join("intermediate/../escape")).is_err());
    assert!(!dir.join("intermediate").exists());
    assert!(!dir.join("escape").exists());
}
#[test]
fn directory_replacement_invalidates_existing_load_and_save_handles() {
    let dir = directory();
    let root = dir.join("root");
    let retained = dir.join("retained");
    let store = SnapshotStore::open(&root).unwrap();
    store.save_new("original", &Document::empty()).unwrap();
    fs::rename(&root, &retained).unwrap();
    fs::create_dir(&root).unwrap();
    let bytes = Document::empty().to_json().unwrap();
    fs::write(root.join("original.emf.json"), &bytes).unwrap();
    assert!(store.load("original").is_err());
    assert!(store.save_new("new", &Document::empty()).is_err());
    for directory in [&root, &retained] {
        assert!(!directory.join("new.emf.json").exists());
        assert!(!directory.join("new.emf.staged").exists());
    }
    assert_eq!(
        fs::read(root.join("original.emf.json")).unwrap(),
        bytes.as_bytes()
    );
    assert_eq!(
        SnapshotStore::open(&root)
            .unwrap()
            .load("original")
            .unwrap(),
        Document::empty()
    );
}
#[test]
fn directory_socket_symlink_and_invalid_utf8_files_refuse_without_following() {
    let Some(marker) = std::env::var_os("RR13_CHILD_TYPES") else {
        let dir = directory();
        child_case(
            "directory_socket_symlink_and_invalid_utf8_files_refuse_without_following",
            "RR13_CHILD_TYPES",
            &dir.join("case"),
        );
        return;
    };
    let marker = PathBuf::from(marker);
    assert!(marker.is_absolute() && marker.starts_with("/workspace"));
    let dir = marker.parent().unwrap().to_path_buf();
    assert_eq!(std::env::current_dir().unwrap(), dir);
    let store = SnapshotStore::open(&dir).unwrap();
    fs::create_dir(dir.join("directory.emf.json")).unwrap();
    let _socket = std::os::unix::net::UnixListener::bind("socket.emf.json").unwrap();
    let referent = dir.join("referent.json");
    let bytes = Document::empty().to_json().unwrap();
    fs::write(&referent, &bytes).unwrap();
    std::os::unix::fs::symlink(&referent, dir.join("link.emf.json")).unwrap();
    fs::write(dir.join("invalid.emf.json"), [0xff]).unwrap();
    for key in ["directory", "socket", "link", "invalid"] {
        assert!(matches!(store.load(key), Err(StoreError::Io(_))), "{key}");
    }
    assert_eq!(fs::read(referent).unwrap(), bytes.as_bytes());
}
#[test]
fn exact_four_mib_raw_input_loads_and_one_extra_byte_refuses() {
    let dir = directory();
    let store = SnapshotStore::open(&dir).unwrap();
    let mut bytes = Document::empty().to_json().unwrap().into_bytes();
    bytes.resize(4 * 1024 * 1024, b' ');
    fs::write(dir.join("exact.emf.json"), &bytes).unwrap();
    assert_eq!(store.load("exact").unwrap(), Document::empty());
    bytes.push(b' ');
    fs::write(dir.join("over.emf.json"), &bytes).unwrap();
    assert!(matches!(store.load("over"), Err(StoreError::TooLarge)));
    assert_eq!(
        fs::metadata(dir.join("over.emf.json")).unwrap().len(),
        bytes.len() as u64
    );
}
struct NoIo {
    calls: usize,
}
impl SnapshotIo for NoIo {
    fn create_staged(&mut self, _: &Path) -> io::Result<File> {
        self.calls += 1;
        Err(io::Error::other("unexpected create"))
    }
    fn write_staged(&mut self, _: &mut File, _: &[u8]) -> io::Result<()> {
        panic!("unexpected write")
    }
    fn sync_staged(&mut self, _: &File) -> io::Result<()> {
        panic!("unexpected sync")
    }
    fn publish(&mut self, _: &Path, _: &Path) -> io::Result<()> {
        panic!("unexpected publish")
    }
    fn sync_directory(&mut self, _: &Path) -> io::Result<()> {
        panic!("unexpected directory sync")
    }
    fn remove_staged(&mut self, _: &Path) -> io::Result<()> {
        panic!("unexpected remove")
    }
}
#[test]
fn provisional_output_limit_refuses_before_any_injected_io() {
    let dir = directory();
    let store = SnapshotStore::open(&dir).unwrap();
    let editor = Editor::new(Document::empty());
    let original = editor.document().clone();
    let mut tx = editor.begin();
    for i in [1, 2] {
        let name = if i == 1 {
            "X".repeat(3 * 1024 * 1024)
        } else {
            "Y".repeat(3 * 1024 * 1024)
        };
        tx.create_package(
            None,
            NodeIds::new(
                &format!("10000000-0000-4000-8000-{i:012x}"),
                &format!("20000000-0000-4000-8000-{i:012x}"),
            )
            .unwrap(),
            None,
            &name,
        )
        .unwrap();
    }
    let mut io = NoIo { calls: 0 };
    assert!(matches!(
        store.save_new_with_io("oversized", tx.document(), &mut io),
        Err(StoreError::Model(_))
    ));
    assert_eq!(io.calls, 0);
    assert!(!dir.join("oversized.emf.staged").exists());
    assert!(!dir.join("oversized.emf.json").exists());
    assert_eq!(editor.document(), &original);
    tx.cancel();
}
struct SwapRoot {
    root: PathBuf,
    retained: PathBuf,
    real: FsSnapshotIo,
    removed: bool,
}
impl SnapshotIo for SwapRoot {
    fn create_staged(&mut self, path: &Path) -> io::Result<File> {
        self.real.create_staged(path)
    }
    fn write_staged(&mut self, file: &mut File, data: &[u8]) -> io::Result<()> {
        self.real.write_staged(file, data)
    }
    fn sync_staged(&mut self, file: &File) -> io::Result<()> {
        self.real.sync_staged(file)
    }
    fn publish(&mut self, staged: &Path, target: &Path) -> io::Result<()> {
        self.real.publish(staged, target)
    }
    fn sync_directory(&mut self, path: &Path) -> io::Result<()> {
        assert_eq!(path, self.root);
        fs::rename(&self.root, &self.retained)?;
        fifo(&self.root);
        self.real.sync_directory(path)
    }
    fn remove_staged(&mut self, path: &Path) -> io::Result<()> {
        self.removed = true;
        self.real.remove_staged(path)
    }
}
#[test]
fn root_to_fifo_at_directory_sync_refuses_after_publication_without_blocking() {
    if let Some(root) = std::env::var_os("RR13_CHILD_SYNC_ROOT") {
        let root = PathBuf::from(root);
        assert!(root.is_absolute() && root.starts_with("/workspace"));
        let retained = root.with_file_name("retained-root");
        let store = SnapshotStore::open(&root).unwrap();
        let mut io = SwapRoot {
            root,
            retained: retained.clone(),
            real: FsSnapshotIo,
            removed: false,
        };
        let document = Document::empty();
        assert!(matches!(
            store.save_new_with_io("candidate", &document, &mut io),
            Err(StoreError::Io(_))
        ));
        assert!(!io.removed);
        for file in ["candidate.emf.json", "candidate.emf.staged"] {
            assert_eq!(
                fs::read(retained.join(file)).unwrap(),
                document.to_json().unwrap().as_bytes()
            );
        }
        assert!(
            fs::symlink_metadata(&io.root)
                .unwrap()
                .file_type()
                .is_fifo()
        );
        return;
    }
    let dir = directory();
    child_case(
        "root_to_fifo_at_directory_sync_refuses_after_publication_without_blocking",
        "RR13_CHILD_SYNC_ROOT",
        &dir.join("root"),
    );
}
