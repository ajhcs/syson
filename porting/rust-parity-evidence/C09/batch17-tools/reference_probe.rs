// SPDX-License-Identifier: EPL-2.0
//! External host-only measurement consumer; never compiled inside model's
//! forbid(unsafe_code) library. Root owns its separate Cargo manifest/execution.
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
// SAFETY: Original allocator pointers/layouts reach System unchanged. Constant
// TLS bookkeeping owns no allocation metadata and tolerates thread teardown.
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

use syson_metamodel::ClassifierId;
use syson_references::{FragmentIdentity, Registry, Resource};
fn memory() -> serde_json::Value {
    let text = std::fs::read_to_string("/proc/self/status").unwrap();
    let get = |key: &str| {
        text.lines()
            .find(|s| s.starts_with(key))
            .unwrap()
            .split_whitespace()
            .nth(1)
            .unwrap()
            .parse::<u64>()
            .unwrap()
    };
    serde_json::json!({"rss_kib":get("VmRSS:"),"hwm_kib":get("VmHWM:")})
}
fn fixture(count: usize) -> Document {
    let rows:Vec<_>=(0..count).map(|n|serde_json::json!({"id":format!("10000000-0000-4000-8000-{n:012x}"),"eClass":"sysml:Package","data":{"elementId":format!("20000000-0000-4000-8000-{n:012x}"),"declaredName":format!("Package{n}")}})).collect();
    Document::from_json(&serde_json::json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":rows}).to_string()).unwrap()
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let count: usize = args[1].parse().unwrap();
    assert!([256, 1024, 4096].contains(&count));
    let doc = fixture(count);
    let ids: Vec<_> = [0, count / 2, count - 1]
        .into_iter()
        .map(|n| serde_json::json!(format!("10000000-0000-4000-8000-{n:012x}")))
        .collect();
    let rss_before = memory();
    COUNTS.set(Counts::default());
    TRACK.set(true);
    let started = Instant::now();
    let registry = Registry::new(vec![Resource {
        key: "local".into(),
        document: doc.clone(),
        fragments: FragmentIdentity::SemanticElementId,
    }])
    .unwrap();
    let build_us = started.elapsed().as_micros();
    let built = COUNTS.get();
    let started = Instant::now();
    for _ in 0..64 {
        for wire in &ids {
            let target = registry
                .resolve("local", Some(wire), ClassifierId::Element)
                .unwrap()
                .unwrap();
            assert_eq!(target.node_id(), wire.as_str().unwrap());
            assert_eq!(registry.target(&target).unwrap()["id"], *wire);
        }
    }
    let lookup_us = started.elapsed().as_micros();
    let resolved = COUNTS.get();
    drop(registry);
    let dropped = COUNTS.get();
    TRACK.set(false);
    assert_eq!(dropped.live, 0);
    println!(
        "{}",
        serde_json::json!({"nodes":count,"lookups":192,"build_us":build_us,"lookup_batch_us":lookup_us,"build_requested_bytes":built.requested,"retained_requested_live":built.live,"peak_requested_live":resolved.peak,"lookup_requested_bytes":resolved.requested-built.requested,"drop_live":dropped.live,"rss_before":rss_before,"rss_after":memory(),"scope":"Main-thread requested allocation counters and separate Linux RSS; fixture setup excluded from API timing; no copy-byte/heap/native/UI/cancellation/capacity guarantee."})
    );
}
