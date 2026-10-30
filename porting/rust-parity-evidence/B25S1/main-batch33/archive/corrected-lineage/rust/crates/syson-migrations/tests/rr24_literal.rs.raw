// SPDX-License-Identifier: EPL-2.0
//! Baseline-compatible manual/raw oracles: never parse expected marker objects.
use serde_json::{Map, Value};
use syson_migrations::{self as migrations, Error, SYSML_URI, b25a, b25b, b25c, b25d};

const MARKERS: &str = r#""numberMarker":{"$serde_json::private::Number":"123"},"rawMarker":{"$serde_json::private::RawValue":"{\"eClass\":\"sysml:ViewUsage\",\"data\":{\"exposedElement\":\"literal text only\"}}"}"#;
fn expected_marker(key: &str, value: &str) -> Value {
    let mut map = Map::new();
    map.insert(key.into(), Value::String(value.into()));
    Value::Object(map)
}
fn assert_markers(value: &Value) {
    assert_eq!(
        value["numberMarker"],
        expected_marker("$serde_json::private::Number", "123")
    );
    assert_eq!(
        value["rawMarker"],
        expected_marker(
            "$serde_json::private::RawValue",
            r#"{"eClass":"sysml:ViewUsage","data":{"exposedElement":"literal text only"}}"#
        )
    );
}
fn semantic(markers: &str) -> String {
    format!(
        r#"{{"ns":{{"z":"{SYSML_URI}","a":"foreign"}},{markers},"content":[{{"eClass":"sysml:ConnectorAsUsage","data":{{"source":"legacy-source","target":"legacy-target","keep":"connector"}}}},{{"eClass":"sysml:Annotation","data":{{"annotatingElement":"legacy","keep":"annotation"}}}},{{"eClass":"sysml:AnnotatingElement","data":{{"annotation":"legacy","keep":"annotating"}}}},{{"eClass":"sysml:ViewUsage","data":{{"exposedElement":"legacy","keep":"view"}}}}]}}"#
    )
}
#[test]
fn every_semantic_participant_keeps_private_marker_objects_through_ordered_reentry() {
    let input = semantic(MARKERS);
    let before = input.clone();
    let mut bytes = input.clone();
    type Participant = fn(&str) -> Result<migrations::Prepared, Error>;
    for (participant, version, index, removed) in [
        (b25b::prepare as Participant, b25b::VERSION, 0, "source"),
        (
            b25a::prepare as Participant,
            b25a::VERSION,
            1,
            "annotatingElement",
        ),
        (
            b25c::prepare as Participant,
            b25c::VERSION,
            3,
            "exposedElement",
        ),
    ] {
        let output = participant(&bytes).unwrap();
        assert!(output.matched_resource);
        assert_eq!(output.participant_version, version);
        assert_markers(&output.json);
        assert!(output.json["content"][index]["data"].get(removed).is_none());
        assert_eq!(
            output.json["content"][index]["data"]["keep"]
                .as_str()
                .unwrap(),
            ["connector", "annotation", "annotating", "view"][index]
        );
        bytes = output.to_json().unwrap();
        assert!(bytes.contains(r#""ns":{"z":"http://www.eclipse.org/syson/sysml","a":"foreign"}"#));
        assert_markers(&migrations::prepare(&bytes, "inspection", &[]).unwrap().json);
    }
    assert_eq!(input, before);
}
#[test]
fn action_control_participant_keeps_marker_objects_in_matching_foreign_and_replay_modes() {
    let input = format!(
        r#"{{{MARKERS},"descriptionId":"{}","label":"雪😀"}}"#,
        b25d::MAPPINGS[0].0
    );
    let before = input.clone();
    let output = b25d::prepare(b25d::KIND, &input, None).unwrap();
    assert_markers(&output.json);
    assert_eq!(output.json["descriptionId"], b25d::MAPPINGS[0].1);
    assert_eq!(output.participant_version.as_deref(), Some(b25d::VERSION));
    let replay =
        b25d::prepare(b25d::KIND, &output.to_json().unwrap(), Some(b25d::VERSION)).unwrap();
    assert!(!replay.changed);
    assert_markers(&replay.json);
    let foreign = b25d::prepare("foreign", &input, Some("foreign-stamp")).unwrap();
    assert!(!foreign.matched_kind && !foreign.changed);
    assert_eq!(
        foreign.participant_version.as_deref(),
        Some("foreign-stamp")
    );
    assert_markers(&foreign.json);
    assert_eq!(foreign.json["descriptionId"], b25d::MAPPINGS[0].0);
    assert_eq!(input, before);
}
#[test]
fn namespace_first_value_is_literal_before_the_resource_predicate() {
    for first in [
        r#"{"$serde_json::private::Number":"123"}"#.to_owned(),
        format!(r#"{{"$serde_json::private::RawValue":"\"{SYSML_URI}\""}}"#),
    ] {
        let input = format!(r#"{{"ns":{{"z":{first},"a":"{SYSML_URI}"}}}}"#);
        assert_eq!(
            migrations::prepare(&input, "inspection", &[]),
            Err(Error::NamespaceShape)
        );
    }
}
#[test]
fn all_four_participants_keep_exact_numeric_token_spelling() {
    let huge = "9".repeat(16_384);
    for token in ["-0", "0.00", "1E+000", "1e-99999", huge.as_str()] {
        let input = format!(r#"{{"ns":{{"s":"{SYSML_URI}"}},"n":{token}}}"#);
        for participant in [b25b::prepare, b25a::prepare, b25c::prepare] {
            let output = participant(&input).unwrap();
            let Value::Number(number) = &output.json["n"] else {
                panic!("number expected")
            };
            assert_eq!(number.to_string(), token);
            assert!(
                output
                    .to_json()
                    .unwrap()
                    .contains(&format!(r#""n":{token}"#))
            );
        }
        let output = b25d::prepare(b25d::KIND, &input, None).unwrap();
        let Value::Number(number) = &output.json["n"] else {
            panic!("number expected")
        };
        assert_eq!(number.to_string(), token);
        assert!(
            output
                .to_json()
                .unwrap()
                .contains(&format!(r#""n":{token}"#))
        );
    }
}

#[test]
fn duplicate_namespace_first_position_uses_final_decoded_key_value_without_sorting() {
    for (first, last, matched) in [("foreign", SYSML_URI, true), (SYSML_URI, "foreign", false)] {
        let input =
            format!(r#"{{"ns":{{"z":"{first}","a":"foreign","\u007a":"{last}"}},{MARKERS}}}"#);
        let output = migrations::prepare(&input, "inspection", &[]).unwrap();
        assert_eq!(output.matched_resource, matched);
        assert_markers(&output.json);
        let bytes = output.to_json().unwrap();
        assert!(bytes.contains(&format!(r#""ns":{{"z":"{last}","a":"foreign"}}"#)));
        assert_eq!(
            migrations::prepare(&bytes, "reentry", &[])
                .unwrap()
                .matched_resource,
            matched
        );
    }
    let input = format!(r#"{{"ns":{{"s":"{SYSML_URI}"}},"ns":{{"s":"foreign"}}}}"#);
    assert!(
        !migrations::prepare(&input, "inspection", &[])
            .unwrap()
            .matched_resource
    );
}
#[test]
fn byte_depth_and_physical_value_edges_remain_bounded_before_materialization() {
    use migrations::{MAX_BYTES, MAX_DEPTH, MAX_VALUES};
    let mut exact = "{}".to_owned();
    exact.push_str(&" ".repeat(MAX_BYTES - 2));
    assert!(migrations::prepare(&exact, "edge", &[]).is_ok());
    assert!(b25d::prepare(b25d::KIND, &exact, None).is_ok());
    exact.push(' ');
    assert_eq!(migrations::prepare(&exact, "edge", &[]), Err(Error::Limit));
    assert_eq!(
        b25d::prepare(b25d::KIND, &exact, None),
        Err(b25d::Error::Limit)
    );
    for extra in [0, 1] {
        let n = MAX_DEPTH - 1 + extra;
        let deep = format!(r#"{{"v":{}0{}}}"#, "[".repeat(n), "]".repeat(n));
        assert_eq!(migrations::prepare(&deep, "edge", &[]).is_ok(), extra == 0);
        // Foreign kind avoids the selected participant's deliberate NestedArray rule.
        assert_eq!(b25d::prepare("foreign", &deep, None).is_ok(), extra == 0);
        let n = MAX_VALUES - 2 + extra;
        let many = format!(r#"{{"v":[{}]}}"#, vec!["null"; n].join(","));
        assert_eq!(migrations::prepare(&many, "edge", &[]).is_ok(), extra == 0);
        assert_eq!(b25d::prepare("foreign", &many, None).is_ok(), extra == 0);
        // Deliberate admission tightening: count overwritten physical values,
        // even though the resulting last-wins object has only one member.
        let duplicate = format!(
            "{{{}}}",
            vec![r#""x":null"#; MAX_VALUES - 1 + extra].join(",")
        );
        assert_eq!(
            migrations::prepare(&duplicate, "edge", &[]).is_ok(),
            extra == 0
        );
        assert_eq!(
            b25d::prepare("foreign", &duplicate, None).is_ok(),
            extra == 0
        );
    }
}
#[test]
fn literal_shape_and_late_failures_publish_no_partial_migration() {
    let input = semantic(MARKERS);
    let published = b25b::prepare(&input).unwrap();
    let previous = published.to_json().unwrap();
    let passes = [
        migrations::Removal {
            class: "ConnectorAsUsage",
            features: &["source"],
        },
        migrations::Removal {
            class: "NotAClass",
            features: &["target"],
        },
    ];
    assert_eq!(
        migrations::prepare(&input, "late", &passes),
        Err(Error::UnknownClass("NotAClass".into()))
    );
    assert_eq!(published.to_json().unwrap(), previous);
    let late = format!(
        r#"{{{MARKERS},"a":{{"descriptionId":"{}"}},"z":[[]]}}"#,
        b25d::MAPPINGS[0].0
    );
    let before = late.clone();
    assert_eq!(
        b25d::prepare(b25d::KIND, &late, None),
        Err(b25d::Error::NestedArray)
    );
    assert_eq!(late, before);
    assert_eq!(published.to_json().unwrap(), previous);
    for invalid in ["{", "[]", "null", r#"{"ns":{"s":null}}"#] {
        assert!(migrations::prepare(invalid, "shape", &[]).is_err());
    }
    for foreign in [
        r#"{"ns":false}"#,
        r#"{"ns":{"s":true}}"#,
        r#"{"ns":{"s":1e99999}}"#,
    ] {
        assert!(
            !migrations::prepare(foreign, "shape", &[])
                .unwrap()
                .matched_resource
        );
    }
}

use std::collections::HashMap;
use syson_migration_pipeline::Resolver;
use syson_persistence::{DIAGRAM_VERSION, REFERENCES_VERSION, ResolvedElement};
use syson_project_preparation::{
    self as project, ProjectInput, RepresentationInput, SemanticInput, SemanticVersions,
};
#[derive(Default)]
struct NoLoad(usize);
impl Resolver for NoLoad {
    fn resolve(&mut self, _: &str) -> Result<Option<ResolvedElement>, String> {
        self.0 += 1;
        Ok(None)
    }
}
fn combined(markers: Option<&str>) -> ProjectInput {
    // Existing ordinary fixture is read before injecting our independent raw
    // marker text. No expected marker object crosses a generic Value parser.
    let packet: Value = serde_json::from_str(include_str!(
        "../../../../porting/rust-parity-evidence/C03b/project-fixtures.json"
    ))
    .unwrap();
    let input = &packet["cases"][0]["input"];
    let add = |raw: &str, lane: &str| {
        if markers == Some(lane) {
            format!("{{{MARKERS},{}", raw.strip_prefix('{').unwrap())
        } else {
            raw.to_owned()
        }
    };
    ProjectInput {
        documents: vec![SemanticInput {
            json: add(input["documents"][0]["json"].as_str().unwrap(), "semantic"),
            versions: SemanticVersions::default(),
            references_version: None,
        }],
        representations: vec![RepresentationInput {
            kind: b25d::KIND.into(),
            json: add(
                input["representations"][0]["json"].as_str().unwrap(),
                "representation",
            ),
            control_version: None,
            diagram_version: None,
        }],
    }
}
fn ids() -> HashMap<String, String> {
    HashMap::from([("old".into(), "new".into())])
}
#[test]
fn actual_four_participant_recipe_and_wire_replay_keep_independent_stamps_and_ids() {
    let input = combined(None);
    let before = input.clone();
    let output = project::prepare(&input, &ids(), &mut NoLoad::default()).unwrap();
    let document = &output.project().documents[0].document;
    for (element, field) in [
        ("connection-semantic", "source"),
        ("connection-semantic", "target"),
        ("annotation-semantic", "annotatingElement"),
        ("comment-semantic", "annotation"),
        ("view-semantic", "exposedElement"),
    ] {
        assert!(
            document.element(element).unwrap()["data"]
                .get(field)
                .is_none()
        );
    }
    let stamps = &output.wire().documents[0];
    assert_eq!(stamps.versions.connector.as_deref(), Some(b25b::VERSION));
    assert_eq!(stamps.versions.annotation.as_deref(), Some(b25a::VERSION));
    assert_eq!(stamps.versions.view.as_deref(), Some(b25c::VERSION));
    assert_eq!(
        stamps.references_version.as_deref(),
        Some(REFERENCES_VERSION)
    );
    let representation = &output.wire().representations[0];
    assert_eq!(
        representation.control_version.as_deref(),
        Some(b25d::VERSION)
    );
    assert_eq!(
        representation.diagram_version.as_deref(),
        Some(DIAGRAM_VERSION)
    );
    assert_eq!(
        output.project().representations[0].content["nodes"][0]["descriptionId"],
        b25d::MAPPINGS[0].1
    );
    assert_eq!(
        output.project().representations[0].content["nodes"][0]["targetObjectId"],
        "new"
    );
    let encoded = syson_project_wire::encode(&output).unwrap();
    let decoded = syson_project_wire::decode(&encoded).unwrap();
    assert_eq!(&decoded, output.wire());
    let mut resolver = NoLoad::default();
    let replay = project::prepare(&decoded, &ids(), &mut resolver).unwrap();
    assert_eq!(replay, output);
    assert_eq!(resolver.0, 0);
    assert_eq!(input, before);
}
fn caller_gate(lane: &str) {
    let input = combined(Some(lane));
    let before = input.clone();
    let output = project::prepare(&input, &ids(), &mut NoLoad::default()).unwrap();
    let encoded = syson_project_wire::encode(&output).unwrap();
    let decoded = syson_project_wire::decode(&encoded).unwrap();
    assert_eq!(&decoded, output.wire());
    let replay = project::prepare(&decoded, &ids(), &mut NoLoad::default()).unwrap();
    assert_eq!(input, before);
    if lane == "semantic" {
        assert_markers(output.project().documents[0].document.value());
        assert_markers(replay.project().documents[0].document.value());
    } else {
        assert_markers(&output.project().representations[0].content);
        assert_markers(&replay.project().representations[0].content);
    }
}
#[test]
#[ignore = "RR22 preparation/model materialization is a separate OPEN caller acceptance gate"]
fn rr22_semantic_preparation_wire_literal_gate() {
    caller_gate("semantic");
}
#[test]
#[ignore = "RR22 representation materialization is a separate OPEN caller acceptance gate"]
fn rr22_representation_preparation_wire_literal_gate() {
    caller_gate("representation");
}
