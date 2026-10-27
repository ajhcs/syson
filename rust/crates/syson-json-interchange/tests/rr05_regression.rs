// SPDX-License-Identifier: EPL-2.0
//! Development-only observer: largest requested allocation, not RSS/live memory.
use serde_json::{Value, json};
use std::sync::{Arc, OnceLock, atomic::AtomicBool};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};
use syson_json_interchange::{
    Bundle, Error, Input, MAX_RESOURCE_BYTES, native_model, selected_model,
};
use syson_model::{ContainmentFeature, Document, Editor, NodeIds};
use syson_references::FragmentIdentity;
use syson_text_import::Importer;

thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static LARGEST: Cell<usize> = const { Cell::new(0) };
}
fn requested(size: usize) {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = LARGEST.try_with(|cell| cell.set(cell.get().max(size)));
    }
}
struct Observer;
// SAFETY: Original pointer/layout arguments always reach System unchanged;
// constant TLS bookkeeping does not allocate or own allocation metadata.
unsafe impl GlobalAlloc for Observer {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            requested(layout.size());
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            requested(layout.size());
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let next = unsafe { System.realloc(pointer, layout, size) };
        if !next.is_null() {
            requested(size);
        }
        next
    }
}
#[global_allocator]
static ALLOCATOR: Observer = Observer;
fn observed<T>(action: impl FnOnce() -> T) -> (T, usize) {
    LARGEST.set(0);
    TRACK.set(true);
    let result = action();
    TRACK.set(false);
    (result, LARGEST.get())
}
const KEY: &str = "rr05-resource";
fn envelope() -> Value {
    json!({"json":{"version":"1.0","encoding":"utf-8"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{
        "id":"10000000-0000-4000-8000-000000000001","eClass":"sysml:Package",
        "data":{"elementId":"20000000-0000-4000-8000-000000000001","declaredName":"Root","opaque":""}
    }]})
}
fn over_canonical_input() -> Input {
    let mut value = envelope();
    let overhead = value.to_string().len();
    value["content"][0]["data"]["opaque"] =
        Value::String("x".repeat(MAX_RESOURCE_BYTES - overhead));
    let json = value.to_string();
    assert_eq!(json.len(), MAX_RESOURCE_BYTES);
    Input {
        key: KEY.into(),
        json,
        fragments: FragmentIdentity::JsonNodeId,
    }
}
#[test]
fn bundle_one_mib_admission_refuses_without_allocating_over_budget_pretty_output() {
    let input = over_canonical_input();
    let document = Document::from_json(&input.json).unwrap();
    assert!(document.json_size(4 * 1024 * 1024, || false).unwrap() > MAX_RESOURCE_BYTES);
    let inputs = vec![input]; // All caller input construction precedes tracking.
    let (result, largest) = observed(|| Bundle::load(inputs));
    eprintln!("bundle canonical-limit refusal: largest_requested={largest}");
    assert!(matches!(result, Err(Error::Limit)));
    // Parsing may own one admitted scalar. It must not own a >1MiB pretty output.
    assert!(largest <= MAX_RESOURCE_BYTES, "largest_requested={largest}");
}
#[test]
fn both_borrowed_bridge_entries_refuse_before_any_large_serialized_copy() {
    let input = over_canonical_input();
    let document = Document::from_json(&input.json).unwrap();
    let before = document.clone();
    let (forward, largest) = observed(|| {
        selected_model::load(
            KEY,
            &document,
            selected_model::SourceEncoding::LocalSemanticElementIds,
        )
    });
    eprintln!("forward canonical-limit refusal: largest_requested={largest}");
    assert!(matches!(forward, Err(selected_model::Error::Limit)));
    assert!(largest <= 65_536, "largest_requested={largest}");
    let (reverse, largest) = observed(|| {
        native_model::load(
            KEY,
            &document,
            native_model::SourceEncoding::LocalJsonNodeIds,
        )
    });
    eprintln!("reverse canonical-limit refusal: largest_requested={largest}");
    assert!(matches!(reverse, Err(native_model::Error::Limit)));
    assert!(largest <= 65_536, "largest_requested={largest}");
    assert_eq!(document, before);
}

