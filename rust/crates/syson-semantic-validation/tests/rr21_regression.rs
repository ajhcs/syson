// SPDX-License-Identifier: EPL-2.0
//! Development-only requested-allocation observer; never application code.
use serde_json::{Value, json};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use syson_model::{ContainmentFeature, Document, Editor, NodeIds};
use syson_semantic_validation::{Error, MAX_DOCUMENT_BYTES, NAMESPACE, PART, Report, validate};
use syson_text_import::Importer;
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
fn record(delta: isize, requested: usize) {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = STATS.try_with(|cell| {
            let mut stats = cell.get();
            stats.live += delta;
            stats.peak = stats.peak.max(stats.live);
            stats.largest = stats.largest.max(requested);
            cell.set(stats);
        });
    }
}
struct Observed;
// SAFETY: Original pointers/layouts delegate unchanged to System. Constant TLS
// bookkeeping allocates nothing, tolerates teardown and never changes ownership.
unsafe impl GlobalAlloc for Observed {
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
static ALLOCATOR: Observed = Observed;
fn id(prefix: u8, n: u32) -> String {
    format!("{prefix:08x}-0000-4000-8000-{n:012x}")
}
fn imported(source: &str) -> Document {
    let mut editor = Editor::new(Document::empty());
    let ids = (1..1024).map(|n| NodeIds::new(&id(1, n), &id(2, n)).unwrap());
    Importer::pinned()
        .unwrap()
        .stage(
            editor.begin(),
            source,
            None,
            ids,
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .commit(&mut editor)
        .unwrap();
    editor.document().clone()
}
#[test]
fn dense_opaque_projection_refuses_without_copying_below_the_rr04_output_cap() {
    let editor = Editor::new(imported("package P;"));
    let before = editor.document().clone();
    let parent = editor
        .document()
        .elements()
        .iter()
        .find(|row| row.name.as_deref() == Some("P"))
        .unwrap()
        .element_id
        .clone();
    let mut opaque = Value::Array(vec![Value::Null; 20_000]);
    for _ in 0..32 {
        opaque = Value::Array(vec![opaque]);
    }
    let fragment = json!({"id":id(3,1),"eClass":"sysml:OwningMembership",
        "data":{"elementId":id(4,1),"opaque":opaque}});
    let mut tx = editor.begin();
    tx.attach_preallocated_subtree(&parent, ContainmentFeature::OwnedRelationship, fragment)
        .unwrap();
    let compact = serde_json::to_vec(tx.document().value()).unwrap();
    assert!(compact.len() < 1_048_576);
    let canonical = tx.document().json_size(4 * 1024 * 1024, || false).unwrap();
    assert!(canonical > 1_048_576 && canonical <= 4 * 1024 * 1024);
    // RR04 already prevents >4MiB output. This distinct baseline is below that
    // ceiling, so F47's unnecessary owned serialization/reparse still executes.
    // Keep a live owner outside observation: validate may drop its Arc clone.
    // Allocating before TRACK and dropping the sole owner inside it skews live.
    let cancel_owner = Arc::new(AtomicBool::new(false));
    let cancel = cancel_owner.clone();
    STATS.set(Allocations::default());
    TRACK.set(true);
    let result = validate("package P;", tx.document(), &[NAMESPACE], cancel);
    TRACK.set(false);
    let stats = STATS.get();
    assert!(!cancel_owner.load(Ordering::Relaxed));
    eprintln!("RR21 canonical_bytes={canonical}; allocation={stats:?}");
    assert!(matches!(result, Err(Error::UnsupportedShape(_))));
    assert!(stats.largest <= 65_536, "{stats:?}");
    assert!(stats.peak <= 131_072, "{stats:?}");
    assert_eq!(stats.live, 0, "{stats:?}");
    assert_eq!(editor.document(), &before);
    tx.cancel();
}

fn flag() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}
fn run(source: &str, document: &Document) -> Report {
    validate(source, document, &[NAMESPACE, PART], flag()).unwrap()
}
// Construct exact canonical sizes independently of the implementation's count.
// JSON escaping is absent for this ASCII pad; retain the original model IDs.
fn padded(document: &Document, bytes: usize) -> Document {
    let mut value = document.value().clone();
    value["rr21Opaque"] = Value::String(String::new());
    let overhead = serde_json::to_vec_pretty(&value).unwrap().len();
    assert!(overhead <= bytes);
    value["rr21Opaque"] = Value::String("x".repeat(bytes - overhead));
    assert_eq!(serde_json::to_vec_pretty(&value).unwrap().len(), bytes);
    Document::from_json(&value.to_string()).unwrap()
}
fn observed(source: &str, document: &Document) -> (Result<Report, Error>, Allocations) {
    let owner = flag();
    let cancel = owner.clone();
    STATS.set(Allocations::default());
    TRACK.set(true);
    let result = validate(source, document, &[NAMESPACE], cancel);
    TRACK.set(false);
    assert!(!owner.load(Ordering::Relaxed));
    (result, STATS.get())
}
fn assert_bounded_refusal(result: Result<Report, Error>, stats: Allocations) {
    assert_eq!(
        result,
        Err(Error::UnsupportedShape(
            "selected document byte/depth budget"
        ))
    );
    assert!(stats.largest <= 65_536, "{stats:?}");
    assert!(stats.peak <= 131_072, "{stats:?}");
    assert_eq!(stats.live, 0, "{stats:?}");
}
#[test]
fn ignored_envelope_payload_cannot_reach_the_full_names_snapshot_clone() {
    let source = "package P {part def T;part wheel:T;}";
    let mut editor = Editor::new(padded(&imported(source), MAX_DOCUMENT_BYTES + 1));
    let before = editor.document().clone();
    // This field is outside the finite node/data whitelist. Removing only the
    // old JSON reparse would still copy it in Snapshot::new without admission.
    assert_eq!(
        editor
            .document()
            .json_size(4 * 1024 * 1024, || false)
            .unwrap(),
        MAX_DOCUMENT_BYTES + 1
    );
    let (result, stats) = observed(source, editor.document());
    eprintln!("RR21 ignored-envelope refusal allocation={stats:?}");
    assert_bounded_refusal(result, stats);
    assert_eq!(editor.document(), &before);
    assert!(!editor.undo());
    assert!(!editor.redo());
}
#[test]
fn exact_one_mib_admission_preserves_f14_checks_and_one_more_byte_refuses() {
    let source = "package P {part def T;part wheel:T;}";
    let document = imported(source);
    let expected = run(source, &document);
    assert!(expected.checks.iter().all(|check| check.passed));
    let boundary = padded(&document, MAX_DOCUMENT_BYTES);
    let before = boundary.clone();
    assert_eq!(run(source, &boundary), expected);
    assert_eq!(boundary, before);
    let excess = padded(&document, MAX_DOCUMENT_BYTES + 1);
    let (result, stats) = observed(source, &excess);
    assert_bounded_refusal(result, stats);
}
fn named<'a>(value: &'a mut Value, name: &str) -> Option<&'a mut Value> {
    if value["data"]["declaredName"] == name {
        return Some(value);
    }
    match value {
        Value::Array(rows) => rows.iter_mut().find_map(|row| named(row, name)),
        Value::Object(rows) => rows.values_mut().find_map(|row| named(row, name)),
        _ => None,
    }
}
#[test]
fn bounded_opaque_unicode_preserves_actual_f46_warning_identity_and_crlf_utf16_spans() {
    let source = "\r\npackage '🚗' {\r\n  part def T;\r\n  part '轮😀':T;\r\n}";
    let mut value = imported(source).value().clone();
    let usage = named(&mut value, "轮😀").unwrap();
    let usage_id = usage["data"]["elementId"].as_str().unwrap().to_owned();
    usage["data"]["ownedRelationship"] = json!([]);
    value["rr21Opaque"] = json!({"label":"雪😀", "nested":[null,{},[true,42]]});
    let document = Document::from_json(&value.to_string()).unwrap();
    let before = document.clone();
    let report = run(source, &document);
    assert_eq!(report.diagnostics.len(), 1);
    let diagnostic = &report.diagnostics[0];
    assert_eq!(diagnostic.rule, PART);
    assert_eq!(diagnostic.element, usage_id);
    assert_eq!(diagnostic.qualified_name.as_deref(), Some("'🚗'::'轮😀'"));
    assert_eq!(diagnostic.severity, "WARNING");
    assert_eq!(&source[diagnostic.span.bytes.clone()], "part '轮😀':T;");
    assert_eq!((diagnostic.line, diagnostic.column_utf16), (4, 3));
    assert_eq!(
        diagnostic.span.utf16.start,
        source[..diagnostic.span.bytes.start].encode_utf16().count()
    );
    assert_eq!(
        diagnostic.span.utf16.end,
        source[..diagnostic.span.bytes.end].encode_utf16().count()
    );
    let untyped = report
        .checks
        .iter()
        .find(|check| check.rule == PART && check.element == usage_id)
        .unwrap();
    assert!(!untyped.passed);
    assert_eq!(document, before);
}
#[test]
fn late_source_mismatch_and_parse_failure_preserve_error_families_and_editor_history() {
    let source = "package P {part def T;part frontPart:T;part finalPart:T;}";
    let mut editor = Editor::new(padded(&imported(source), 32_768));
    let before = editor.document().clone();
    let mismatch = source.replace("finalPart", "changed");
    assert_eq!(
        validate(&mismatch, editor.document(), &[NAMESPACE], flag()),
        Err(Error::SourceAssociation)
    );
    let broken = format!("{source} @");
    assert!(matches!(
        validate(&broken, editor.document(), &[NAMESPACE], flag()),
        Err(Error::Parse(_))
    ));
    assert_eq!(editor.document(), &before);
    assert!(!editor.undo());
    assert!(!editor.redo());
}
#[test]
fn deep_provisional_opaque_data_refuses_before_parser_or_snapshot_without_publication() {
    let mut editor = Editor::new(imported("package P;"));
    let before = editor.document().clone();
    let parent = editor
        .document()
        .elements()
        .iter()
        .find(|row| row.name.as_deref() == Some("P"))
        .unwrap()
        .element_id
        .clone();
    let mut opaque = Value::Null;
    for _ in 0..128 {
        opaque = Value::Array(vec![opaque]);
    }
    let fragment = json!({"id":id(5,1),"eClass":"sysml:OwningMembership",
        "data":{"elementId":id(6,1),"opaque":opaque}});
    let mut tx = editor.begin();
    tx.attach_preallocated_subtree(&parent, ContainmentFeature::OwnedRelationship, fragment)
        .unwrap();
    // The provisional edit is not a published/reloadable model. RR04's depth
    // authority also covers unknown scalar payload; the parser must not run.
    assert!(tx.document().json_size(4 * 1024 * 1024, || false).is_err());
    let (result, stats) = observed("@", tx.document());
    assert_bounded_refusal(result, stats);
    tx.cancel();
    assert_eq!(editor.document(), &before);
    assert!(!editor.undo());
    assert!(!editor.redo());
}
#[test]
fn cancellation_and_explicit_rule_selection_stay_typed_before_document_admission() {
    let document = padded(&imported("package P;"), MAX_DOCUMENT_BYTES + 1);
    assert_eq!(
        validate(
            "@",
            &document,
            &[NAMESPACE],
            Arc::new(AtomicBool::new(true))
        ),
        Err(Error::Cancelled)
    );
    assert_eq!(
        validate("@", &document, &[], flag()),
        Err(Error::UnsupportedShape(
            "explicit nonempty rule selection required"
        ))
    );
    assert_eq!(
        validate("@", &document, &["unknown"], flag()),
        Err(Error::UnsupportedRule("unknown".into()))
    );
    let small = imported("package P;");
    assert_eq!(
        validate("package P;", &small, &[NAMESPACE, NAMESPACE], flag()).unwrap(),
        validate("package P;", &small, &[NAMESPACE], flag()).unwrap()
    );
}
