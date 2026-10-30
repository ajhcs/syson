// SPDX-License-Identifier: EPL-2.0
use serde_json::{Map, Value};
use std::collections::HashSet;
use syson_migrations::{Error, MAX_BYTES, SYSML_URI, b25c};
const ENCODED: &str = r#"{"eClass":"sysml:ViewUsage","data":{"exposedElement":"phantom"}}"#;
const MARKERS: &str = r#""number":{"$serde_json::private::Number":"not numeric"},"raw":{"$serde_json::private::RawValue":"{\"eClass\":\"sysml:ViewUsage\",\"data\":{\"exposedElement\":\"phantom\"}}"}"#;
const CONTENT: &str = r#"{"eClass":"sysml:ViewUsage","data":{"exposedElement":"old","exposeElement":"neighbor","keep":7}}"#;
fn marker(key: &str, text: &str) -> Value {
    let mut object = Map::new();
    object.insert(key.into(), Value::String(text.into()));
    Value::Object(object)
}
fn opaque(value: &Value) {
    assert_eq!(
        value["number"],
        marker("$serde_json::private::Number", "not numeric")
    );
    assert_eq!(
        value["raw"],
        marker("$serde_json::private::RawValue", ENCODED)
    );
}
#[test]
fn literal_order_and_private_failure_survive_the_actual_view_participant() {
    for token in ["-0", "1.2300e+004", "1E+400"] {
        for (first, last, matched) in [("foreign", SYSML_URI, true), (SYSML_URI, "foreign", false)]
        {
            let source = format!(
                r#"{{{MARKERS},"token":{token},"ns":{{"z":"{first}","a":"foreign","\u007a":"{last}"}},"content":[{CONTENT}]}}"#
            );
            let original = source.clone();
            let prepared = b25c::prepare(&source).unwrap();
            assert_eq!(prepared.matched_resource, matched);
            assert_eq!(prepared.participant_version, "2025.6.0-202506110000");
            opaque(&prepared.json);
            assert_eq!(prepared.json["token"].to_string(), token);
            let data = &prepared.json["content"][0]["data"];
            assert_eq!(data.get("exposedElement").is_none(), matched);
            assert_eq!(data["exposeElement"], "neighbor");
            assert_eq!(data["keep"], 7);
            let wire = prepared.to_json().unwrap();
            assert!(wire.contains(&format!(r#""ns":{{"z":"{last}","a":"foreign"}}"#)));
            let replay = b25c::prepare(&wire).unwrap();
            opaque(&replay.json);
            assert_eq!(replay.json["token"].to_string(), token);
            assert_eq!(replay.matched_resource, matched);
            assert_eq!(replay.to_json().unwrap(), wire);
            assert!(matches!(
                b25c::prepare(&(source.clone() + " trailing")),
                Err(Error::Input(_))
            ));
            let mut private = b25c::prepare(&source).unwrap();
            private.json["oversized"] = Value::String("x".repeat(MAX_BYTES));
            assert_eq!(private.to_json(), Err(Error::Limit));
            assert_eq!(prepared.to_json().unwrap(), wire);
            assert_eq!(source, original);
        }
    }
}
#[test]
fn complete_thirteen_row_packet_matches_its_labeled_source_expectations() {
    let packet: Value = serde_json::from_str(include_str!(
        "../../../../porting/rust-parity-evidence/B25c/native-fixtures.json"
    ))
    .unwrap();
    let predictions: Value = serde_json::from_str(include_str!(
        "../../../../porting/rust-parity-evidence/B25c/source-predictions.json"
    ))
    .unwrap();
    assert_eq!(
        predictions["status"],
        "SOURCE_DERIVED_NOT_NATIVE_OBSERVATIONS"
    );
    let cases = packet["cases"].as_array().unwrap();
    let expected = predictions["cases"].as_array().unwrap();
    assert_eq!((cases.len(), expected.len()), (13, 13));
    let mut seen = HashSet::new();
    for case in cases {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id), "duplicate packet ID {id}");
        let expected = expected.iter().find(|row| row["id"] == id).unwrap();
        let source = case["input"].as_str().unwrap();
        let original = source.to_owned();
        let result = b25c::prepare(source);
        if id == "11-null-first-namespace" {
            assert_eq!(expected["native_expectation"], "THROWS_NAMESPACE_ACCESS");
            assert_eq!(result, Err(Error::NamespaceShape));
        } else {
            assert_eq!(expected["native_expectation"], "SUCCESS");
            let prepared = result.unwrap();
            let matched = !matches!(id, "09-first-namespace-foreign" | "10-empty-namespace");
            assert_eq!(prepared.json, expected["expected_json"], "{id}");
            assert_eq!(prepared.matched_resource, matched, "{id}");
            assert_eq!(prepared.participant_version, "2025.6.0-202506110000");
            let wire = prepared.to_json().unwrap();
            let replay = b25c::prepare(&wire).unwrap();
            assert_eq!(replay.json, prepared.json, "{id}");
            assert_eq!(replay.matched_resource, matched, "{id}");
            assert_eq!(replay.to_json().unwrap(), wire, "{id}");
        }
        assert_eq!(source, original);
    }
}
