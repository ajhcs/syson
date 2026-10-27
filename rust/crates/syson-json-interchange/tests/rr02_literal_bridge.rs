// SPDX-License-Identifier: EPL-2.0
use serde_json::{Map, Value};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};
use syson_json_interchange::{Bundle, Input, MAX_RESOURCE_BYTES, native_model, selected_model};
use syson_model::{Document, Editor, NodeIds};
use syson_references::FragmentIdentity;
use syson_text_import::Importer;

const KEY: &str = "rr02-resource";
const NUMBER: &str = "$serde_json::private::Number";
const RAW: &str = "$serde_json::private::RawValue";
static NEXT: AtomicU64 = AtomicU64::new(0);
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
            "package Vehicle {part def Car;part car:Car;}",
            None,
            (1..100).map(ids),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .commit(&mut editor)
        .unwrap();
    editor.document().clone()
}
fn object(pairs: Vec<(&str, Value)>) -> Value {
    let mut fields = Map::new();
    for (key, value) in pairs {
        fields.insert(key.into(), value);
    }
    Value::Object(fields)
}
fn payload() -> Value {
    object(vec![
        ("numberObject", object(vec![(NUMBER, Value::String("123".into()))])),
        ("rawObject", object(vec![(RAW, Value::String(r#"{"id":"not-a-physical-node","eClass":"sysml:FeatureTyping","data":{"type":"opaque","typedFeature":"opaque"}}"#.into()))])),
        ("nested", Value::Array(vec![object(vec![("before", Value::Null), (NUMBER, Value::String("-0".into())), (RAW, Value::String("[1,2]".into()))])])),
    ])
}
fn named(document: &Document, name: &str) -> String {
    document
        .elements()
        .iter()
        .find(|n| n.name.as_deref() == Some(name))
        .unwrap()
        .element_id
        .clone()
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
fn decorated() -> Document {
    let document = imported();
    let mut value = document.value().clone();
    value["vendor"] = payload();
    value["content"][0]["annotation"] = payload();
    value["content"][0]["data"]["vendor"] = payload();
    Document::from_json(&value.to_string()).unwrap()
}
fn assert_payload(document: &Document) {
    let expected = payload();
    assert_eq!(document.value()["vendor"], expected);
    assert_eq!(document.value()["content"][0]["annotation"], expected);
    assert_eq!(document.value()["content"][0]["data"]["vendor"], expected);
}
fn fixture_file() -> PathBuf {
    let root = PathBuf::from(std::env::var_os("TMPDIR").expect("owned workspace TMPDIR required"));
    assert!(root.is_absolute() && root.starts_with("/workspace"));
    assert!(
        !root
            .components()
            .any(|part| part == std::path::Component::ParentDir)
    );
    let dir = root.join(format!(
        "rr02-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).unwrap();
    dir.join("native.emf.json")
}
#[test]
fn real_editor_forward_save_file_reload_reverse_preserves_literal_data_and_identity() {
    let selected = decorated();
    let original = selected.clone();
    let original_nodes = selected.elements();
    assert_payload(&selected);
    let native = forward(&selected).document(KEY).unwrap().clone();
    assert_payload(&native);
    assert_eq!(native.elements(), original_nodes);
    let mut editor = Editor::new(reverse(&native));
    let car = named(editor.document(), "car");
    let mut transaction = editor.begin();
    transaction.rename(&car, "frontCar").unwrap();
    assert!(editor.commit(transaction).unwrap());
    let expected = editor.document().clone();
    let saved = forward(&expected).save().unwrap();
    assert_eq!(saved.len(), 1);
    let file = fixture_file();
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&file)
        .unwrap();
    output.write_all(saved[0].json.as_bytes()).unwrap();
    output.sync_all().unwrap();
    drop(output);
    let reloaded = Bundle::load(vec![Input {
        key: KEY.into(),
        json: fs::read_to_string(&file).unwrap(),
        fragments: FragmentIdentity::JsonNodeId,
    }])
    .unwrap();
    let reopened = reverse(reloaded.document(KEY).unwrap());
    assert_payload(&reopened);
    assert_eq!(reopened, expected);
    assert_eq!(selected, original);
    assert_eq!(named(&reopened, "frontCar"), car);
    assert_eq!(reopened.elements().len(), original_nodes.len());
    for row in original_nodes {
        let after = reopened.element(&row.element_id).unwrap();
        assert_eq!(after["id"], row.resource_id);
        assert_eq!(after["data"]["elementId"], row.element_id);
    }
    let typing = &reopened.element(&car).unwrap()["data"]["ownedRelationship"][0]["data"];
    assert_eq!(typing["typedFeature"], car);
    assert_eq!(typing["type"], named(&reopened, "Car"));
}
#[test]
fn raw_numeric_tokens_survive_both_bridges_without_generic_value_reparse() {
    for token in ["-0", "0.00", "1E+000", "1e-99999", "900719925474099312345"] {
        let mut value = imported().value().clone();
        value["vendor"] = Value::String("RR02_NUMBER_TOKEN".into());
        let raw = value.to_string().replace("\"RR02_NUMBER_TOKEN\"", token);
        let selected = Document::from_json(&raw).unwrap();
        let converted = forward(&selected);
        let saved = Bundle::load(converted.save().unwrap()).unwrap();
        let reopened = reverse(saved.document(KEY).unwrap());
        let Value::Number(number) = &reopened.value()["vendor"] else {
            panic!("number expected")
        };
        assert_eq!(number.to_string(), token);
    }
}
#[test]
fn literal_markers_do_not_expand_bridge_budgets_or_mutate_published_documents() {
    let selected = decorated();
    let native = forward(&selected).document(KEY).unwrap().clone();
    let selected_before = selected.clone();
    let native_before = native.clone();
    for (source, local) in [(&selected, true), (&native, false)] {
        let mut value = source.value().clone();
        value["large"] = Value::String("x".repeat(MAX_RESOURCE_BYTES));
        let oversized = Document::from_json(&value.to_string()).unwrap();
        let before = oversized.clone();
        if local {
            assert!(matches!(
                selected_model::load(
                    KEY,
                    &oversized,
                    selected_model::SourceEncoding::LocalSemanticElementIds
                ),
                Err(selected_model::Error::Limit)
            ));
        } else {
            assert!(matches!(
                native_model::load(
                    KEY,
                    &oversized,
                    native_model::SourceEncoding::LocalJsonNodeIds
                ),
                Err(native_model::Error::Limit)
            ));
        }
        assert_eq!(oversized, before);
    }
    assert_eq!(selected, selected_before);
    assert_eq!(native, native_before);
    assert_payload(&reverse(&native));
}
#[test]
fn duplicate_decoded_keys_refuse_at_bundle_document_boundary() {
    let document = imported();
    let raw = document.to_json().unwrap();
    // Add a literal duplicate at the root without parsing the expected object.
    let trimmed = raw.trim_end();
    let duplicate = format!(
        "{},\"rr02\":0,\"\\u0072r02\":1}}",
        &trimmed[..trimmed.len() - 1]
    );
    assert!(
        matches!(Bundle::load(vec![Input { key: KEY.into(), json: duplicate, fragments: FragmentIdentity::JsonNodeId }]), Err(syson_json_interchange::Error::Model(message)) if message.contains("DuplicateKey"))
    );
}

#[test]
fn dense_pretty_expansion_keeps_bundle_limit_error_without_materialized_output() {
    let mut value = imported().value().clone();
    let mut opaque = Value::Array(vec![Value::Null; 100_000]);
    for _ in 0..60 {
        opaque = Value::Array(vec![opaque]);
    }
    value["opaque"] = opaque;
    let raw = value.to_string();
    assert!(raw.len() < MAX_RESOURCE_BYTES);
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
        Err(syson_json_interchange::Error::Limit)
    ));
}
