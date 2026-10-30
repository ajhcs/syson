// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::collections::HashMap;
use syson_migration_pipeline::{Project, Resolver, SemanticDocument};
use syson_migrations::b25b::{VERSION, prepare};
use syson_model::Document;
use syson_persistence::{REFERENCES_VERSION, ResolvedElement};

const NS: &str = "http://www.eclipse.org/syson/sysml";
const CONNECTORS: [&str; 8] = [
    "AllocationUsage",
    "BindingConnectorAsUsage",
    "ConnectionUsage",
    "ConnectorAsUsage",
    "FlowUsage",
    "InterfaceUsage",
    "SuccessionAsUsage",
    "SuccessionFlowUsage",
];
fn resource(content: Value) -> Value {
    json!({"json":{"version":"1.0","encoding":"utf-8"},
        "ns":{"sysml":NS},"content":content})
}

#[test]
fn exact_version_and_every_connector_subclass_remove_only_direct_object_children_fields() {
    assert_eq!(VERSION, "2025.4.0-202502270000");
    for class in CONNECTORS {
        let input = resource(json!([{
            "eClass":format!("sysml:{class}"), "source":"raw-parent-source", "target":73,
            "data":{"source":["old-source"],"target":null,"declaredName":"Link","other":true},
            "vendor":{"source":{"nested":"old"},"target":[1,false],"keep":"opaque"}
        }]));
        let source = input.to_string();
        let original = source.clone();
        let result = prepare(&source).unwrap();
        assert_eq!(result.participant_version, VERSION);
        assert!(result.matched_resource);
        assert_eq!(source, original);
        let expected = resource(json!([{
            "eClass":format!("sysml:{class}"), "source":"raw-parent-source", "target":73,
            "data":{"declaredName":"Link","other":true}, "vendor":{"keep":"opaque"}
        }]));
        assert_eq!(result.json, expected, "{class}");
    }
}

#[test]
fn matching_typed_parent_selects_vendor_objects_but_not_array_items_or_grandchildren() {
    let input = resource(json!({
        "eClass":"sysml:ConnectorAsUsage", "source":{"source":"remove-inner","target":2,"keep":3},
        "target":[{"source":"array-item-stays","target":4}],
        "vendor":{"source":"direct-child-removes","target":null,
            "deeper":{"source":"grandchild-stays","target":5}},
        "data":{"source":"remove","target":"remove","nested":[[{
            "eClass":"sysml:SuccessionAsUsage", "data":{"source":"nested-remove","target":"nested-remove","keep":true}
        }]]}
    }));
    let expected = resource(json!({
        "eClass":"sysml:ConnectorAsUsage", "source":{"keep":3},
        "target":[{"source":"array-item-stays","target":4}],
        "vendor":{"deeper":{"source":"grandchild-stays","target":5}},
        "data":{"nested":[[{"eClass":"sysml:SuccessionAsUsage","data":{"keep":true}}]]}
    }));
    assert_eq!(prepare(&input.to_string()).unwrap().json, expected);
}

#[test]
fn arbitrary_nested_arrays_reach_connectors_without_migrating_unrelated_classes() {
    let input = resource(json!([null, 42, [[{
        "eClass":"sysml:Package", "data":{"source":"package-source","target":"package-target"},
        "vendor":[{"eClass":"sysml:ConnectionUsage","data":{"source":[],"target":{},"keep":1}}]
    }]], {"eClass":"sysml:Connector","data":{"source":"kernel","target":"kernel"}},
        {"eClass":"vendor:ConnectorAsUsage","data":{"source":"prefix","target":"prefix"}},
        {"eClass":"sysml:UnknownConnector","data":{"source":"unknown","target":"unknown"}},
        {"eClass":false,"data":{"source":"boolean-class","target":7}},
        {"eClass":{},"data":{"source":"object-class","target":8}}
    ]));
    let mut expected = input.clone();
    expected["content"][2][0][0]["vendor"][0]["data"] = json!({"keep":1});
    assert_eq!(prepare(&input.to_string()).unwrap().json, expected);
}

#[test]
fn first_namespace_entry_controls_matching_without_reordering_or_later_fallback() {
    // Literal input order matters even when the JSON output map is sorted.
    let first_wrong = r#"{"ns":{"a":"urn:vendor","sysml":"http://www.eclipse.org/syson/sysml"},"eClass":"sysml:ConnectorAsUsage","data":{"source":1,"target":2}}"#;
    let first_right = r#"{"ns":{"sysml":"http://www.eclipse.org/syson/sysml","a":"urn:vendor"},"eClass":"sysml:ConnectorAsUsage","data":{"source":1,"target":2}}"#;
    let wrong = prepare(first_wrong).unwrap();
    assert!(!wrong.matched_resource);
    assert_eq!(
        wrong.json,
        serde_json::from_str::<Value>(first_wrong).unwrap()
    );
    let right = prepare(first_right).unwrap();
    assert!(right.matched_resource);
    assert_eq!(right.json["data"], json!({}));
    for source in [
        r#"{"ns":{},"data":{"source":1}}"#,
        r#"{"ns":null,"data":{"target":2}}"#,
        r#"{"data":{"source":3}}"#,
    ] {
        let result = prepare(source).unwrap();
        assert!(!result.matched_resource);
        assert_eq!(result.json, serde_json::from_str::<Value>(source).unwrap());
    }
}

