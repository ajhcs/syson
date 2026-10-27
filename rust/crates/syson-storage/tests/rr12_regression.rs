// SPDX-License-Identifier: EPL-2.0
//! Allocation accounting is confined to this development-only test executable.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};
use syson_model::Document;
use syson_storage::{Change, Error, Key, Store};

#[derive(Clone, Copy, Debug, Default)]
struct Allocations {
    live: isize,
    peak: isize,
    largest: usize,
}
thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static STATS: Cell<Allocations> = const { Cell::new(Allocations { live: 0, peak: 0, largest: 0 }) };
}
fn record(change: isize, requested: usize) {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = STATS.try_with(|cell| {
            let mut stats = cell.get();
            stats.live += change;
            stats.peak = stats.peak.max(stats.live);
            stats.largest = stats.largest.max(requested);
            cell.set(stats);
        });
    }
}
struct TrackingAllocator;
// SAFETY: All allocation operations preserve System's pointer/layout contract;
// constant thread-local bookkeeping allocates nothing and tolerates teardown.
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record(layout.size() as isize, layout.size());
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            record(layout.size() as isize, layout.size());
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        record(-(layout.size() as isize), 0);
        unsafe { System.dealloc(pointer, layout) };
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let next = unsafe { System.realloc(pointer, layout, size) };
        if !next.is_null() {
            record(size as isize - layout.size() as isize, size);
        }
        next
    }
}
#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;
static NEXT: AtomicU64 = AtomicU64::new(0);
fn path() -> PathBuf {
    let root = PathBuf::from(std::env::var_os("TMPDIR").expect("owned workspace TMPDIR required"));
    assert!(root.is_absolute() && root.starts_with("/workspace"));
    let directory = root.join(format!(
        "rr12-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    directory.join("store.sqlite3")
}
fn key(id: u8) -> Key {
    Key {
        project: [1; 16],
        document: [id; 16],
    }
}
fn create(store: &Store, name: &str) {
    store
        .transact(
            &[Change::Create {
                key: key(1),
                name,
                document: &Document::empty(),
                read_only: false,
            }],
            1_048_576,
        )
        .unwrap();
}
fn seed(path: &Path, mode: &str) {
    let script = r#"
import pathlib,signal,sqlite3,sys
signal.alarm(10)
p=pathlib.Path(sys.argv[1]); mode=sys.argv[2]
assert p.is_absolute() and p.is_relative_to('/workspace') and '..' not in p.parts
assert p.is_file() and not p.is_symlink()
with sqlite3.connect(p.as_uri()+'?mode=rw',uri=True) as db:
    db.execute('PRAGMA ignore_check_constraints=ON')
    cases={
        'huge-name':('name','X'*(2*1024*1024)),
        'name-one-over':('name','X'*4097),
        'name-utf8-over':('name','é'*2049),
        'name-blob':('name',b'X'),
        'content-one-over':('content','X'*(1024*1024+1)),
        'content-blob':('content',b'{}'),
        'flag-corrupt':('read_only',3),
        'revision-corrupt':('revision',-1),
    }
    if mode in cases:
        field,value=cases[mode]
        db.execute(f'UPDATE f27_documents SET {field}=? WHERE project=? AND document=?', [value,'01'*16,'01'*16])
    elif mode=='large-generation':
        db.execute('UPDATE f27_generation SET value=zeroblob(20000) WHERE singleton=1')
    elif mode=='large-schema':
        db.execute("CREATE TABLE rr12_foreign(value TEXT DEFAULT '"+'X'*20000+"')")
    elif mode=='max-escaped':
        db.execute('UPDATE f27_documents SET name=?,content=?', ['\x00'*4096,'\x00'*(1024*1024)])
    else: raise AssertionError('unknown fixed fixture mode')
"#;
    assert!(
        Command::new("/usr/bin/python3")
            .args(["-I", "-S", "-B", "-c", script])
            .arg(path)
            .arg(mode)
            .env_clear()
            .status()
            .unwrap()
            .success()
    );
}
#[test]
fn oversized_borrowed_update_name_refuses_before_a_proportional_copy() {
    let file = path();
    let store = Store::create(&file).unwrap();
    create(&store, "original");
    let before = store.load(key(1)).unwrap().unwrap();
    let oversized = "X".repeat(2 * 1024 * 1024);
    let document = Document::empty();
    let bytes = fs::read(&file).unwrap();
    let changes = [Change::Update {
        before: &before,
        name: &oversized,
        document: &document,
    }];
    STATS.set(Allocations::default());
    TRACK.set(true);
    let result = store.transact(&changes, 1_048_576);
    TRACK.set(false);
    let stats = STATS.get();
    assert_eq!(result, Err(Error::Limit));
    assert!(stats.largest <= 65_536 && stats.peak <= 65_536, "{stats:?}");
    assert_eq!(stats.live, 0, "{stats:?}");
    assert_eq!(fs::read(&file).unwrap(), bytes);
}
#[test]
fn fresh_connection_refuses_an_oversized_stored_name_without_mutation() {
    let file = path();
    let store = Store::create(&file).unwrap();
    create(&store, "original");
    seed(&file, "huge-name");
    let bytes = fs::read(&file).unwrap();
    let reopened = Store::open(&file).unwrap();
    assert!(
        matches!(reopened.load(key(1)), Err(Error::Refused(code)) if code == "limit"),
        "oversized persisted metadata was admitted"
    );
    assert_eq!(fs::read(&file).unwrap(), bytes);
}

#[test]
fn exact_utf8_name_bound_roundtrips_while_one_extra_byte_refuses_atomically() {
    for name in ["X".repeat(4096), "é".repeat(2048)] {
        let file = path();
        let store = Store::create(&file).unwrap();
        create(&store, &name);
        let before = store.load(key(1)).unwrap().unwrap();
        assert_eq!(before.name(), name);
        assert_eq!(before.name().len(), 4096);
        let next = if name.starts_with('X') {
            "é".repeat(2048)
        } else {
            "X".repeat(4096)
        };
        store
            .transact(
                &[Change::Update {
                    before: &before,
                    name: &next,
                    document: &Document::empty(),
                }],
                1_048_576,
            )
            .unwrap();
        let before = store.load(key(1)).unwrap().unwrap();
        assert_eq!(before.name(), next);
        assert!(before.revision() > 1);
        for oversized in ["X".repeat(4097), "é".repeat(2048) + "x"] {
            let bytes = fs::read(&file).unwrap();
            assert_eq!(
                store.transact(
                    &[
                        Change::Create {
                            key: key(2),
                            name: "earlier valid operation",
                            document: &Document::empty(),
                            read_only: false
                        },
                        Change::Update {
                            before: &before,
                            name: &oversized,
                            document: &Document::empty()
                        },
                    ],
                    1_048_576
                ),
                Err(Error::Limit)
            );
            assert_eq!(
                store.transact(
                    &[Change::Create {
                        key: key(2),
                        name: &oversized,
                        document: &Document::empty(),
                        read_only: false
                    }],
                    1_048_576
                ),
                Err(Error::Limit)
            );
            assert_eq!(fs::read(&file).unwrap(), bytes);
            assert!(store.load(key(2)).unwrap().is_none());
            assert_eq!(
                store.load(key(1)).unwrap().unwrap().revision(),
                before.revision()
            );
        }
    }
}
#[test]
fn maximum_valid_document_and_name_fit_the_connection_record_limit() {
    let mut value = Document::empty().value().clone();
    value["vendor"] = serde_json::Value::String(String::new());
    let fixed = value.to_string().len();
    value["vendor"] = serde_json::Value::String("X".repeat(1_048_576 - fixed));
    let document = Document::from_json(&value.to_string()).unwrap();
    assert_eq!(document.value().to_string().len(), 1_048_576);
    let file = path();
    let store = Store::create(&file).unwrap();
    let name = "轮".repeat(1365) + "x";
    assert_eq!(name.len(), 4096);
    store
        .transact(
            &[Change::Create {
                key: key(1),
                name: &name,
                document: &document,
                read_only: false,
            }],
            1_048_576,
        )
        .unwrap();
    let restored = Store::open(&file).unwrap().load(key(1)).unwrap().unwrap();
    assert_eq!(restored.name(), name);
    assert_eq!(restored.document().value(), document.value());
}
const PROBE: &str = r#"
import json,pathlib,signal,sqlite3,sys,tracemalloc
signal.alarm(10)
filename,source=sys.argv[1:]
p=pathlib.Path(filename)
assert p.is_absolute() and p.is_relative_to('/workspace') and '..' not in p.parts
assert p.is_file() and not p.is_symlink()
original=sqlite3.connect
observation={'payload_selects':0,'largest_returned_string_chars':0}
def inspect(row):
    if row is not None:
        for value in row:
            if isinstance(value,str):
                observation['largest_returned_string_chars']=max(observation['largest_returned_string_chars'],len(value))
class Cursor:
    def __init__(self,cursor): self.cursor=cursor
    def fetchone(self):
        row=self.cursor.fetchone(); inspect(row); return row
    def fetchall(self):
        rows=self.cursor.fetchall()
        for row in rows: inspect(row)
        return rows
class Connection:
    def __init__(self,db): self.db=db
    def __enter__(self): self.db.__enter__(); return self
    def __exit__(self,*args): return self.db.__exit__(*args)
    def setlimit(self,*args): return self.db.setlimit(*args)
    def execute(self,statement,*args):
        if statement.startswith('SELECT name, content'): observation['payload_selects']+=1
        return Cursor(self.db.execute(statement,*args))
def connect(*args,**kwargs): return Connection(original(*args,**kwargs))
sqlite3.connect=connect
sys.argv=[sys.argv[0],filename]
code=compile(source,'<exact-owned-rr12-bridge>','exec')
# Whole bridge protocol phase includes its existing read(LIMIT+1) input buffer.
tracemalloc.start()
exec(code,{'__name__':'__main__'})
current,peak=tracemalloc.get_traced_memory(); tracemalloc.stop()
observation['python_peak_bytes']=peak
print(json.dumps(observation),file=sys.stderr)
"#;
fn epoch(file: &Path) -> String {
    let script = "import pathlib,signal,sqlite3,sys; signal.alarm(10); p=pathlib.Path(sys.argv[1]); assert p.is_relative_to('/workspace') and p.is_absolute() and not p.is_symlink(); db=sqlite3.connect(p.as_uri()+'?mode=ro',uri=True); print(db.execute('SELECT lower(hex(value)) FROM f27_generation WHERE singleton=1').fetchone()[0]); db.close()";
    let output = Command::new("/usr/bin/python3")
        .args(["-I", "-S", "-B", "-c", script])
        .arg(file)
        .env_clear()
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().into()
}
fn probe(file: &Path, load: bool) -> (serde_json::Value, serde_json::Value, usize) {
    assert!(file.is_absolute() && file.starts_with("/workspace"));
    let request = serde_json::json!({
        "action": if load { "load" } else { "open" },
        "generation": if load { Some(epoch(file)) } else { None },
        "value": if load { serde_json::json!(["01".repeat(16),"01".repeat(16)]) } else { serde_json::Value::Null },
    });
    let mut child = Command::new("/usr/bin/python3")
        .args(["-I", "-S", "-B", "-c", PROBE])
        .arg(file)
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
    (
        serde_json::from_slice(&output.stdout).unwrap(),
        serde_json::from_slice(&output.stderr).unwrap(),
        output.stdout.len(),
    )
}
#[test]
fn row_type_and_byte_preflight_refuses_before_returning_large_fields_to_python() {
    for (mode, expected) in [
        ("name-one-over", "limit"),
        ("name-utf8-over", "limit"),
        ("huge-name", "limit"),
        ("name-blob", "format"),
        ("content-one-over", "limit"),
        ("content-blob", "format"),
        ("flag-corrupt", "format"),
        ("revision-corrupt", "format"),
    ] {
        let file = path();
        let store = Store::create(&file).unwrap();
        create(&store, "original");
        seed(&file, mode);
        let bytes = fs::read(&file).unwrap();
        let (result, observed, written) = probe(&file, true);
        assert_eq!(result, serde_json::json!({"error":expected}), "{mode}");
        assert_eq!(observed["payload_selects"], 0, "{mode}: {observed}");
        assert!(
            observed["largest_returned_string_chars"].as_u64().unwrap() <= 1024,
            "{mode}: {observed}"
        );
        assert!(
            observed["python_peak_bytes"].as_u64().unwrap() <= 9 * 1024 * 1024,
            "{mode}: {observed}"
        );
        assert!(written <= 128);
        assert_eq!(fs::read(&file).unwrap(), bytes);
    }
}
#[test]
fn generation_and_schema_strings_are_bounded_before_crossing_the_driver_boundary() {
    for mode in ["large-generation", "large-schema"] {
        let file = path();
        let store = Store::create(&file).unwrap();
        create(&store, "original");
        seed(&file, mode);
        let bytes = fs::read(&file).unwrap();
        let (result, observed, written) = probe(&file, false);
        assert_eq!(result, serde_json::json!({"error":"format"}));
        assert!(
            observed["largest_returned_string_chars"].as_u64().unwrap() <= 1024,
            "{observed}"
        );
        assert!(
            observed["python_peak_bytes"].as_u64().unwrap() <= 9 * 1024 * 1024,
            "{observed}"
        );
        assert!(written <= 128);
        assert_eq!(fs::read(&file).unwrap(), bytes);
    }
}
#[test]
fn maximal_json_escape_response_stays_below_protocol_limit_with_bounded_python_allocations() {
    let file = path();
    let store = Store::create(&file).unwrap();
    create(&store, "original");
    // These raw NULs make invalid model JSON; this case tests bridge admission
    // and response allocation only, never successful Document loading.
    seed(&file, "max-escaped");
    let bytes = fs::read(&file).unwrap();
    let (result, observed, written) = probe(&file, true);
    assert_eq!(result["row"][0].as_str().unwrap().len(), 4096);
    assert_eq!(result["row"][1].as_str().unwrap().len(), 1_048_576);
    assert_eq!(observed["payload_selects"], 1);
    assert!(written > 6_000_000 && written <= 8 * 1024 * 1024);
    assert!(
        observed["python_peak_bytes"].as_u64().unwrap() <= 32 * 1024 * 1024,
        "{observed}"
    );
    assert_eq!(fs::read(&file).unwrap(), bytes);
}
#[test]
fn external_utf16_v2_database_refuses_unchanged_including_epoch_renewal() {
    let file = path();
    let store = Store::create(&file).unwrap();
    create(&store, "original");
    let utf16 = file.with_file_name("utf16.sqlite3");
    let script = r#"
import pathlib,signal,sqlite3,sys
signal.alarm(10)
source,target=map(pathlib.Path,sys.argv[1:])
for p in (source,target):
    assert p.is_absolute() and p.is_relative_to('/workspace') and '..' not in p.parts and not p.is_symlink()
assert source.is_file() and not target.exists()
s=sqlite3.connect(source.as_uri()+'?mode=ro',uri=True)
d=sqlite3.connect(target.as_uri()+'?mode=rwc',uri=True)
d.execute("PRAGMA encoding='UTF-16le'")
for table,columns in [('f27_clock',2),('f27_documents',6),('f27_generation',2)]:
    sql=s.execute('SELECT sql FROM sqlite_master WHERE name=?',[table]).fetchone()[0]
    d.execute(sql)
    d.executemany('INSERT INTO '+table+' VALUES('+','.join('?'*columns)+')',s.execute('SELECT * FROM '+table).fetchall())
d.execute(f'PRAGMA application_id={0x46323731}')
d.execute('PRAGMA user_version=2'); d.commit()
assert d.execute('PRAGMA encoding').fetchone()[0]=='UTF-16le'
s.close(); d.close()
"#;
    assert!(
        Command::new("/usr/bin/python3")
            .args(["-I", "-S", "-B", "-c", script])
            .arg(&file)
            .arg(&utf16)
            .env_clear()
            .status()
            .unwrap()
            .success()
    );
    let bytes = fs::read(&utf16).unwrap();
    assert!(matches!(Store::open(&utf16), Err(Error::Refused(code)) if code == "format"));
    assert!(matches!(Store::open_restored(&utf16), Err(Error::Refused(code)) if code == "format"));
    assert_eq!(fs::read(&utf16).unwrap(), bytes);
}
