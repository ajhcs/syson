// SPDX-License-Identifier: EPL-2.0
use serde_json::{Map, Value};
use std::collections::HashMap;
use syson_migration_pipeline::{
    self as library, Project, Representation, Resolver, SemanticDocument,
};
use syson_migrations::{Error, MAX_BYTES, MAX_DEPTH, MAX_VALUES, Removal, SYSML_URI, prepare};
use syson_model::Document;
use syson_persistence::{DIAGRAM_KIND, DIAGRAM_VERSION, REFERENCES_VERSION, ResolvedElement};

const ENCODED: &str = r#"{"eClass":"sysml:ViewUsage","data":{"exposedElement":"phantom"}}"#;
const OPAQUE: &str = r#""number":{"$serde_json::private::Number":"not numeric"},"raw":{"$serde_json::private::RawValue":"{\"eClass\":\"sysml:ViewUsage\",\"data\":{\"exposedElement\":\"phantom\"}}"},"n":-0,"exp":1E+400"#;
fn object(key: &str, value: &str) -> Value {
    let mut expected = Map::new();
    expected.insert(key.into(), Value::String(value.into()));
    Value::Object(expected)
}
fn opaque(value: &Value) {
    assert_eq!(
        value["number"],
        object("$serde_json::private::Number", "not numeric")
    );
    assert_eq!(
        value["raw"],
        object("$serde_json::private::RawValue", ENCODED)
    );
    assert_eq!(value["n"].to_string(), "-0");
    assert_eq!(value["exp"].to_string(), "1E+400");
}
#[test]
fn manual_objects_and_numeric_tokens_survive_each_shared_transport_boundary() {
    let huge = "9".repeat(16_384);
    for token in ["-0", "0.00", "1E+000", "1e-99999", huge.as_str()] {
        let source = format!(r#"{{{OPAQUE},"token":{token},"ns":{{"s":"{SYSML_URI}"}}}}"#);
        let first = prepare(&source, "first", &[]).unwrap();
        opaque(&first.json);
        assert_eq!(first.json["token"].to_string(), token);
        let wire = first.to_json().unwrap();
        assert!(wire.contains(&format!(r#""token":{token}"#)));
        let second = prepare(&wire, "second", &[]).unwrap();
        opaque(&second.json);
        assert_eq!(second.json["token"].to_string(), token);
        assert_eq!(second.to_json().unwrap(), wire);
        assert_eq!(first.participant_version, "first");
        assert_eq!(second.participant_version, "second");
    }
}
#[test]
fn literal_first_namespace_and_duplicate_order_cannot_be_coerced() {
    for first in [
        r#"{"$serde_json::private::Number":"123"}"#,
        r#"{"$serde_json::private::RawValue":"\"http://www.eclipse.org/syson/sysml\""}"#,
    ] {
        let source = format!(r#"{{"ns":{{"z":{first},"a":"{SYSML_URI}"}}}}"#);
        assert_eq!(prepare(&source, "v", &[]), Err(Error::NamespaceShape));
    }
    for (first, last, matched) in [("foreign", SYSML_URI, true), (SYSML_URI, "foreign", false)] {
        let source =
            format!(r#"{{{OPAQUE},"ns":{{"z":"{first}","a":"foreign","\u007a":"{last}"}}}}"#);
        let prepared = prepare(&source, "v", &[]).unwrap();
        assert_eq!(prepared.matched_resource, matched);
        opaque(&prepared.json);
        let wire = prepared.to_json().unwrap();
        assert!(wire.contains(&format!(r#""ns":{{"z":"{last}","a":"foreign"}}"#)));
        assert_eq!(
            prepare(&wire, "reentry", &[]).unwrap().matched_resource,
            matched
        );
    }
    let duplicate = format!(r#"{{"ns":{{"s":"{SYSML_URI}"}},"ns":{{"s":"foreign"}}}}"#);
    assert!(!prepare(&duplicate, "v", &[]).unwrap().matched_resource);
}
#[test]
fn physical_overwritten_values_and_exact_byte_and_depth_edges_remain_bounded() {
    let exact = format!("{{{}}}", vec![r#""x":null"#; MAX_VALUES - 1].join(","));
    assert_eq!(
        prepare(&exact, "v", &[]).unwrap().to_json().unwrap(),
        r#"{"x":null}"#
    );
    let over = format!("{{{}}}", vec![r#""x":null"#; MAX_VALUES].join(","));
    assert_eq!(prepare(&over, "v", &[]), Err(Error::Limit));
    let mut bytes = format!("{{}}{}", " ".repeat(MAX_BYTES - 2));
    assert!(prepare(&bytes, "v", &[]).is_ok());
    bytes.push(' ');
    assert_eq!(prepare(&bytes, "v", &[]), Err(Error::Limit));
    for extra in [0, 1] {
        let count = MAX_DEPTH - 1 + extra;
        let source = format!(r#"{{"v":{}0{}}}"#, "[".repeat(count), "]".repeat(count));
        assert_eq!(prepare(&source, "v", &[]).is_ok(), extra == 0);
    }
}
#[test]
fn late_pass_failure_leaves_input_and_prior_preparation_unchanged() {
    let source = format!(
        r#"{{{OPAQUE},"ns":{{"s":"{SYSML_URI}"}},"content":[{{"eClass":"sysml:ViewUsage","data":{{"exposedElement":"old","keep":"yes"}}}}]}}"#
    );
    let before = source.clone();
    let remove = Removal {
        class: "ViewUsage",
        features: &["exposedElement"],
    };
    let previous = prepare(&source, "selected", &[remove]).unwrap();
    assert!(
        previous.json["content"][0]["data"]
            .get("exposedElement")
            .is_none()
    );
    assert_eq!(previous.json["content"][0]["data"]["keep"], "yes");
    opaque(&previous.json);
    let published = previous.to_json().unwrap();
    let bad = Removal {
        class: "NotAClass",
        features: &["keep"],
    };
    assert_eq!(
        prepare(&source, "late", &[remove, bad]),
        Err(Error::UnknownClass("NotAClass".into()))
    );
    assert_eq!(source, before);
    assert_eq!(previous.to_json().unwrap(), published);
}
#[derive(Default)]
struct NoLoad(usize);
impl Resolver for NoLoad {
    fn resolve(&mut self, _: &str) -> Result<Option<ResolvedElement>, String> {
        self.0 += 1;
        Ok(None)
    }
}
#[test]
fn actual_document_and_b01_handoff_preserve_literal_payloads_ids_and_reopen() {
    let fixture =
        include_str!("../../syson-migration-pipeline/tests/fixtures/native-document.json");
    let source = format!(
        "{{{OPAQUE},{}",
        fixture.trim_start().strip_prefix('{').unwrap()
    );
    let original = source.clone();
    let carrier = prepare(&source, "shared", &[]).unwrap();
    let document = Document::from_json(&carrier.to_json().unwrap()).unwrap();
    opaque(document.value());
    let old_id = "b352ded6-38ba-4e7e-ae55-9299cb0cc6d0";
    let new_id = "76028d3d-69a4-5e12-9002-ce403e0244bd";
    let mut content = Map::new();
    content.insert("targetObjectId".into(), Value::String(old_id.into()));
    content.insert("opaque".into(), carrier.json.clone());
    // Direct insertion avoids fixture construction serializing and normalizing numbers.
    opaque(content.get("opaque").unwrap());
    let input = Project {
        documents: vec![SemanticDocument {
            document,
            references_version: None,
        }],
        representations: vec![Representation {
            kind: DIAGRAM_KIND.into(),
            content: Value::Object(content),
            diagram_version: None,
        }],
    };
    let before = input.clone();
    let ids: HashMap<String, String> = syson_persistence::parse_id_map(include_str!(
        "../../syson-migration-pipeline/tests/fixtures/native-id-map.txt"
    ));
    assert_eq!(ids.get(old_id).map(String::as_str), Some(new_id));
    let output = library::prepare(&input, &ids, &mut NoLoad::default()).unwrap();
    let saved = &output.documents[0].document;
    opaque(saved.value());
    opaque(&output.representations[0].content["opaque"]);
    assert_eq!(output.representations[0].content["targetObjectId"], new_id);
    let typing = saved
        .element("b2850b04-3f1f-4799-8b44-f0706f22628b")
        .unwrap();
    assert_eq!(
        typing["data"]["type"],
        "sysml:DataType kermllibrary:///b2c6dd37-2084-3ce4-9ce2-580fdf30629c#76028d3d-69a4-5e12-9002-ce403e0244bd"
    );
    let reopened = Document::from_json(&saved.to_json().unwrap()).unwrap();
    opaque(reopened.value());
    assert_eq!(
        reopened.value()["content"][0]["id"],
        "3f891ac9-b702-4e7f-ba4a-99fc1d18e3fc"
    );
    assert_eq!(
        reopened.value()["content"][0]["data"]["elementId"],
        "c945c5ed-5a29-429a-8be7-d98286295a8c"
    );
    assert_eq!(
        output.documents[0].references_version.as_deref(),
        Some(REFERENCES_VERSION)
    );
    assert_eq!(
        output.representations[0].diagram_version.as_deref(),
        Some(DIAGRAM_VERSION)
    );
    let mut replay = NoLoad::default();
    assert_eq!(
        library::prepare(&output, &ids, &mut replay).unwrap(),
        output
    );
    assert_eq!(replay.0, 0);
    assert_eq!(input, before);
    assert_eq!(source, original);
}
