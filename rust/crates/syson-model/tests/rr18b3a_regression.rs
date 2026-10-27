// SPDX-License-Identifier: EPL-2.0
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};
use syson_model::Document;
const DENSE: &str = include_str!(
    "../../../../porting/rust-parity-evidence/RR18b3a/fixtures/ownership-amplification.json"
);
thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static BYTES: Cell<usize> = const { Cell::new(0) };
    static LARGEST: Cell<usize> = const { Cell::new(0) };
}
fn record(size: usize) {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = BYTES.try_with(|v| v.set(v.get().saturating_add(size)));
        let _ = LARGEST.try_with(|v| v.set(v.get().max(size)));
    }
}
struct Observed;
// SAFETY: unchanged pointers/layouts go to System; constant TLS never allocates.
unsafe impl GlobalAlloc for Observed {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            record(layout.size());
        }
        p
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(layout) };
        if !p.is_null() {
            record(layout.size());
        }
        p
    }
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let p = unsafe { System.realloc(p, layout, size) };
        if !p.is_null() {
            record(size);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        unsafe { System.dealloc(p, layout) }
    }
}
#[global_allocator]
static ALLOCATOR: Observed = Observed;
fn observed<T>(operation: impl FnOnce() -> T) -> (T, usize, usize) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TRACK.with(|v| v.set(false));
        }
    }
    BYTES.with(|v| v.set(0));
    LARGEST.with(|v| v.set(0));
    TRACK.with(|v| v.set(true));
    let reset = Reset;
    let value = operation();
    drop(reset);
    (value, BYTES.with(Cell::get), LARGEST.with(Cell::get))
}
#[test]
fn borrowed_rows_follow_manual_preorder_owner_and_class_oracles() {
    let doc = Document::from_json(r#"{
      "json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},
      "opaque":{"$serde_json::private::Number":"7","n":-0},
      "content":[{"id":"rp","eClass":"sysml:Package","data":{"elementId":"P","declaredName":"",
        "ownedRelationship":[{"id":"rm","eClass":"sysml:OwningMembership","data":{"elementId":"M",
          "ownedRelatedElement":[{"id":"ra","eClass":"sysml:PartUsage","data":{"elementId":"A","declaredName":"雪\n",
            "ownedRelationship":[{"id":"rb","eClass":"sysml:OwningMembership","data":{"elementId":"B",
              "ownedRelatedElement":[{"id":"rg","eClass":"Package","data":{"elementId":"G"}}]}}]}},
            {"id":"ru","eClass":"vendor:Ghost","data":{"elementId":"U","declaredName":null}}],
          "ownedRelationship":[{"id":"rn","eClass":"sysml:FeatureTyping","data":{"elementId":"N",
            "ownedRelatedElement":[{"id":"rc","eClass":"sysml:PartDefinition","data":{"elementId":"C","declaredName":"nested"}}]}}]}}],
        "ownedRelatedElement":[{"id":"rd","eClass":"sysml:PartUsage","data":{"elementId":"D","declaredName":42}}]}}]
    }"#).unwrap();
    let before = doc.to_json().unwrap();
    let rows = doc.element_summaries();
    let tuples: Vec<_> = rows
        .iter()
        .map(|r| (r.resource_id, r.element_id, r.class, r.name, r.owner))
        .collect();
    assert_eq!(
        tuples,
        vec![
            ("rp", "P", "Package", Some(""), None),
            ("rd", "D", "PartUsage", None, None),
            ("rm", "M", "OwningMembership", None, None),
            ("ra", "A", "PartUsage", Some("雪\n"), Some("P")),
            ("rb", "B", "OwningMembership", None, None),
            ("rg", "G", "Package", None, Some("A")),
            ("ru", "U", "vendor:Ghost", None, Some("P")),
            ("rn", "N", "FeatureTyping", None, None),
            ("rc", "C", "PartDefinition", Some("nested"), Some("M")),
        ]
    );
    let owned = doc.elements();
    assert_eq!(owned.len(), rows.len());
    for (owned, borrowed) in owned.iter().zip(&rows) {
        assert_eq!(owned.resource_id, borrowed.resource_id);
        assert_eq!(owned.element_id, borrowed.element_id);
        assert_eq!(owned.class, borrowed.class);
        assert_eq!(owned.name.as_deref(), borrowed.name);
        assert_eq!(owned.owner.as_deref(), borrowed.owner);
    }
    let parent = &doc.value()["content"][0];
    assert!(std::ptr::eq(
        rows[0].resource_id.as_ptr(),
        parent["id"].as_str().unwrap().as_ptr()
    ));
    assert!(std::ptr::eq(
        rows[0].class.as_ptr(),
        parent["eClass"].as_str().unwrap()[6..].as_ptr()
    ));
    assert!(std::ptr::eq(
        rows[3].owner.unwrap().as_ptr(),
        rows[0].element_id.as_ptr()
    ));
    assert_eq!(doc.to_json().unwrap(), before);
    assert!(before.contains("\"n\": -0"));
    assert!(before.contains("$serde_json::private::Number"));
}
#[test]
fn dense_long_owner_rows_allocate_only_borrowed_record_storage() {
    let doc = Document::from_json(DENSE).unwrap();
    let before = doc.to_json().unwrap();
    let (rows, requested, largest) = observed(|| doc.element_summaries());
    eprintln!(
        "RR18B3A borrowed rows: requested={requested}; largest={largest}; rows={}",
        rows.len()
    );
    assert_eq!(rows.len(), 1025);
    assert!(
        requested <= 512 * 1024,
        "borrowed summaries copied payloads: {requested}"
    );
    assert!(largest <= 256 * 1024);
    let owner = doc.value()["content"][0]["data"]["elementId"]
        .as_str()
        .unwrap();
    assert_eq!(owner.len(), 65_542);
    let mut children = 0;
    for row in &rows {
        if row.class == "PartUsage" {
            children += 1;
            assert_eq!(row.owner, Some(owner));
            assert!(std::ptr::eq(row.owner.unwrap().as_ptr(), owner.as_ptr()));
        } else {
            assert_eq!(row.owner, None);
        }
    }
    assert_eq!(children, 512);
    // The existing owned API intentionally still owns every field. Compare it
    // outside the borrowed-window measurement; no legacy API policy changed.
    for (owned, borrowed) in doc.elements().iter().zip(&rows) {
        assert_eq!(owned.element_id, borrowed.element_id);
        assert_eq!(owned.owner.as_deref(), borrowed.owner);
    }
    assert_eq!(doc.to_json().unwrap(), before);
}
#[test]
fn empty_borrowed_collection_does_not_allocate() {
    let doc = Document::empty();
    let (rows, requested, largest) = observed(|| doc.element_summaries());
    assert!(rows.is_empty());
    assert_eq!((requested, largest), (0, 0));
}
