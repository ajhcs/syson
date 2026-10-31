// SPDX-License-Identifier: EPL-2.0
//! Deterministic failures against real, isolated filesystem snapshots.
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use syson_model::{Document, Editor, NodeIds};
use syson_workflow::{FsSnapshotIo, SnapshotIo, SnapshotStore, StoreError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operation {
    Create,
    Write,
    SyncFile,
    Publish,
    SyncDirectory,
    Remove,
}
#[derive(Clone, Copy)]
enum Fault {
    Before(Operation),
    PartialWrite,
    AfterPublish,
    AfterRemove,
}
struct FaultIo {
    fault: Option<Fault>,
    calls: Vec<Operation>,
    real: FsSnapshotIo,
}
impl FaultIo {
    fn new(fault: Option<Fault>) -> Self {
        Self {
            fault,
            calls: Vec::new(),
            real: FsSnapshotIo,
        }
    }
    fn enter(&mut self, operation: Operation) -> io::Result<()> {
        self.calls.push(operation);
        if matches!(self.fault, Some(Fault::Before(expected)) if expected == operation) {
            return Err(injected());
        }
        Ok(())
    }
}
fn injected() -> io::Error {
    io::Error::other("F31 controlled failure")
}
impl SnapshotIo for FaultIo {
    fn create_staged(&mut self, path: &Path) -> io::Result<File> {
        self.enter(Operation::Create)?;
        self.real.create_staged(path)
    }
    fn write_staged(&mut self, file: &mut File, data: &[u8]) -> io::Result<()> {
        self.enter(Operation::Write)?;
        if matches!(self.fault, Some(Fault::PartialWrite)) {
            file.write_all(&data[..data.len().min(11)])?;
            return Err(injected());
        }
        self.real.write_staged(file, data)
    }
    fn sync_staged(&mut self, file: &File) -> io::Result<()> {
        self.enter(Operation::SyncFile)?;
        self.real.sync_staged(file)
    }
    fn publish(&mut self, staged: &Path, destination: &Path) -> io::Result<()> {
        self.enter(Operation::Publish)?;
        self.real.publish(staged, destination)?;
        if matches!(self.fault, Some(Fault::AfterPublish)) {
            return Err(injected());
        }
        Ok(())
    }
    fn sync_directory(&mut self, directory: &Path) -> io::Result<()> {
        self.enter(Operation::SyncDirectory)?;
        self.real.sync_directory(directory)
    }
    fn remove_staged(&mut self, staged: &Path) -> io::Result<()> {
        self.enter(Operation::Remove)?;
        self.real.remove_staged(staged)?;
        if matches!(self.fault, Some(Fault::AfterRemove)) {
            return Err(injected());
        }
        Ok(())
    }
}
fn document() -> Document {
    let mut editor = Editor::new(Document::empty());
    let mut tx = editor.begin();
    tx.create_package(
        None,
        NodeIds::new(
            "00000000-0000-4000-8000-000000000001",
            "10000000-0000-4000-8000-000000000001",
        )
        .unwrap(),
        None,
        "F31",
    )
    .unwrap();
    editor.commit(tx).unwrap();
    editor.document().clone()
}
struct Fixture {
    root: PathBuf,
    original: Vec<u8>,
    document: Document,
}
impl Fixture {
    fn new(label: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let base = PathBuf::from(
            std::env::var_os("SYSON_TEST_DIR")
                .expect("SYSON_TEST_DIR must name the approved workspace test directory"),
        )
        .canonicalize()
        .unwrap();
        assert!(
            base.starts_with("/workspace"),
            "No root-disk fixture fallback"
        );
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = base.join(format!(
            "f31-{label}-{}-{tick}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let store = SnapshotStore::open(&root).unwrap();
        store.save_new("original", &Document::empty()).unwrap();
        let original = fs::read(root.join("original.emf.json")).unwrap();
        Self {
            root,
            original,
            document: document(),
        }
    }
    fn staged(&self) -> PathBuf {
        self.root.join("candidate.emf.staged")
    }
    fn destination(&self) -> PathBuf {
        self.root.join("candidate.emf.json")
    }
    fn bytes(&self) -> Vec<u8> {
        self.document.to_json().unwrap().into_bytes()
    }
    fn store(&self) -> SnapshotStore {
        SnapshotStore::open(&self.root).unwrap()
    }
    fn original_unchanged(&self) {
        assert_eq!(
            fs::read(self.root.join("original.emf.json")).unwrap(),
            self.original
        );
        assert_eq!(self.store().load("original").unwrap(), Document::empty());
    }
    fn save(&self, fault: Option<Fault>) -> (Result<PathBuf, StoreError>, FaultIo) {
        let mut io = FaultIo::new(fault);
        let result = self
            .store()
            .save_new_with_io("candidate", &self.document, &mut io);
        (result, io)
    }
    fn record(&self, result: &str) {
        self.original_unchanged();
        fs::write(self.root.join("observed.txt"), result).unwrap();
        println!("F31 retained fixture: {} ({result})", self.root.display());
    }
}
fn io_error<T: std::fmt::Debug>(result: Result<T, StoreError>) {
    assert!(matches!(result, Err(StoreError::Io(_))), "{result:?}");
}
fn through(last: Operation) -> Vec<Operation> {
    let mut order = vec![
        Operation::Create,
        Operation::Write,
        Operation::SyncFile,
        Operation::Publish,
    ];
    #[cfg(unix)]
    order.push(Operation::SyncDirectory);
    order.push(Operation::Remove);
    order.truncate(order.iter().position(|&v| v == last).unwrap() + 1);
    order
}
#[test]
fn successful_sequence_loads_from_fresh_store_and_removes_staging() {
    let f = Fixture::new("success");
    let (result, io) = f.save(None);
    assert_eq!(result.unwrap(), f.destination());
    assert_eq!(io.calls, through(Operation::Remove));
    assert!(!f.staged().exists());
    assert_eq!(f.store().load("candidate").unwrap(), f.document);
    f.record("published; full sequence returned success; fresh load matches; no staging");
}
fn before_publish_failure(operation: Operation, staged_bytes: Option<Vec<u8>>) {
    let f = Fixture::new(&format!("before-{operation:?}"));
    let (result, io) = f.save(Some(Fault::Before(operation)));
    io_error(result);
    assert_eq!(io.calls, through(operation));
    assert!(!f.destination().exists());
    assert!(f.store().load("candidate").is_err());
    if let Some(bytes) = staged_bytes {
        assert_eq!(fs::read(f.staged()).unwrap(), bytes);
    } else {
        assert!(!f.staged().exists());
    }
    f.record("unpublished; original preserved; any staged bytes retained");
}
#[test]
fn create_failure_leaves_no_candidate() {
    before_publish_failure(Operation::Create, None);
}
#[test]
fn write_failure_retains_empty_staging() {
    before_publish_failure(Operation::Write, Some(Vec::new()));
}
#[test]
fn file_sync_failure_retains_complete_staging_without_publishing() {
    before_publish_failure(
        Operation::SyncFile,
        Some(document().to_json().unwrap().into_bytes()),
    );
}
#[test]
fn publication_failure_retains_complete_staging_without_publishing() {
    before_publish_failure(
        Operation::Publish,
        Some(document().to_json().unwrap().into_bytes()),
    );
}
#[test]
fn partial_write_is_retained_and_never_loaded_as_a_snapshot() {
    let f = Fixture::new("partial-write");
    let (result, io) = f.save(Some(Fault::PartialWrite));
    io_error(result);
    assert_eq!(io.calls, through(Operation::Write));
    assert_eq!(fs::read(f.staged()).unwrap(), f.bytes()[..11]);
    assert!(!f.destination().exists());
    assert!(f.store().load("candidate").is_err());
    f.record("unpublished; exact partial-write prefix retained");
}
fn published_error(fault: Fault, last: Operation, label: &str) {
    let f = Fixture::new(label);
    let (result, io) = f.save(Some(fault));
    io_error(result);
    assert_eq!(io.calls, through(last));
    assert_eq!(fs::read(f.destination()).unwrap(), f.bytes());
    assert_eq!(fs::read(f.staged()).unwrap(), f.bytes());
    assert_eq!(f.store().load("candidate").unwrap(), f.document);
    let before_retry = fs::read(f.destination()).unwrap();
    let retry = f.store().save_new("candidate", &Document::empty());
    assert!(
        matches!(retry, Err(StoreError::Io(ref e)) if e.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(fs::read(f.destination()).unwrap(), before_retry);
    assert_eq!(fs::read(f.staged()).unwrap(), f.bytes());
    f.record("published despite Io error; fresh load succeeds; durability unproven; retry collides; staging retained");
}
#[test]
fn publish_side_effect_then_error_makes_failure_ambiguous() {
    published_error(Fault::AfterPublish, Operation::Publish, "after-publish");
}
#[cfg(unix)]
#[test]
fn directory_sync_error_is_a_published_but_unconfirmed_outcome() {
    published_error(
        Fault::Before(Operation::SyncDirectory),
        Operation::SyncDirectory,
        "directory-sync",
    );
}
#[test]
fn staging_removal_error_retains_published_snapshot_and_staging() {
    published_error(
        Fault::Before(Operation::Remove),
        Operation::Remove,
        "remove-staged",
    );
}
#[test]
fn existing_published_snapshot_is_not_replaced_by_retry() {
    let f = Fixture::new("existing-destination");
    f.store().save_new("candidate", &Document::empty()).unwrap();
    let original = fs::read(f.destination()).unwrap();
    let (result, io) = f.save(None);
    assert!(
        matches!(result, Err(StoreError::Io(ref e)) if e.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(io.calls, through(Operation::Publish));
    assert_eq!(fs::read(f.destination()).unwrap(), original);
    assert_eq!(f.store().load("candidate").unwrap(), Document::empty());
    assert_eq!(fs::read(f.staged()).unwrap(), f.bytes());
    f.record("existing destination preserved; new complete staged bytes retained after no-replace collision");
}
#[test]
fn existing_staging_is_preserved_and_not_used_as_a_snapshot() {
    let f = Fixture::new("existing-staging");
    let sentinel = b"unrelated retained staged bytes";
    fs::write(f.staged(), sentinel).unwrap();
    let (result, io) = f.save(None);
    assert!(
        matches!(result, Err(StoreError::Io(ref e)) if e.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(io.calls, through(Operation::Create));
    assert_eq!(fs::read(f.staged()).unwrap(), sentinel);
    assert!(!f.destination().exists());
    assert!(f.store().load("candidate").is_err());
    f.record("existing staging preserved byte-for-byte; candidate unpublished");
}
#[test]
fn invalid_keys_are_rejected_before_any_injected_io() {
    let f = Fixture::new("invalid-keys");
    for key in ["", "../escape", "a/b", "a\\b", ".", &"a".repeat(81)] {
        let mut io = FaultIo::new(None);
        assert!(matches!(
            f.store().save_new_with_io(key, &f.document, &mut io),
            Err(StoreError::InvalidKey)
        ));
        assert!(io.calls.is_empty());
    }
    assert!(!f.destination().exists());
    assert!(!f.staged().exists());
    f.record("invalid keys rejected without snapshot I/O");
}

#[test]
fn removal_side_effect_then_error_leaves_no_staging_but_a_published_snapshot() {
    let f = Fixture::new("after-remove");
    let (result, io) = f.save(Some(Fault::AfterRemove));
    io_error(result);
    assert_eq!(io.calls, through(Operation::Remove));
    assert!(!f.staged().exists());
    let saved = fs::read(f.destination()).unwrap();
    assert_eq!(saved, f.bytes());
    assert_eq!(f.store().load("candidate").unwrap(), f.document);
    let retry_document = Document::empty();
    let retry = f.store().save_new("candidate", &retry_document);
    assert!(
        matches!(retry, Err(StoreError::Io(ref e)) if e.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(fs::read(f.destination()).unwrap(), saved);
    assert_eq!(f.store().load("candidate").unwrap(), f.document);
    assert_eq!(
        fs::read(f.staged()).unwrap(),
        retry_document.to_json().unwrap().as_bytes()
    );
    f.record("published despite Io after removal; original staging absent; retry collision preserves destination and new staging; durability unproven");
}
