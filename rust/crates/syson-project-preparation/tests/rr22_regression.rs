// SPDX-License-Identifier: EPL-2.0
//! Manual literal oracles through the real preparation and wire consumers.
use serde_json::{Map, Value};
use std::collections::HashMap;
use syson_migration_pipeline::Resolver;
use syson_migrations::{b25a, b25b, b25c, b25d};
use syson_persistence::{DIAGRAM_VERSION, REFERENCES_VERSION, ResolvedElement};
use syson_project_preparation::{
    PreparedProject, ProjectInput, RepresentationInput, SemanticInput, SemanticVersions, prepare,
};
use syson_project_wire::{decode, encode};
#[derive(Default)]
struct Snapshot(Vec<String>);
impl Resolver for Snapshot {
    fn resolve(&mut self, id: &str) -> Result<Option<ResolvedElement>, String> {
        self.0.push(id.into());
        Ok(None)
    }
}
fn input(case: &str) -> ProjectInput {
    // Only the unchanged ordinary fixture wrapper crosses a generic parser.
    // Expected marker objects below are constructed manually, never parsed.
    let packet: Value = serde_json::from_str(include_str!(
        "../../../../porting/rust-parity-evidence/C03b/project-fixtures.json"
    ))
    .unwrap();
    let rows = packet["cases"].as_array().unwrap();
    let value = &rows.iter().find(|row| row["id"] == case).unwrap()["input"];
    ProjectInput {
        documents: value["documents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| SemanticInput {
                json: d["json"].as_str().unwrap().into(),
                versions: SemanticVersions::default(),
                references_version: None,
            })
            .collect(),
        representations: value["representations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| RepresentationInput {
                kind: r["kind"].as_str().unwrap().into(),
                json: r["json"].as_str().unwrap().into(),
                control_version: r["control_version"].as_str().map(str::to_owned),
                diagram_version: r["diagram_version"].as_str().map(str::to_owned),
            })
            .collect(),
    }
}
fn object(key: &str, text: &str) -> Value {
    let mut map = Map::new();
    map.insert(key.into(), Value::String(text.into()));
    Value::Object(map)
}
fn inject(raw: &str, members: &str) -> String {
    format!("{{{members},{}", raw.strip_prefix('{').unwrap())
}
fn ids() -> HashMap<String, String> {
    HashMap::from([
        ("old".into(), "new".into()),
        ("new".into(), "must-not-replay".into()),
    ])
}
fn roundtrip(input: &ProjectInput) -> PreparedProject {
    let before = input.clone();
    let mut resolver = Snapshot::default();
    let prepared = prepare(input, &ids(), &mut resolver).unwrap();
    assert_eq!(resolver.0, ["sysmllibrary:/Types#old", "old"]);
    let wire = encode(&prepared).unwrap();
    let decoded = decode(&wire).unwrap();
    assert_eq!(&decoded, prepared.wire());
    let mut replay_resolver = Snapshot::default();
    let replay = prepare(&decoded, &ids(), &mut replay_resolver).unwrap();
    assert_eq!(replay, prepared);
    assert!(replay_resolver.0.is_empty());
    assert_eq!(encode(&replay).unwrap(), wire);
    assert_eq!(input, &before);
    let doc = &prepared.wire().documents[0];
    assert_eq!(doc.versions.connector.as_deref(), Some(b25b::VERSION));
    assert_eq!(doc.versions.annotation.as_deref(), Some(b25a::VERSION));
    assert_eq!(doc.versions.view.as_deref(), Some(b25c::VERSION));
    assert_eq!(doc.references_version.as_deref(), Some(REFERENCES_VERSION));
    let rep = &prepared.wire().representations[0];
    assert_eq!(rep.control_version.as_deref(), Some(b25d::VERSION));
    assert_eq!(rep.diagram_version.as_deref(), Some(DIAGRAM_VERSION));
    let doc = &prepared.project().documents[0].document;
    assert_eq!(
        doc.element("typing-semantic").unwrap()["data"]["type"],
        "sysml:Type sysmllibrary:/Types#new"
    );
    assert_eq!(
        doc.element("typing-semantic").unwrap()["data"]["typedFeature"],
        "usage-node"
    );
    prepared
}
#[test]
fn semantic_preflight_accepts_non_numeric_private_number_object_through_wire_replay() {
    let mut input = input("combined");
    input.documents[0].json = inject(
        &input.documents[0].json,
        r#""literal":{"$serde_json::private::Number":"literal text, not a number"},"nested":{"literal":{"$serde_json::private::RawValue":"{\"trap\":true}"}}"#,
    );
    let prepared = roundtrip(&input);
    let value = prepared.project().documents[0].document.value();
    assert_eq!(
        value["literal"],
        object("$serde_json::private::Number", "literal text, not a number")
    );
    assert_eq!(
        value["nested"]["literal"],
        object("$serde_json::private::RawValue", r#"{"trap":true}"#)
    );
}
#[test]
fn representation_literal_numeric_lookalike_stays_an_object_through_wire_replay() {
    let mut input = input("combined");
    input.representations[0].json = inject(
        &input.representations[0].json,
        r#""literal":{"$serde_json::private::Number":"123"},"nested":{"literal":{"$serde_json::private::RawValue":"{\"trap\":true}"}}"#,
    );
    let prepared = roundtrip(&input);
    let value = &prepared.project().representations[0].content;
    assert_eq!(
        value["literal"],
        object("$serde_json::private::Number", "123")
    );
    assert_eq!(
        value["nested"]["literal"],
        object("$serde_json::private::RawValue", r#"{"trap":true}"#)
    );
}

#[test]
fn actual_both_lanes_keep_nested_literal_objects_and_original_numeric_tokens() {
    for token in [
        "-0",
        "0.00",
        "1E+000",
        "1e-99999",
        "900719925474099312345678901234567890",
    ] {
        let members = format!(
            r#""bag":[{{"$serde_json::private::Number":"opaque"}},{{"nested":{{"$serde_json::private::RawValue":"{{\"trap\":true}}"}}}}],"n":{token},"label":"雪😀""#
        );
        let mut input = input("combined");
        input.documents[0].json = inject(&input.documents[0].json, &members);
        input.representations[0].json = inject(&input.representations[0].json, &members);
        let prepared = roundtrip(&input);
        for value in [
            prepared.project().documents[0].document.value(),
            &prepared.project().representations[0].content,
        ] {
            assert_eq!(
                value["bag"][0],
                object("$serde_json::private::Number", "opaque")
            );
            assert_eq!(
                value["bag"][1]["nested"],
                object("$serde_json::private::RawValue", r#"{"trap":true}"#)
            );
            assert_eq!(value["n"].to_string(), token);
            assert_eq!(value["label"], "雪😀");
        }
        assert!(
            prepared.wire().documents[0]
                .json
                .contains(&format!(r#""n":{token}"#))
        );
        assert!(
            prepared.wire().representations[0]
                .json
                .contains(&format!(r#""n":{token}"#))
        );
    }
}
fn foreign(json: String) -> ProjectInput {
    ProjectInput {
        documents: vec![],
        representations: vec![RepresentationInput {
            kind: "foreign".into(),
            json,
            control_version: Some("future-control".into()),
            diagram_version: Some("future-diagram".into()),
        }],
    }
}
fn prepare_foreign(json: String) -> PreparedProject {
    let input = foreign(json);
    let before = input.clone();
    let mut resolver = Snapshot::default();
    let output = prepare(&input, &ids(), &mut resolver).unwrap();
    assert!(resolver.0.is_empty());
    assert_eq!(input, before);
    assert_eq!(
        output.wire().representations[0].control_version,
        before.representations[0].control_version
    );
    assert_eq!(
        output.wire().representations[0].diagram_version,
        before.representations[0].diagram_version
    );
    let decoded = decode(&encode(&output).unwrap()).unwrap();
    assert_eq!(&decoded, output.wire());
    assert_eq!(
        prepare(&decoded, &ids(), &mut Snapshot::default()).unwrap(),
        output
    );
    output
}
#[test]
fn last_wins_preparation_keeps_namespace_order_and_strict_model_policy() {
    let mut input = input("foreign-first");
    input.documents[0].json = input.documents[0].json.replacen(
        r#""ns":{"z":"urn:foreign","a":"http://www.eclipse.org/syson/sysml""#,
        r#""ns":{"z":"discarded","a":"http://www.eclipse.org/syson/sysml","\u007a":"urn:foreign""#,
        1,
    );
    input.documents[0].json = inject(
        &input.documents[0].json,
        r#""dup":{"$serde_json::private::Number":"discarded"},"\u0064up":{"$serde_json::private::RawValue":"final literal"}"#,
    );
    let before = input.clone();
    let prepared = prepare(&input, &ids(), &mut Snapshot::default()).unwrap();
    let value = prepared.project().documents[0].document.value();
    assert_eq!(
        value["dup"],
        object("$serde_json::private::RawValue", "final literal")
    );
    assert_eq!(
        prepared.wire().documents[0].versions,
        SemanticVersions::default()
    );
    let raw = &prepared.wire().documents[0].json;
    assert!(
        raw.find(r#""z":"urn:foreign""#).unwrap()
            < raw
                .find(r#""a":"http://www.eclipse.org/syson/sysml""#)
                .unwrap()
    );
    let decoded = decode(&encode(&prepared).unwrap()).unwrap();
    let mut replay_resolver = Snapshot::default();
    assert_eq!(
        prepare(&decoded, &ids(), &mut replay_resolver).unwrap(),
        prepared
    );
    assert!(replay_resolver.0.is_empty());
    assert_eq!(input, before);
    // The opt-in migration policy never changes the strict public model API.
    assert!(matches!(
        syson_model::Document::from_json(r#"{"x":1,"x":2}"#),
        Err(syson_model::ModelError::JsonAdmission(
            syson_json_value::Error::DuplicateKey
        ))
    ));
}
#[test]
fn physical_before_collapse_and_depth_and_byte_boundaries_are_explicit() {
    let exact = format!("\"{}\"", "x".repeat(syson_migrations::MAX_BYTES - 2));
    assert_eq!(
        prepare_foreign(exact.clone()).wire().representations[0].json,
        exact
    );
    let values = |n: usize| format!("[{}]", vec!["null"; n].join(","));
    assert!(
        prepare_foreign(values(syson_migrations::MAX_VALUES - 1))
            .project()
            .representations[0]
            .content
            .is_array()
    );
    let duplicates = |n: usize| format!("{{{}}}", vec![r#""k":null"#; n].join(","));
    assert_eq!(
        prepare_foreign(duplicates(syson_migrations::MAX_VALUES - 1))
            .wire()
            .representations[0]
            .json,
        r#"{"k":null}"#
    );
    let nested = |n: usize| format!("{}null{}", "[".repeat(n), "]".repeat(n));
    prepare_foreign(nested(syson_migrations::MAX_DEPTH));
    for raw in [
        format!("{exact} "),
        values(syson_migrations::MAX_VALUES),
        duplicates(syson_migrations::MAX_VALUES),
        nested(syson_migrations::MAX_DEPTH + 1),
        format!(
            r#"{{"k":{},"k":null}}"#,
            nested(syson_migrations::MAX_DEPTH)
        ),
    ] {
        let input = foreign(raw);
        let before = input.clone();
        let mut resolver = Snapshot::default();
        assert!(matches!(
            prepare(&input, &ids(), &mut resolver),
            Err(syson_project_preparation::Error::Limit)
        ));
        assert!(resolver.0.is_empty());
        assert_eq!(input, before);
    }
    for raw in [r#"{"k":tru,"k":null}"#, r#""\uD800""#] {
        let mut resolver = Snapshot::default();
        assert!(matches!(
            prepare(&foreign(raw.into()), &ids(), &mut resolver),
            Err(syson_project_preparation::Error::Admission {
                kind: "representation",
                index: 0,
                ..
            })
        ));
        assert!(resolver.0.is_empty());
    }
}
#[test]
fn late_failure_keeps_literal_input_previous_wire_and_private_stamps_atomic() {
    let mut good = input("combined");
    good.documents[0].json = inject(
        &good.documents[0].json,
        r#""literal":{"$serde_json::private::Number":"opaque"}"#,
    );
    let published = roundtrip(&good);
    let published_wire = encode(&published).unwrap();
    let mut late_doc = good.clone();
    let mut invalid = late_doc.documents[0].clone();
    invalid.json = invalid
        .json
        .replacen(r#""id":"package-node""#, r#""id":false"#, 1);
    late_doc.documents.push(invalid);
    let original = late_doc.clone();
    let mut resolver = Snapshot::default();
    assert!(matches!(
        prepare(&late_doc, &ids(), &mut resolver),
        Err(syson_project_preparation::Error::Admission {
            kind: "document",
            index: 1,
            ..
        })
    ));
    assert!(resolver.0.is_empty());
    assert_eq!(late_doc, original);
    let mut late_rep = good.clone();
    late_rep.representations[0].json = inject(&late_rep.representations[0].json, r#""z":[[]]"#);
    let original = late_rep.clone();
    let mut resolver = Snapshot::default();
    assert!(matches!(
        prepare(&late_rep, &ids(), &mut resolver),
        Err(syson_project_preparation::Error::Representation {
            index: 0,
            cause: b25d::Error::NestedArray
        })
    ));
    assert_eq!(resolver.0, ["sysmllibrary:/Types#old", "old"]);
    assert_eq!(late_rep, original);
    let mut late_json = good.clone();
    late_json.representations.push(RepresentationInput {
        kind: "foreign".into(),
        json: "{".into(),
        control_version: None,
        diagram_version: None,
    });
    let original = late_json.clone();
    let mut resolver = Snapshot::default();
    assert!(matches!(
        prepare(&late_json, &ids(), &mut resolver),
        Err(syson_project_preparation::Error::Admission {
            kind: "representation",
            index: 1,
            ..
        })
    ));
    assert!(resolver.0.is_empty());
    assert_eq!(late_json, original);
    assert_eq!(encode(&published).unwrap(), published_wire);
}

#[test]
fn marker_keys_in_first_and_later_member_positions_keep_their_other_members() {
    for (raw, marker, text, ordinary, flag) in [
        (
            r#"{"$serde_json::private::Number":"opaque","after":true}"#,
            "$serde_json::private::Number",
            "opaque",
            "after",
            true,
        ),
        (
            r#"{"before":false,"$serde_json::private::Number":"opaque"}"#,
            "$serde_json::private::Number",
            "opaque",
            "before",
            false,
        ),
        (
            r#"{"$serde_json::private::RawValue":"quoted text","after":true}"#,
            "$serde_json::private::RawValue",
            "quoted text",
            "after",
            true,
        ),
        (
            r#"{"before":false,"$serde_json::private::RawValue":"quoted text"}"#,
            "$serde_json::private::RawValue",
            "quoted text",
            "before",
            false,
        ),
    ] {
        let mut input = input("combined");
        let members = format!(r#""placed":{raw}"#);
        input.documents[0].json = inject(&input.documents[0].json, &members);
        input.representations[0].json = inject(&input.representations[0].json, &members);
        let prepared = roundtrip(&input);
        let mut expected = Map::new();
        expected.insert(marker.into(), Value::String(text.into()));
        expected.insert(ordinary.into(), Value::Bool(flag));
        let expected = Value::Object(expected);
        assert_eq!(
            prepared.project().documents[0].document.value()["placed"],
            expected
        );
        assert_eq!(
            prepared.project().representations[0].content["placed"],
            expected
        );
    }
}
