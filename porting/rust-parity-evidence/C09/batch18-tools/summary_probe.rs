// SPDX-License-Identifier: EPL-2.0
//! Development-only single-thread public API probe; no GUI/native/capacity claim.
//! Root owns compilation, exact fixture hashing and fresh-process repetitions.
use serde_json::{Value, json};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    time::Instant,
};
use syson_model::Document;
#[derive(Clone, Copy, Default)]
struct Counts {
    live: i64,
    peak: i64,
    requested: u64,
    largest: usize,
    calls: u64,
}
thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static COUNTS: Cell<Counts> = const { Cell::new(Counts {live:0,peak:0,requested:0,largest:0,calls:0}) };
}
fn record(delta: i64, requested: usize) {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = COUNTS.try_with(|cell| {
            let mut c = cell.get();
            c.live += delta;
            c.peak = c.peak.max(c.live);
            c.requested += requested as u64;
            c.largest = c.largest.max(requested);
            c.calls += u64::from(requested != 0);
            cell.set(c);
        });
    }
}
struct Observed;
// SAFETY: System receives unchanged allocator pointers/layouts. Constant TLS
// bookkeeping allocates no metadata and tolerates thread teardown.
unsafe impl GlobalAlloc for Observed {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            record(l.size() as i64, l.size());
        }
        p
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(l) };
        if !p.is_null() {
            record(l.size() as i64, l.size());
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        record(-(l.size() as i64), 0);
        unsafe { System.dealloc(p, l) };
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let q = unsafe { System.realloc(p, l, n) };
        if !q.is_null() {
            record(n as i64 - l.size() as i64, n);
        }
        q
    }
}
#[global_allocator]
static ALLOCATOR: Observed = Observed;
fn memory() -> Value {
    let text = std::fs::read_to_string("/proc/self/status").ok();
    let get = |key: &str| {
        text.as_ref()
            .and_then(|s| s.lines().find(|l| l.starts_with(key)))
            .and_then(|s| s.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u64>().ok())
    };
    json!({"rss_kib":get("VmRSS:"),"hwm_kib":get("VmHWM:")})
}
fn checksum<'a>(rows: impl Iterator<Item = [Option<&'a str>; 5]>) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    let mut feed = |byte| {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    };
    for fields in rows {
        for field in fields {
            feed(u8::from(field.is_some()));
            if let Some(text) = field {
                for byte in (text.len() as u64).to_le_bytes() {
                    feed(byte);
                }
                for byte in text.bytes() {
                    feed(byte);
                }
            }
        }
    }
    hash
}
fn observe<T>(build: impl FnOnce() -> T, inspect: impl FnOnce(&T) -> (usize, u64)) -> Value {
    let before = memory();
    COUNTS.set(Counts::default());
    TRACK.set(true);
    let started = Instant::now();
    let rows = std::hint::black_box(build());
    let elapsed = started.elapsed().as_nanos();
    let built = COUNTS.get();
    TRACK.set(false);
    // Equal field traversal is outside API timing and requested-allocation window.
    let (count, checksum) = std::hint::black_box(inspect(&rows));
    let retained = memory();
    TRACK.set(true);
    drop(rows);
    let dropped = COUNTS.get();
    TRACK.set(false);
    assert_eq!(count, 1025);
    assert_eq!(dropped.live, 0);
    json!({"elapsed_ns":elapsed,"rows":count,"checksum":format!("{checksum:016x}"),
        "requested_bytes":built.requested,"largest_requested_bytes":built.largest,
        "allocation_requests":built.calls,"retained_requested_live":built.live,
        "peak_requested_live":built.peak,"drop_live":dropped.live,
        "memory_before":before,"memory_retained":retained,"memory_after_drop":memory()})
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        3,
        "absolute fixture path and owned|borrowed mode required"
    );
    assert!(std::path::Path::new(&args[1]).is_absolute());
    let input = std::fs::read_to_string(&args[1]).unwrap();
    assert_eq!(
        input.len(),
        192_234,
        "named fixture size; root also verifies exact hash"
    );
    let doc = Document::from_json(&input).unwrap();
    drop(input);
    assert_eq!(
        doc.value()["content"][0]["data"]["elementId"]
            .as_str()
            .unwrap()
            .len(),
        65_542
    );
    let sample = match args[2].as_str() {
        "owned" => observe(
            || doc.elements(),
            |rows| {
                (
                    rows.len(),
                    checksum(rows.iter().map(|r| {
                        [
                            Some(r.resource_id.as_str()),
                            Some(r.element_id.as_str()),
                            Some(r.class.as_str()),
                            r.name.as_deref(),
                            r.owner.as_deref(),
                        ]
                    })),
                )
            },
        ),
        "borrowed" => observe(
            || doc.element_summaries(),
            |rows| {
                (
                    rows.len(),
                    checksum(rows.iter().map(|r| {
                        [
                            Some(r.resource_id),
                            Some(r.element_id),
                            Some(r.class),
                            r.name,
                            r.owner,
                        ]
                    })),
                )
            },
        ),
        _ => panic!("mode must be owned or borrowed"),
    };
    println!(
        "{}",
        json!({"schema":1,"mode":args[2],"sample":sample,
        "scope":"Main-thread requested allocation counters; separate Linux RSS/HWM. Parse, checksum and output formatting excluded from API timer/window. Drop-live zero is a scoped counter observation, not leak proof, heap/RSS bound, GUI latency, native equivalence or general capacity."})
    );
}
