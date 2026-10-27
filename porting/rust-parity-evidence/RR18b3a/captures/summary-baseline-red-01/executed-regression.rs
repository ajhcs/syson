// SPDX-License-Identifier: EPL-2.0
//! Real ABI import/render allocation counterexample, not an alternate renderer.
use std::{alloc::{GlobalAlloc, Layout, System}, cell::Cell};
use syson_model::Document;
use syson_web::{dispatch, initialize, input_byte, input_reset, output_len, output_ptr};
const FIXTURE: &str = include_str!("../../../../porting/rust-parity-evidence/RR18b3a/fixtures/ownership-amplification.json");
#[derive(Clone, Copy, Debug, Default)]
struct Allocations { requested: usize, largest: usize, calls: usize }
thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static STATS: Cell<Allocations> = const { Cell::new(Allocations { requested: 0, largest: 0, calls: 0 }) };
}
fn record(bytes: usize) {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = STATS.try_with(|value| {
            let mut stats = value.get();
            stats.requested = stats.requested.saturating_add(bytes);
            stats.largest = stats.largest.max(bytes);
            stats.calls = stats.calls.saturating_add(1);
            value.set(stats);
        });
    }
}
struct Observed;
// SAFETY: delegate unchanged pointer/layout ownership to System. Constant TLS
// bookkeeping allocates nothing. Only successful requests are counted; realloc
// charges its full new request (aggregate traffic, not live memory or RSS).
unsafe impl GlobalAlloc for Observed {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() { record(layout.size()); }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() { record(layout.size()); }
        pointer
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let next = unsafe { System.realloc(pointer, layout, size) };
        if !next.is_null() { record(size); }
        next
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}
#[global_allocator]
static ALLOCATOR: Observed = Observed;
fn observed<T>(operation: impl FnOnce() -> T) -> (T, Allocations) {
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { TRACK.with(|flag| flag.set(false)); } }
    STATS.with(|value| value.set(Allocations::default()));
    TRACK.with(|flag| flag.set(true));
    let reset = Reset;
    let result = operation();
    drop(reset);
    (result, STATS.with(Cell::get))
}
fn html() -> String {
    // SAFETY: synchronous exports on this thread; copy the bytes before any
    // next initialize/dispatch can invalidate the pointer or length.
    let bytes = unsafe { std::slice::from_raw_parts(output_ptr() as *const u8, output_len()) };
    String::from_utf8(bytes.to_vec()).unwrap()
}
fn committed_export(html: &str) -> String {
    let rest = html.split_once("<textarea id=\"export-json\"").unwrap().1;
    rest.split_once('>').unwrap().1.split_once("</textarea>").unwrap().0
        .replace("&quot;", "\"").replace("&#39;", "'").replace("&lt;", "<")
        .replace("&gt;", ">").replace("&amp;", "&")
}
#[test]
fn actual_abi_import_does_not_amplify_long_semantic_owners_into_summary_copies() {
    assert!(FIXTURE.len() < 1_048_576);
    let doc = Document::from_json(FIXTURE).unwrap();
    let expected = doc.to_json().unwrap();
    assert!(expected.len() < 1_048_576);
    assert_eq!(doc.value()["content"][0]["data"]["elementId"].as_str().unwrap().len(), 65_542);
    assert_eq!(doc.value()["content"][0]["data"]["ownedRelationship"].as_array().unwrap().len(), 512);
    let input = serde_json::to_vec(&serde_json::json!({"action":"import","fields":{"json":FIXTURE}})).unwrap();
    assert!(input.len() < 1_048_576);
    initialize(); input_reset();
    for byte in input { input_byte(byte.into()); }
    // Fixture construction, parsing, expected serialization and input transfer
    // are excluded. This is the real import preflight PLUS publication render.
    let (_, stats) = observed(|| dispatch());
    let rendered = html();
    eprintln!("RR18B3A real import: {stats:?}; input_fixture_bytes={}; html_bytes={}", FIXTURE.len(), rendered.len());
    assert!(rendered.len() < 16 * 1_048_576);
    assert!(rendered.contains("Imported a new document."));
    assert!(!rendered.contains("id=\"recovery-export\""));
    assert_eq!(committed_export(&rendered), expected);
    for index in 0..512 {
        assert!(rendered.contains(&format!("data-value=\"child-{index:04}\"")));
    }
    // The old shared summaries alone clone a65,542-byte owner512times per
    // render (>64MiB over two renders).32MiB leaves room for ordinary HTML and
    // existing model/metadata allocations while rejecting that amplification.
    assert!(stats.requested <= 32 * 1_048_576,
        "semantic-owner rendering amplification: requested {} bytes", stats.requested);
}