fn ids(n: u32) -> NodeIds {
    NodeIds::new(
        &format!("10000000-0000-4000-8000-{n:012x}"),
        &format!("20000000-0000-4000-8000-{n:012x}"),
    )
    .unwrap()
}
fn imported() -> Document {
    static IMPORTER: OnceLock<Importer> = OnceLock::new();
    let mut editor = Editor::new(Document::empty());
    IMPORTER
        .get_or_init(|| Importer::pinned().unwrap())
        .stage(
            editor.begin(),
            "package Vehicle {part def Wheel;part wheel:Wheel;}",
            None,
            (1..100).map(ids),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .commit(&mut editor)
        .unwrap();
    editor.document().clone()
}
fn marker_object() -> Value {
    let mut fields = serde_json::Map::new();
    fields.insert(
        "$serde_json::private::Number".into(),
        Value::String("123".into()),
    );
    fields.insert(
        "$serde_json::private::RawValue".into(),
        Value::String(r#"{"id":"not-a-node","eClass":"sysml:Namespace"}"#.into()),
    );
    fields.insert("unicode".into(), Value::String("é🚗\\\"\n".into()));
    Value::Object(fields)
}
fn at_canonical_size(size: usize) -> Document {
    let mut value = imported().value().clone();
    value["vendor"] = marker_object();
    value["content"][0]["data"]["padding"] = Value::String(String::new());
    let empty = Document::from_json(&value.to_string()).unwrap();
    let overhead = empty.json_size(4 * 1024 * 1024, || false).unwrap();
    value["content"][0]["data"]["padding"] = Value::String("x".repeat(size - overhead));
    let document = Document::from_json(&value.to_string()).unwrap();
    assert_eq!(document.json_size(4 * 1024 * 1024, || false).unwrap(), size);
    document
}
fn forward(document: &Document) -> Bundle {
    selected_model::load(
        KEY,
        document,
        selected_model::SourceEncoding::LocalSemanticElementIds,
    )
    .unwrap()
}
fn reverse(document: &Document) -> Document {
    native_model::load(
        KEY,
        document,
        native_model::SourceEncoding::LocalJsonNodeIds,
    )
    .unwrap()
}
#[test]
fn exact_and_near_one_mib_actual_bridges_save_reload_without_large_fragment_resizing() {
    for bytes in [MAX_RESOURCE_BYTES - 1, MAX_RESOURCE_BYTES] {
        let selected = at_canonical_size(bytes);
        let before = selected.clone();
        let (native, largest) = observed(|| forward(&selected));
        assert!(
            largest <= MAX_RESOURCE_BYTES,
            "forward largest_requested={largest}"
        );
        let native = native.document(KEY).unwrap();
        assert_eq!(
            native.json_size(MAX_RESOURCE_BYTES, || false).unwrap(),
            bytes
        );
        assert_eq!(native.value()["vendor"], marker_object());
        assert_eq!(native.elements(), selected.elements());
        let (restored, largest) = observed(|| reverse(native));
        assert!(
            largest <= MAX_RESOURCE_BYTES,
            "reverse largest_requested={largest}"
        );
        assert_eq!(restored, selected);
        assert_eq!(
            restored.json_size(MAX_RESOURCE_BYTES, || false).unwrap(),
            bytes
        );
        let bundle = forward(&restored);
        let (saved, largest) = observed(|| bundle.save().unwrap());
        assert!(
            largest <= MAX_RESOURCE_BYTES,
            "save largest_requested={largest}"
        );
        assert_eq!(saved[0].json.len(), bytes);
        assert_eq!(
            reverse(Bundle::load(saved).unwrap().document(KEY).unwrap()),
            selected
        );
        assert_eq!(selected, before);
    }
}
#[test]
fn one_byte_over_canonical_budget_refuses_each_admission_with_no_borrowed_copy() {
    let selected = at_canonical_size(MAX_RESOURCE_BYTES + 1);
    let before = selected.clone();
    let (result, largest) = observed(|| {
        selected_model::load(
            KEY,
            &selected,
            selected_model::SourceEncoding::LocalSemanticElementIds,
        )
    });
    assert!(matches!(result, Err(selected_model::Error::Limit)));
    assert!(largest <= 65_536, "largest_requested={largest}");
    let (result, largest) = observed(|| {
        native_model::load(
            KEY,
            &selected,
            native_model::SourceEncoding::LocalJsonNodeIds,
        )
    });
    assert!(matches!(result, Err(native_model::Error::Limit)));
    assert!(largest <= 65_536, "largest_requested={largest}");
    let input = Input {
        key: KEY.into(),
        json: serde_json::to_string(selected.value()).unwrap(),
        fragments: FragmentIdentity::JsonNodeId,
    };
    assert!(input.json.len() < MAX_RESOURCE_BYTES);
    assert!(matches!(Bundle::load(vec![input]), Err(Error::Limit)));
    assert_eq!(selected, before);
}
#[test]
fn numeric_tokens_and_literal_private_objects_survive_both_bridges() {
    for token in ["-0", "0.00", "1E+000", "1e-99999", "900719925474099312345"] {
        let mut value = imported().value().clone();
        value["vendor"] = marker_object();
        value["number"] = Value::String("RR05_NUMBER_TOKEN".into());
        let raw = value.to_string().replace("\"RR05_NUMBER_TOKEN\"", token);
        let selected = Document::from_json(&raw).unwrap();
        let native = forward(&selected);
        let reloaded = Bundle::load(native.save().unwrap()).unwrap();
        let restored = reverse(reloaded.document(KEY).unwrap());
        assert_eq!(restored.value()["vendor"], marker_object());
        let Value::Number(number) = &restored.value()["number"] else {
            panic!("number expected")
        };
        assert_eq!(number.to_string(), token);
        assert_eq!(restored, selected);
    }
}
#[test]
fn wrong_identity_domains_still_refuse_before_conversion_and_preserve_inputs() {
    let selected = at_canonical_size(MAX_RESOURCE_BYTES);
    let native = forward(&selected).document(KEY).unwrap().clone();
    let selected_before = selected.clone();
    let native_before = native.clone();
    assert!(
        native_model::load(
            KEY,
            &selected,
            native_model::SourceEncoding::LocalJsonNodeIds
        )
        .is_err()
    );
    assert!(
        selected_model::load(
            KEY,
            &native,
            selected_model::SourceEncoding::LocalSemanticElementIds
        )
        .is_err()
    );
    assert_eq!(selected, selected_before);
    assert_eq!(native, native_before);
}
fn original_dense_json() -> String {
    // Exact original audit fixture. The RR02/04 model4MiB guard now refuses it
    // already; it is not the remaining #94 one-MiB allocation counterexample.
    let mut opaque = String::new();
    for _ in 0..60 {
        opaque.push_str("{\"x\":");
    }
    opaque.push('[');
    for i in 0..100_000 {
        if i != 0 {
            opaque.push(',');
        }
        opaque.push_str("null");
    }
    opaque.push(']');
    for _ in 0..60 {
        opaque.push('}');
    }
    let raw = format!(
        r#"{{"json":{{"version":"1.0","encoding":"utf-8"}},"ns":{{"sysml":"http://www.eclipse.org/syson/sysml"}},"content":[],"vendor":{opaque}}}"#
    );
    assert_eq!(raw.len(), 500_481);
    raw
}
#[test]
fn original_sixteen_mib_audit_case_is_an_already_inherited_early_refusal() {
    let raw = original_dense_json();
    assert!(matches!(
        Document::from_json(&raw),
        Err(syson_model::ModelError::JsonAdmission(
            syson_json_value::Error::Limit(syson_json_value::Limit::Bytes)
        ))
    ));
    assert!(matches!(
        Bundle::load(vec![Input {
            key: KEY.into(),
            json: raw,
            fragments: FragmentIdentity::JsonNodeId
        }]),
        Err(Error::Limit)
    ));
}
#[test]
fn provisional_model_ceiling_failure_remains_a_bridge_limit_without_serialized_copy() {
    let editor = Editor::new(Document::from_json(&envelope().to_string()).unwrap());
    let original = editor.document().clone();
    let mut opaque = Value::Array(vec![Value::Null; 100_000]);
    for _ in 0..60 {
        opaque = Value::Array(vec![opaque]);
    }
    let relation = json!({"id":ids(2).resource,"eClass":"sysml:OwningMembership","data":{"elementId":ids(2).element,"opaque":opaque}});
    let mut tx = editor.begin();
    tx.attach_preallocated_subtree(
        &ids(1).element,
        ContainmentFeature::OwnedRelationship,
        relation,
    )
    .unwrap();
    let (result, largest) = observed(|| {
        selected_model::load(
            KEY,
            tx.document(),
            selected_model::SourceEncoding::LocalSemanticElementIds,
        )
    });
    assert!(matches!(result, Err(selected_model::Error::Limit)));
    assert!(largest <= 65_536, "largest_requested={largest}");
    let (result, largest) = observed(|| {
        native_model::load(
            KEY,
            tx.document(),
            native_model::SourceEncoding::LocalJsonNodeIds,
        )
    });
    assert!(matches!(result, Err(native_model::Error::Limit)));
    assert!(largest <= 65_536, "largest_requested={largest}");
    assert_eq!(editor.document(), &original);
    tx.cancel();
}
