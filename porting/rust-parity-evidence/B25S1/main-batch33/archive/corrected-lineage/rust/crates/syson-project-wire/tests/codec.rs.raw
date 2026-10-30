// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::collections::HashMap;
use syson_migration_pipeline::Resolver;
use syson_persistence::ResolvedElement;
use syson_project_preparation::{
    ProjectInput, RepresentationInput, SemanticInput, SemanticVersions, prepare,
};
use syson_project_wire::{
    Error, FORMAT, MAX_ENVELOPE_BYTES, MAX_METADATA_BYTES, MAX_PAYLOAD_BYTES, decode, encode,
};

const PROJECTS: &str =
    include_str!("../../../../porting/rust-parity-evidence/C03b/project-fixtures.json");
fn optional(v: &Value) -> Option<String> {
    v.as_str().map(str::to_owned)
}
fn project(id: &str) -> ProjectInput {
    let all: Value = serde_json::from_str(PROJECTS).unwrap();
    let v = &all["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == id)
        .unwrap()["input"];
    ProjectInput {
        documents: v["documents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| SemanticInput {
                json: d["json"].as_str().unwrap().into(),
                references_version: optional(&d["references_version"]),
                versions: SemanticVersions {
                    connector: optional(&d["versions"]["connector"]),
                    annotation: optional(&d["versions"]["annotation"]),
                    view: optional(&d["versions"]["view"]),
                },
            })
            .collect(),
        representations: v["representations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| RepresentationInput {
                kind: r["kind"].as_str().unwrap().into(),
                json: r["json"].as_str().unwrap().into(),
                control_version: optional(&r["control_version"]),
                diagram_version: optional(&r["diagram_version"]),
            })
            .collect(),
    }
}
#[derive(Default)]
struct Snapshot(usize);
impl Resolver for Snapshot {
    fn resolve(&mut self, _: &str) -> Result<Option<ResolvedElement>, String> {
        self.0 += 1;
        Ok(None)
    }
}
fn ids() -> HashMap<String, String> {
    [
        ("old".into(), "new".into()),
        ("new".into(), "wrong-replay".into()),
    ]
    .into()
}
fn minimal() -> Value {
    json!({"format":FORMAT,"documents":[{"json":{},"versions":{"connector":null,"annotation":null,"view":null},"references_version":null}],
        "representations":[{"kind":"Foreign","json":null,"control_version":null,"diagram_version":null}]})
}
fn packet(payload: &str, n: usize) -> Vec<u8> {
    let row = format!(
        r#"{{"kind":"Foreign","json":{payload},"control_version":null,"diagram_version":null}}"#
    );
    format!(
        r#"{{"format":"{FORMAT}","documents":[],"representations":[{}]}}"#,
        vec![row; n].join(",")
    )
    .into_bytes()
}

#[test]
fn real_prepare_encode_decode_prepare_preserves_authoritative_wire_and_all_six_stamps() {
    let input = project("combined");
    let before = input.clone();
    let mut resolver = Snapshot::default();
    let prepared = prepare(&input, &ids(), &mut resolver).unwrap();
    assert_eq!(resolver.0, 2);
    let first = encode(&prepared).unwrap();
    let decoded = decode(&first).unwrap();
    assert_eq!(&decoded, prepared.wire());
    assert_eq!(input, before);
    let envelope: Value = serde_json::from_slice(&first).unwrap();
    assert!(envelope["documents"][0]["json"].is_object());
    assert!(envelope["representations"][0]["json"].is_object());
    for key in ["connector", "annotation", "view"] {
        assert!(envelope["documents"][0]["versions"][key].is_string());
    }
    assert!(envelope["documents"][0]["references_version"].is_string());
    for key in ["control_version", "diagram_version"] {
        assert!(envelope["representations"][0][key].is_string());
    }
    let mut replay_resolver = Snapshot::default();
    let replay = prepare(&decoded, &ids(), &mut replay_resolver).unwrap();
    assert_eq!(replay_resolver.0, 0);
    assert_eq!(replay, prepared);
    assert_eq!(encode(&replay).unwrap(), first);
    assert!(
        String::from_utf8(first)
            .unwrap()
            .contains("900719925474099312345")
    );
}

#[test]
fn foreign_first_namespace_and_pending_null_stamps_survive_authoritative_wire_replay() {
    let prepared = prepare(&project("foreign-first"), &ids(), &mut Snapshot::default()).unwrap();
    let encoded = encode(&prepared).unwrap();
    let value: Value = serde_json::from_slice(&encoded).unwrap();
    for key in ["connector", "annotation", "view"] {
        assert!(
            value["documents"][0]["versions"]
                .get(key)
                .unwrap()
                .is_null()
        );
    }
    let decoded = decode(&encoded).unwrap();
    let raw = &decoded.documents[0].json;
    assert_eq!(raw.as_bytes(), prepared.wire().documents[0].json.as_bytes());
    assert!(raw.find("\"z\":").unwrap() < raw.find("\"a\":").unwrap());
    let mut resolver = Snapshot::default();
    let replay = prepare(&decoded, &ids(), &mut resolver).unwrap();
    assert_eq!(resolver.0, 0);
    assert_eq!(replay, prepared);
}