#[test]
fn absent_fields_replay_and_unknown_json_values_are_preserved_without_null_insertion() {
    let input = resource(json!([{
        "eClass":"sysml:InterfaceUsage", "data":{"declaredName":"Link","nested":{"large":900719925474099312345u128}},
        "vendor":{"keep":[false,null,"source"]}
    }]));
    let first = prepare(&input.to_string()).unwrap();
    assert_eq!(first.json, input);
    let replay = prepare(&first.to_json().unwrap()).unwrap();
    assert_eq!(replay.json, first.json);
    assert_eq!(replay.participant_version, VERSION);
    assert!(replay.matched_resource);
}

#[test]
fn malformed_input_and_nonobject_roots_publish_no_prepared_output() {
    let published = prepare(&resource(json!([])).to_string()).unwrap().json;
    let original = published.clone();
    for invalid in ["{", "{\"ns\":", "[]", "null", "12"] {
        assert!(prepare(invalid).is_err(), "{invalid}");
        assert_eq!(published, original);
    }
}

struct NoLoad;
impl Resolver for NoLoad {
    fn resolve(&mut self, _: &str) -> Result<Option<ResolvedElement>, String> {
        Ok(None)
    }
}
#[test]
fn migrated_json_opens_as_document_then_b01_prepares_library_ids_with_separate_watermark() {
    let input = resource(json!([
        {"id":"connector-node","eClass":"sysml:ConnectionUsage","data":{
            "elementId":"connector-semantic","declaredName":"Link","source":["legacy-source"],"target":["legacy-target"]}},
        {"id":"typing-node","eClass":"sysml:FeatureTyping","data":{
            "elementId":"typing-semantic","type":"sysml:Type sysmllibrary:/Types#old"}}
    ]));
    let before = input.clone();
    let prepared = prepare(&input.to_string()).unwrap();
    let document = Document::from_json(&prepared.to_json().unwrap()).unwrap();
    assert_eq!(input, before);
    let connector = document.element("connector-semantic").unwrap();
    assert!(connector["data"].get("source").is_none());
    assert!(connector["data"].get("target").is_none());
    assert_eq!(connector["id"], "connector-node");
    assert_eq!(connector["data"]["declaredName"], "Link");
    let project = Project {
        documents: vec![SemanticDocument {
            document,
            references_version: None,
        }],
        representations: vec![],
    };
    let snapshot = project.clone();
    let ids = HashMap::from([
        ("old".into(), "new".into()),
        ("new".into(), "wrong-on-replay".into()),
    ]);
    let migrated = syson_migration_pipeline::prepare(&project, &ids, &mut NoLoad).unwrap();
    assert_eq!(project, snapshot);
    assert_eq!(
        migrated.documents[0].references_version.as_deref(),
        Some(REFERENCES_VERSION)
    );
    assert_eq!(prepared.participant_version, VERSION);
    assert_ne!(VERSION, REFERENCES_VERSION);
    assert_eq!(
        migrated.documents[0]
            .document
            .element("typing-semantic")
            .unwrap()["data"]["type"],
        "sysml:Type sysmllibrary:/Types#new"
    );
    assert_eq!(
        syson_migration_pipeline::prepare(&migrated, &ids, &mut NoLoad).unwrap(),
        migrated
    );
    assert_eq!(
        prepare(&migrated.documents[0].document.to_json().unwrap())
            .unwrap()
            .json,
        *migrated.documents[0].document.value()
    );
}

#[test]
fn exact_packet_bytes_match_separately_recorded_source_predictions() {
    let packet: Value = serde_json::from_str(include_str!(
        "../../../../porting/rust-parity-evidence/B25b/native-fixtures.json"
    ))
    .unwrap();
    let predictions: Value = serde_json::from_str(include_str!(
        "../../../../porting/rust-parity-evidence/B25b/source-predictions.json"
    ))
    .unwrap();
    let cases = packet["cases"].as_array().unwrap();
    let predictions = predictions["predictions"].as_array().unwrap();
    assert_eq!(cases.len(), 12);
    assert_eq!(cases.len(), predictions.len());
    for case in cases {
        let input = case["input"].as_str().unwrap();
        let expected = &predictions
            .iter()
            .find(|row| row["id"] == case["id"])
            .unwrap()["source_prediction"];
        let result = prepare(input);
        if expected["outcome"] == "ERROR_NO_PREPARED_OUTPUT" {
            assert!(result.is_err(), "case {}", case["id"]);
        } else {
            let result = result.unwrap();
            assert_eq!(result.json, expected["json"], "case {}", case["id"]);
            assert_eq!(result.participant_version, VERSION);
            assert_eq!(
                result.matched_resource,
                expected["matched_resource"].as_bool().unwrap()
            );
        }
    }
}
