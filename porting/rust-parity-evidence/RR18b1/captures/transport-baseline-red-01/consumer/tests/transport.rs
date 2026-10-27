// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use syson_model::{Document, Editor, NodeIds};
use syson_web::layout_controller::Canvas;
thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static LARGEST: Cell<usize> = const { Cell::new(0) };
}
struct Observed;
#[global_allocator]
static ALLOCATOR: Observed = Observed;
fn record(bytes: usize) {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = LARGEST.try_with(|value| value.set(value.get().max(bytes)));
    }
}
unsafe impl GlobalAlloc for Observed {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record(size);
        unsafe { System.realloc(ptr, layout, size) }
    }
}
fn observed<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { TRACK.with(|v| v.set(false)); } }
    LARGEST.with(|v| v.set(0));
    TRACK.with(|v| v.set(true));
    let reset = Reset;
    let result = operation();
    drop(reset);
    (result, LARGEST.with(Cell::get))
}
fn document() -> Document {
    let mut editor = Editor::new(Document::empty());
    let mut tx = editor.begin();
    tx.create_package(None, NodeIds::new("00000000-0000-4000-8000-000000000001", "10000000-0000-4000-8000-000000000001").unwrap(), None, "Vehicle").unwrap();
    editor.commit(tx).unwrap();
    editor.document().clone()
}
fn opaque(doc: &Document, text: &str) -> Document {
    let mut raw: Value = serde_json::from_str(&doc.to_json().unwrap()).unwrap();
    raw["vendor"] = json!({"opaque":text});
    Document::from_json(&raw.to_string()).unwrap()
}
fn request(canvas: &mut Canvas, doc: &Document) -> String {
    let labels = Value::Array(doc.elements().iter().filter(|e| e.class == "Package")
        .map(|e| json!({"id":e.element_id,"width":80.25,"height":24.25})).collect()).to_string();
    canvas.request(doc, &labels, &canvas.revision(), vec![], false).unwrap()
}
fn publish(canvas: &mut Canvas, doc: &Document) {
    let ticket = request(canvas, doc);
    assert!(canvas.compute(&ticket).unwrap());
    assert!(canvas.apply(&ticket, doc).unwrap());
}
#[test]
fn quoted_geometry_refusal_precedes_large_output_allocation_and_keeps_history() {
    let small = document();
    let large = opaque(&small, &"\\".repeat(150_000));
    assert!(large.to_json().unwrap().len() < 1_048_576);
    let mut canvas = Canvas::default();
    publish(&mut canvas, &small);
    let saved = canvas.export(&small);
    let ticket = request(&mut canvas, &large);
    assert!(canvas.compute(&ticket).unwrap());
    let (result, largest) = observed(|| canvas.apply(&ticket, &large));
    assert!(result.unwrap_err().contains("byte budget"));
    assert!(largest <= 65_536, "refused geometry allocated {largest} bytes in one allocation");
    assert!(canvas.phase().is_none());
    assert_eq!(canvas.export(&small), saved);
    assert!(canvas.history(false, &small).unwrap());
    assert!(canvas.export(&small).is_empty());
    assert!(canvas.history(true, &small).unwrap());
    assert_eq!(canvas.export(&small), saved);
}