#[test]
fn foreign_kind_unknown_stamps_and_empty_projects_round_trip_without_native_interpretation() {
    for input in [project("foreign-kind"), ProjectInput::default()] {
        let prepared = prepare(&input, &ids(), &mut Snapshot::default()).unwrap();
        let decoded = decode(&encode(&prepared).unwrap()).unwrap();
        assert_eq!(&decoded, prepared.wire());
        assert_eq!(
            prepare(&decoded, &ids(), &mut Snapshot::default()).unwrap(),
            prepared
        );
    }
    let mut future = minimal();
    future["documents"][0]["versions"]["connector"] = json!("future-selected-version");
    let decoded = decode(&future.to_string().into_bytes()).unwrap();
    assert_eq!(
        decoded.documents[0].versions.connector.as_deref(),
        Some("future-selected-version")
    );
    assert!(prepare(&decoded, &ids(), &mut Snapshot::default()).is_err());
}

#[test]
fn raw_payload_tokens_preserve_whitespace_duplicates_opaque_numbers_and_unicode_exactly() {
    let payload = "{\r\n \"ns\": {\"z\":\"foreign\", \"a\":\"sysml\"}, \"duplicate\":1, \"duplicate\":2, \"n\":900719925474099312345, \"s\":\"🚗\\n\", \"future\":[null,true]}";
    let decoded = decode(&packet(payload, 1)).unwrap();
    assert_eq!(
        decoded.representations[0].json.as_bytes(),
        payload.as_bytes()
    );
    assert!(
        decoded.representations[0]
            .json
            .contains("\"duplicate\":1, \"duplicate\":2")
    );
    // Transport accepts this opaque payload; it does not claim semantic admission.
    assert_eq!(decoded.representations[0].kind, "Foreign");
}

#[test]
fn raw_embedding_avoids_quoting_amplification_for_real_prepared_foreign_content() {
    let payload =
        json!({"text":"\"\n\\🚗".repeat(20_000),"integer":900719925474099312345u128}).to_string();
    let input = ProjectInput {
        documents: vec![],
        representations: vec![RepresentationInput {
            kind: "Foreign".into(),
            json: payload,
            control_version: Some("future".into()),
            diagram_version: None,
        }],
    };
    let prepared = prepare(&input, &ids(), &mut Snapshot::default()).unwrap();
    let raw = &prepared.wire().representations[0].json;
    let encoded = encode(&prepared).unwrap();
    assert!(encoded.len() < raw.len() + 256);
    assert_eq!(decode(&encoded).unwrap(), *prepared.wire());
    assert!(
        encoded
            .windows(raw.len())
            .any(|bytes| bytes == raw.as_bytes())
    );
}

#[test]
fn every_nullable_stamp_is_required_and_unknown_envelope_metadata_fields_refuse() {
    let original = minimal();
    for (pointer, key) in [
        ("/documents/0/versions", "connector"),
        ("/documents/0/versions", "annotation"),
        ("/documents/0/versions", "view"),
        ("/documents/0", "references_version"),
        ("/representations/0", "control_version"),
        ("/representations/0", "diagram_version"),
    ] {
        let mut input = original.clone();
        input
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert!(
            decode(input.to_string().as_bytes()).is_err(),
            "{pointer}/{key}"
        );
    }
    for (pointer, key) in [
        ("", "extra"),
        ("/documents/0", "id"),
        ("/documents/0/versions", "extra"),
        ("/representations/0", "resource_id"),
    ] {
        let mut input = original.clone();
        input
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(key.into(), json!(true));
        assert!(
            decode(input.to_string().as_bytes()).is_err(),
            "{pointer}/{key}"
        );
    }
    for key in ["format", "documents", "representations"] {
        let mut input = original.clone();
        input.as_object_mut().unwrap().remove(key);
        assert!(decode(input.to_string().as_bytes()).is_err());
    }
    assert!(decode(original.to_string().as_bytes()).is_ok());
}

#[test]
fn duplicate_metadata_refuses_even_when_the_first_value_is_explicit_null() {
    let original = minimal().to_string();
    for pair in [
        format!("\"format\":\"{FORMAT}\""),
        "\"connector\":null".into(),
        "\"references_version\":null".into(),
        "\"control_version\":null".into(),
        "\"diagram_version\":null".into(),
        "\"kind\":\"Foreign\"".into(),
        "\"json\":{}".into(),
    ] {
        let duplicate = original.replacen(&pair, &format!("{pair},{pair}"), 1);
        assert_ne!(duplicate, original);
        assert!(decode(duplicate.as_bytes()).is_err(), "{pair}");
    }
}

