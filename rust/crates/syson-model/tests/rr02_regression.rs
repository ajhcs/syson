// SPDX-License-Identifier: EPL-2.0
use serde_json::{Map, Value};
use syson_json_value::{Error as Admission, Limit};
use syson_model::{Document, Editor, ModelError};

const NODE: &str = "10000000-0000-4000-8000-000000000001";
const ELEMENT: &str = "20000000-0000-4000-8000-000000000001";
const NUMBER: &str = "$serde_json::private::Number";
const RAW: &str = "$serde_json::private::RawValue";
fn envelope(vendor: &str) -> String {
    format!(
        r#"{{"json":{{"version":"1.0","encoding":"utf-8"}},"ns":{{"sysml":"http://www.eclipse.org/syson/sysml"}},"content":[{{"id":"{NODE}","eClass":"sysml:Package","data":{{"elementId":"{ELEMENT}","declaredName":"Original","vendor":{vendor}}}}}]}}"#
    )
}
fn object(pairs: Vec<(&str, Value)>) -> Value {
    let mut fields = Map::new();
    for (key, value) in pairs {
        fields.insert(key.into(), value);
    }
    Value::Object(fields)
}
fn vendor(document: &Document) -> &Value {
    &document.element(ELEMENT).unwrap()["data"]["vendor"]
}
#[test]
fn literal_marker_objects_survive_every_member_position_with_manual_oracles() {
    for marker in [NUMBER, RAW] {
        for raw in [
            format!(r#"{{"{marker}":"123","a":false,"z":null}}"#),
            format!(r#"{{"a":false,"{marker}":"123","z":null}}"#),
            format!(r#"{{"a":false,"z":null,"{marker}":"123"}}"#),
        ] {
            let expected = object(vec![
                (marker, Value::String("123".into())),
                ("a", Value::Bool(false)),
                ("z", Value::Null),
            ]);
            let document = Document::from_json(&envelope(&raw)).unwrap();
            assert_eq!(vendor(&document), &expected);
            let reopened = Document::from_json(&document.to_json().unwrap()).unwrap();
            assert_eq!(vendor(&reopened), &expected);
        }
    }
}
#[test]
fn unrelated_editor_rename_and_save_reopen_preserve_nested_literal_objects() {
    let raw = r#"{"nested":[{"$serde_json::private::Number":"123"},{"$serde_json::private::RawValue":"{\"id\":\"not-a-physical-node\",\"eClass\":\"sysml:Namespace\"}"}]}"#;
    let expected = object(vec![(
        "nested",
        Value::Array(vec![
            object(vec![(NUMBER, Value::String("123".into()))]),
            object(vec![(
                RAW,
                Value::String(r#"{"id":"not-a-physical-node","eClass":"sysml:Namespace"}"#.into()),
            )]),
        ]),
    )]);
    let original = Document::from_json(&envelope(raw)).unwrap();
    let before = original.clone();
    let mut editor = Editor::new(original.clone());
    let mut transaction = editor.begin();
    transaction.rename(ELEMENT, "Renamed").unwrap();
    assert!(editor.commit(transaction).unwrap());
    let reopened = Document::from_json(&editor.document().to_json().unwrap()).unwrap();
    assert_eq!(vendor(&reopened), &expected);
    assert_eq!(reopened.element(ELEMENT).unwrap()["id"], NODE);
    assert_eq!(
        reopened.element(ELEMENT).unwrap()["data"]["elementId"],
        ELEMENT
    );
    assert_eq!(
        reopened.element(ELEMENT).unwrap()["data"]["declaredName"],
        "Renamed"
    );
    assert_eq!(original, before);
}

#[test]
fn original_numeric_lexemes_survive_model_save_and_unrelated_edits() {
    for token in ["-0", "0.00", "1E+000", "1e-99999", "900719925474099312345"] {
        let document = Document::from_json(&envelope(token)).unwrap();
        let Value::Number(number) = vendor(&document) else {
            panic!("number expected")
        };
        assert_eq!(number.to_string(), token);
        let mut editor = Editor::new(document);
        let mut tx = editor.begin();
        tx.rename(ELEMENT, "Changed").unwrap();
        assert!(editor.commit(tx).unwrap());
        let reopened = Document::from_json(&editor.document().to_json().unwrap()).unwrap();
        let Value::Number(number) = vendor(&reopened) else {
            panic!("number expected")
        };
        assert_eq!(number.to_string(), token);
    }
}
#[test]
fn decoded_duplicate_keys_have_typed_refusal_while_syntax_remains_json_error() {
    for raw in [
        r#"{"a":0,"\u0061":1}"#,
        r#"{"$serde_json::private::Number":"1","$serde_json::private::Number":"2"}"#,
    ] {
        assert!(matches!(
            Document::from_json(&envelope(raw)),
            Err(ModelError::JsonAdmission(Admission::DuplicateKey))
        ));
    }
    for raw in ["{", "01", "[1,]", r#""\uD800""#] {
        assert!(matches!(
            Document::from_json(&envelope(raw)),
            Err(ModelError::Json(_))
        ));
    }
    assert!(matches!(
        Document::from_json("[]"),
        Err(ModelError::Shape(_))
    ));
}
#[test]
fn model_value_depth_128_is_explicit_and_one_level_more_refuses() {
    for (arrays, admitted) in [(123, true), (124, false)] {
        let payload = format!("{}null{}", "[".repeat(arrays), "]".repeat(arrays));
        let result = Document::from_json(&envelope(&payload));
        if admitted {
            let document = result.unwrap();
            assert_eq!(
                Document::from_json(&document.to_json().unwrap()).unwrap(),
                document
            );
        } else {
            assert!(matches!(
                result,
                Err(ModelError::JsonAdmission(Admission::Limit(Limit::Depth)))
            ));
        }
    }
}

fn near_byte_limit() -> Document {
    let empty = Document::from_json(&envelope("\"\"")).unwrap();
    let overhead = empty.to_json().unwrap().len();
    let payload = format!("\"{}\"", "x".repeat(4 * 1024 * 1024 - overhead));
    let raw = envelope(&payload);
    assert!(raw.len() < 4 * 1024 * 1024);
    let document = Document::from_json(&raw).unwrap();
    assert_eq!(document.to_json().unwrap().len(), 4 * 1024 * 1024);
    document
}
#[test]
fn canonical_byte_boundary_is_reloadable_and_raw_or_pretty_overflow_refuses() {
    let exact = near_byte_limit();
    assert_eq!(
        Document::from_json(&exact.to_json().unwrap()).unwrap(),
        exact
    );
    let payload = vendor(&exact).as_str().unwrap();
    let over_pretty = envelope(&format!("\"{payload}x\""));
    assert!(over_pretty.len() < 4 * 1024 * 1024);
    assert!(matches!(
        Document::from_json(&over_pretty),
        Err(ModelError::JsonAdmission(Admission::Limit(Limit::Bytes)))
    ));
    let over_raw = " ".repeat(4 * 1024 * 1024 + 1);
    assert!(matches!(
        Document::from_json(&over_raw),
        Err(ModelError::JsonAdmission(Admission::Limit(Limit::Bytes)))
    ));
}
#[test]
fn oversized_rename_refusal_preserves_document_history_revision_and_ids() {
    let exact = near_byte_limit();
    let mut editor = Editor::new(exact.clone());
    let mut tx = editor.begin();
    tx.rename(ELEMENT, "Before").unwrap();
    assert!(editor.commit(tx).unwrap());
    assert!(editor.undo());
    assert_eq!(editor.document(), &exact);
    let mut bad = editor.begin();
    bad.rename(ELEMENT, "OriginalX").unwrap();
    assert!(matches!(
        editor.commit(bad),
        Err(ModelError::JsonAdmission(Admission::Limit(Limit::Bytes)))
    ));
    assert_eq!(editor.document(), &exact);
    assert!(editor.redo());
    assert_eq!(
        editor.document().element(ELEMENT).unwrap()["data"]["declaredName"],
        "Before"
    );
    assert!(editor.undo());
    assert_eq!(editor.document(), &exact);
    assert!(!editor.undo());
    // A failed publication must not advance revision and stale another valid tx.
    let mut editor = Editor::new(exact.clone());
    let mut valid = editor.begin();
    valid.rename(ELEMENT, "After").unwrap();
    let mut bad = editor.begin();
    bad.rename(ELEMENT, "OriginalX").unwrap();
    assert!(editor.commit(bad).is_err());
    assert!(editor.commit(valid).unwrap());
    let reopened = Document::from_json(&editor.document().to_json().unwrap()).unwrap();
    assert_eq!(reopened.element(ELEMENT).unwrap()["id"], NODE);
    assert_eq!(
        reopened.element(ELEMENT).unwrap()["data"]["elementId"],
        ELEMENT
    );
    assert_eq!(vendor(&reopened), vendor(&exact));
}

#[test]
fn created_nested_packages_obey_the_same_reload_depth_before_publication() {
    for (count, admitted) in [(21, true), (22, false)] {
        let mut editor = Editor::new(Document::empty());
        let mut tx = editor.begin();
        let mut parent = None;
        let ids = |n: u32| {
            syson_model::NodeIds::new(
                &format!("10000000-0000-4000-8000-{n:012x}"),
                &format!("20000000-0000-4000-8000-{n:012x}"),
            )
            .unwrap()
        };
        for n in 1..=count {
            let node = ids(2 * n);
            let membership = parent.as_ref().map(|_| ids(2 * n + 1));
            tx.create_package(
                parent.as_deref(),
                node.clone(),
                membership,
                &format!("P{n}"),
            )
            .unwrap();
            parent = Some(node.element);
        }
        if admitted {
            assert!(editor.commit(tx).unwrap());
            assert_eq!(
                Document::from_json(&editor.document().to_json().unwrap()).unwrap(),
                *editor.document()
            );
        } else {
            assert!(matches!(
                editor.commit(tx),
                Err(ModelError::JsonAdmission(Admission::Limit(Limit::Depth)))
            ));
            assert_eq!(editor.document(), &Document::empty());
            assert!(!editor.undo());
        }
    }
}

#[test]
fn typed_scalar_edit_and_owned_file_reopen_preserve_literal_payloads_and_identities() {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::{
        fs::{self, OpenOptions},
        io::Write,
        path::{Component, PathBuf},
    };
    static NEXT_FILE: AtomicU64 = AtomicU64::new(0);
    let raw = r#"{"marker":{"$serde_json::private::Number":"123"},"raw":{"$serde_json::private::RawValue":"{\"id\":\"opaque\",\"eClass\":\"sysml:Package\"}"},"zero":-0,"exp":1E+000,"big":900719925474099312345}"#;
    let original = Document::from_json(&envelope(raw)).unwrap();
    let mut editor = Editor::new(original.clone());
    let mut tx = editor.begin();
    tx.set_attribute(
        ELEMENT,
        "declaredShortName",
        Value::String("Typed alias\r\n雪".into()),
    )
    .unwrap();
    assert!(editor.commit(tx).unwrap());
    let canonical = editor.document().to_json().unwrap();
    let root = PathBuf::from(std::env::var_os("TMPDIR").expect("owned workspace TMPDIR required"));
    assert!(root.is_absolute() && root.starts_with("/workspace"));
    assert!(!root.components().any(|part| part == Component::ParentDir));
    let directory = root.join(format!(
        "rr02-model-{}-{}",
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    let file = directory.join("typed-model.emf.json");
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&file)
        .unwrap();
    output.write_all(canonical.as_bytes()).unwrap();
    output.sync_all().unwrap();
    drop(output);
    let saved = fs::read_to_string(&file).unwrap();
    assert_eq!(saved, canonical);
    let reopened = Document::from_json(&saved).unwrap();
    assert_eq!(&reopened, editor.document());
    let values = vendor(&reopened);
    assert_eq!(
        values["marker"],
        object(vec![(NUMBER, Value::String("123".into()))])
    );
    assert_eq!(
        values["raw"],
        object(vec![(
            RAW,
            Value::String(r#"{"id":"opaque","eClass":"sysml:Package"}"#.into())
        )])
    );
    for (key, token) in [
        ("zero", "-0"),
        ("exp", "1E+000"),
        ("big", "900719925474099312345"),
    ] {
        let Value::Number(number) = &values[key] else {
            panic!("literal number expected")
        };
        assert_eq!(number.to_string(), token);
    }
    assert_eq!(reopened.element(ELEMENT).unwrap()["id"], NODE);
    assert_eq!(
        reopened.element(ELEMENT).unwrap()["data"]["elementId"],
        ELEMENT
    );
    assert_eq!(
        reopened.element(ELEMENT).unwrap()["data"]["declaredShortName"],
        "Typed alias\r\n雪"
    );
    assert_eq!(reopened.elements().len(), 1); // RawValue-shaped text is opaque.
    assert!(editor.undo());
    assert_eq!(editor.document(), &original);
    assert!(editor.redo());
    assert_eq!(editor.document(), &reopened);
    // A successful owned-file roundtrip is not a crash-safe publication claim.
}
