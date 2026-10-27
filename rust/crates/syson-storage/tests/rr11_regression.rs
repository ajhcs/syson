// SPDX-License-Identifier: EPL-2.0
use serde_json::json;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};
use syson_model::Document;
use syson_storage::{Change, Error, Key, Store};

static NEXT: AtomicU64 = AtomicU64::new(0);
fn directory() -> PathBuf {
    let root = PathBuf::from(std::env::var_os("TMPDIR").expect("workspace TMPDIR required"));
    assert!(root.is_absolute() && root.starts_with("/workspace"));
    let path = root.join(format!(
        "rr11-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    path
}
fn key(id: u8) -> Key {
    Key {
        project: [1; 16],
        document: [id; 16],
    }
}
fn document(name: &str) -> Document {
    Document::from_json(
        &json!({
            "json":{"version":"1.0"},
            "ns":{"sysml":"http://www.eclipse.org/syson/sysml"},
            "content":[{"id":"10000000-0000-4000-8000-000000000001", "eClass":"sysml:Package",
                "data":{"elementId":"20000000-0000-4000-8000-000000000001", "declaredName":name}}]
        })
        .to_string(),
    )
    .unwrap()
}
fn create(store: &Store, id: u8, content: &Document) {
    store
        .transact(
            &[Change::Create {
                key: key(id),
                name: "document",
                document: content,
                read_only: false,
            }],
            1_048_576,
        )
        .unwrap();
}
fn replacement_refuses(delete: bool) {
    let dir = directory();
    let a = dir.join("a.sqlite3");
    let b = dir.join("b.sqlite3");
    let store_a = Store::create(&a).unwrap();
    let store_b = Store::create(&b).unwrap();
    let content_a = document("original A");
    let content_b = document("replacement B");
    create(&store_a, 1, &content_a);
    create(&store_b, 1, &content_b);
    let before = store_a.load(key(1)).unwrap().unwrap();
    assert_eq!(
        before.revision(),
        store_b.load(key(1)).unwrap().unwrap().revision()
    );
    // Preserve both valid databases; only replace this owned pathname.
    fs::rename(&a, dir.join("retained-a.sqlite3")).unwrap();
    fs::rename(&b, &a).unwrap();
    let replacement_bytes = fs::read(&a).unwrap();
    let stale = if delete {
        Change::Delete { before: &before }
    } else {
        Change::Update {
            before: &before,
            name: "wrong database",
            document: &content_a,
        }
    };
    let result = store_a.transact(
        &[
            Change::Create {
                key: key(2),
                name: "must not commit",
                document: &content_a,
                read_only: false,
            },
            stale,
        ],
        1_048_576,
    );
    assert!(
        result.is_err(),
        "an observation from A authorized mutation of replacement B"
    );
    assert_eq!(fs::read(&a).unwrap(), replacement_bytes);
    let reopened = Store::open(&a).unwrap();
    assert_eq!(
        reopened.load(key(1)).unwrap().unwrap().document().value(),
        content_b.value()
    );
    assert!(reopened.load(key(2)).unwrap().is_none());
}
#[test]
fn replaced_database_refuses_stale_update_and_rolls_back_earlier_create() {
    replacement_refuses(false);
}
#[test]
fn replaced_database_refuses_stale_delete_and_rolls_back_earlier_create() {
    replacement_refuses(true);
}

fn owned(path: &Path) {
    assert!(path.is_absolute() && path.starts_with("/workspace"));
    assert!(
        !path
            .components()
            .any(|c| c == std::path::Component::ParentDir)
    );
}
fn sql(path: &Path, script: &str) -> String {
    owned(path);
    let script = format!("import signal; signal.alarm(10); {script}");
    let output = Command::new("/usr/bin/python3")
        .args(["-I", "-S", "-B", "-c", &script])
        .arg(path)
        .env_clear()
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
fn epoch(path: &Path) -> String {
    sql(
        path,
        "import pathlib,sqlite3,sys; db=sqlite3.connect(pathlib.Path(sys.argv[1]).as_uri()+'?mode=ro',uri=True); print(db.execute('SELECT lower(hex(value)) FROM f27_generation WHERE singleton=1').fetchone()[0]); db.close()",
    )
}
fn generation_refusal<T>(result: Result<T, Error>) {
    assert!(matches!(result, Err(Error::Refused(code)) if code == "generation"));
}
#[test]
fn healthy_reopen_keeps_epoch_and_existing_handle_but_not_foreign_observations() {
    let path = directory().join("store.sqlite3");
    let first = Store::create(&path).unwrap();
    create(&first, 1, &document("initial"));
    let before = first.load(key(1)).unwrap().unwrap();
    let original_epoch = epoch(&path);
    let second = Store::open(&path).unwrap();
    assert_eq!(epoch(&path), original_epoch);
    assert!(matches!(
        second.transact(&[Change::Delete { before: &before }], 1_048_576),
        Err(Error::ForeignObservation)
    ));
    first
        .transact(
            &[Change::Update {
                before: &before,
                name: "healthy",
                document: &document("updated"),
            }],
            1_048_576,
        )
        .unwrap();
    assert_eq!(second.load(key(1)).unwrap().unwrap().name(), "healthy");
}
#[test]
fn offline_restored_snapshot_requires_new_epoch_and_invalidates_create_only_batches() {
    let dir = directory();
    let path = dir.join("live.sqlite3");
    let backup = dir.join("backup.sqlite3");
    let old = Store::create(&path).unwrap();
    let initial = document("snapshot");
    create(&old, 1, &initial);
    let before = old.load(key(1)).unwrap().unwrap();
    let original_epoch = epoch(&path);
    fs::copy(&path, &backup).unwrap();
    old.transact(
        &[Change::Update {
            before: &before,
            name: "newer",
            document: &document("newer"),
        }],
        1_048_576,
    )
    .unwrap();
    // No connection remains open; this is a caller-owned offline restore.
    fs::rename(&path, dir.join("retained-newer.sqlite3")).unwrap();
    fs::rename(&backup, &path).unwrap();
    let restored = Store::open_restored(&path).unwrap();
    assert_ne!(epoch(&path), original_epoch);
    let bytes = fs::read(&path).unwrap();
    generation_refusal(old.load(key(1)));
    generation_refusal(old.transact(
        &[Change::Create {
            key: key(2),
            name: "old handle",
            document: &initial,
            read_only: false,
        }],
        1_048_576,
    ));
    generation_refusal(old.transact(&[Change::Delete { before: &before }], 1_048_576));
    assert_eq!(fs::read(&path).unwrap(), bytes);
    let actual = restored.load(key(1)).unwrap().unwrap();
    assert_eq!(actual.document().value(), initial.value());
    assert_eq!(actual.revision(), before.revision());
    create(&restored, 2, &initial);
    assert!(Store::open(&path).unwrap().load(key(2)).unwrap().is_some());
}
#[test]
fn old_format_is_refused_without_implicit_upgrade_or_epoch_renewal() {
    let path = directory().join("v1.sqlite3");
    let store = Store::create(&path).unwrap();
    create(&store, 1, &document("preserved"));
    sql(
        &path,
        "import pathlib,sqlite3,sys; db=sqlite3.connect(pathlib.Path(sys.argv[1]).as_uri()+'?mode=rw',uri=True,isolation_level=None); db.execute('DROP TABLE f27_generation'); db.execute('PRAGMA user_version=1'); db.close()",
    );
    let bytes = fs::read(&path).unwrap();
    assert!(matches!(Store::open(&path), Err(Error::Refused(code)) if code == "format"));
    assert!(matches!(Store::open_restored(&path), Err(Error::Refused(code)) if code == "format"));
    assert_eq!(fs::read(&path).unwrap(), bytes);
}
// Fixed, dev-only interleavings around the exact production bridge. No bridge
// runtime flag, SQL endpoint or production fault-injection hook is introduced.
const WRAPPER: &str = r#"
import pathlib,signal,sqlite3,sys
mode,filename,other,source=sys.argv[1:]
assert mode in {'commit-failure','load-interleave','replace-before-connect'}
for value in (filename,other):
    path=pathlib.Path(value)
    assert path.is_absolute() and '..' not in path.parts and path.is_relative_to('/workspace')
    assert path.is_file() and not path.is_symlink()
def deadline(signum,frame): raise SystemExit('RR11 owned fixture deadline')
signal.signal(signal.SIGALRM,deadline); signal.alarm(10)
original=sqlite3.connect
class Connection:
    def __init__(self,db): self.db=db
    def setlimit(self,*args): return self.db.setlimit(*args)
    def __enter__(self): self.db.__enter__(); return self
    def __exit__(self,*args): return self.db.__exit__(*args)
    def execute(self,statement,*args):
        if mode=='commit-failure' and statement=='COMMIT':
            raise sqlite3.OperationalError('injected commit refusal')
        if mode=='load-interleave' and statement.startswith('SELECT name, content'):
            writer=original(pathlib.Path(filename).as_uri()+'?mode=rw',uri=True,isolation_level=None)
            writer.execute('BEGIN IMMEDIATE')
            writer.execute('UPDATE f27_generation SET value=randomblob(16) WHERE singleton=1')
            writer.execute("UPDATE f27_documents SET name='new epoch row'")
            writer.execute('COMMIT'); writer.close()
        return self.db.execute(statement,*args)
def connect(*args,**kwargs):
    if mode=='replace-before-connect':
        path=pathlib.Path(filename)
        path.rename(path.with_name('retained-opened-a.sqlite3'))
        pathlib.Path(other).rename(path)
    return Connection(original(*args,**kwargs))
sqlite3.connect=connect
sys.argv=[sys.argv[0],filename]
exec(compile(source,'<exact-owned-bridge>','exec'))
"#;
fn bridge(path: &Path, other: &Path, mode: &str, request: serde_json::Value) -> serde_json::Value {
    owned(path);
    owned(other);
    let mut child = Command::new("/usr/bin/python3")
        .args(["-I", "-S", "-B", "-c", WRAPPER, mode])
        .arg(path)
        .arg(other)
        .arg(include_str!("../src/bridge.py"))
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(request.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
fn create_request(generation: &str) -> serde_json::Value {
    json!({"action":"transact", "generation":generation, "value":[{
        "kind":"create", "key":["01".repeat(16), "02".repeat(16)], "name":"must refuse",
        "content":document("race").value().to_string(), "revision":0, "changed":true, "read_only":false
    }]})
}
#[test]
fn replacement_between_path_admission_and_actual_connect_refuses_without_mutation() {
    let dir = directory();
    let a = dir.join("a.sqlite3");
    let b = dir.join("b.sqlite3");
    let store_a = Store::create(&a).unwrap();
    let store_b = Store::create(&b).unwrap();
    create(&store_a, 1, &document("A"));
    create(&store_b, 1, &document("B"));
    let generation_a = epoch(&a);
    let bytes_b = fs::read(&b).unwrap();
    assert_eq!(
        bridge(
            &a,
            &b,
            "replace-before-connect",
            create_request(&generation_a)
        ),
        json!({"error":"generation"})
    );
    assert_eq!(fs::read(&a).unwrap(), bytes_b);
    assert!(Store::open(&a).unwrap().load(key(2)).unwrap().is_none());
}
#[test]
fn failed_epoch_renewal_rolls_back_and_does_not_invalidate_existing_observations() {
    let path = directory().join("store.sqlite3");
    let store = Store::create(&path).unwrap();
    create(&store, 1, &document("original"));
    let before = store.load(key(1)).unwrap().unwrap();
    let generation = epoch(&path);
    let bytes = fs::read(&path).unwrap();
    let result = bridge(
        &path,
        &path,
        "commit-failure",
        json!({"action":"renew_epoch","generation":null,"value":null}),
    );
    assert_eq!(result, json!({"error":"sql"}));
    assert_eq!(epoch(&path), generation);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    store
        .transact(&[Change::Delete { before: &before }], 1_048_576)
        .unwrap();
}
#[test]
fn load_returns_generation_and_row_from_one_snapshot_during_concurrent_renewal() {
    let path = directory().join("store.sqlite3");
    let store = Store::create(&path).unwrap();
    create(&store, 1, &document("original"));
    let generation = epoch(&path);
    sql(
        &path,
        "import pathlib,sqlite3,sys; db=sqlite3.connect(pathlib.Path(sys.argv[1]).as_uri()+'?mode=rw',uri=True,isolation_level=None); assert db.execute('PRAGMA journal_mode=WAL').fetchone()[0]=='wal'; db.close()",
    );
    let result = bridge(
        &path,
        &path,
        "load-interleave",
        json!({"action":"load","generation":generation,"value":["01".repeat(16),"01".repeat(16)]}),
    );
    assert_eq!(result["generation"], generation);
    assert_eq!(result["row"][0], "document");
    generation_refusal(store.load(key(1)));
    assert_eq!(
        Store::open(&path)
            .unwrap()
            .load(key(1))
            .unwrap()
            .unwrap()
            .name(),
        "new epoch row"
    );
}