#[test]
fn future_format_wrong_metadata_types_trailing_and_late_failures_leave_no_partial_result() {
    let prepared = prepare(&project("combined"), &ids(), &mut Snapshot::default()).unwrap();
    let published = encode(&prepared).unwrap();
    let before = published.clone();
    let mut future = minimal();
    future["format"] = json!("syson-project-wire/2");
    assert_eq!(
        decode(future.to_string().as_bytes()),
        Err(Error::UnsupportedFormat)
    );
    for pointer in [
        "/format",
        "/documents/0/versions/connector",
        "/documents/0/references_version",
        "/representations/0/kind",
        "/representations/0/control_version",
        "/representations/0/diagram_version",
    ] {
        let mut bad = minimal();
        *bad.pointer_mut(pointer).unwrap() = json!(42);
        assert!(decode(bad.to_string().as_bytes()).is_err());
    }
    for bad in [b"{".as_slice(), b"null", b"[]", b"{}"] {
        assert!(decode(bad).is_err());
    }
    let mut trailing = published.clone();
    trailing.extend_from_slice(b" true");
    assert!(decode(&trailing).is_err());
    let mut late = published.clone();
    late.pop();
    assert!(decode(&late).is_err());
    assert_eq!(published, before);
    assert_eq!(encode(&prepared).unwrap(), published);
}

#[test]
fn exact_payload_aggregate_item_metadata_and_envelope_byte_caps_are_enforced() {
    let exact = format!("\"{}\"", "x".repeat(MAX_PAYLOAD_BYTES - 2));
    assert_eq!(
        decode(&packet(&exact, 1)).unwrap().representations[0]
            .json
            .len(),
        MAX_PAYLOAD_BYTES
    );
    let over = format!("\"{}\"", "x".repeat(MAX_PAYLOAD_BYTES - 1));
    assert!(matches!(
        decode(&packet(&over, 1)),
        Err(Error::Limit("payload bytes"))
    ));
    assert_eq!(decode(&packet(&exact, 8)).unwrap().representations.len(), 8);
    assert!(matches!(
        decode(&packet(&exact, 9)),
        Err(Error::Limit("aggregate payload bytes"))
    ));
    assert_eq!(
        decode(&packet("null", 64)).unwrap().representations.len(),
        64
    );
    assert!(decode(&packet("null", 65)).is_err());
    for pointer in [
        "/documents/0/versions/connector",
        "/representations/0/control_version",
        "/representations/0/kind",
    ] {
        let mut bad = minimal();
        *bad.pointer_mut(pointer).unwrap() = json!("🚗".repeat(MAX_METADATA_BYTES / 4 + 1));
        assert!(matches!(
            decode(bad.to_string().as_bytes()),
            Err(Error::Limit(_))
        ));
    }
    assert!(matches!(
        decode(&vec![b' '; MAX_ENVELOPE_BYTES + 1]),
        Err(Error::Limit("envelope bytes"))
    ));
}

#[test]
fn literal_payload_transport_prepares_and_replays_without_private_marker_coercion() {
    let payload = r#"{"number":{"$serde_json::private::Number":"not numeric"},"raw":{"$serde_json::private::RawValue":"{\"future\":true}"},"n":1E+000,"duplicate":1,"duplicate":2}"#;
    let decoded = decode(&packet(payload, 1)).unwrap();
    assert_eq!(decoded.representations[0].json, payload);
    let input_before = decoded.clone();
    let prepared = prepare(&decoded, &ids(), &mut Snapshot::default()).unwrap();
    let mut number = serde_json::Map::new();
    number.insert(
        "$serde_json::private::Number".into(),
        Value::String("not numeric".into()),
    );
    let mut raw = serde_json::Map::new();
    raw.insert(
        "$serde_json::private::RawValue".into(),
        Value::String(r#"{"future":true}"#.into()),
    );
    let value = &prepared.project().representations[0].content;
    assert_eq!(value["number"], Value::Object(number));
    assert_eq!(value["raw"], Value::Object(raw));
    assert_eq!(value["n"].to_string(), "1E+000");
    assert_eq!(value["duplicate"], 2);
    let wire = encode(&prepared).unwrap();
    let replay_input = decode(&wire).unwrap();
    assert_eq!(&replay_input, prepared.wire());
    let mut resolver = Snapshot::default();
    let replay = prepare(&replay_input, &ids(), &mut resolver).unwrap();
    assert_eq!(replay, prepared);
    assert_eq!(resolver.0, 0);
    assert_eq!(encode(&replay).unwrap(), wire);
    assert_eq!(decoded, input_before);
}
