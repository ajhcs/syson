// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::collections::HashMap;
use syson_migration_pipeline::{Project, Resolver, SemanticDocument};
use syson_migrations::b25c;
use syson_model::Document;
use syson_persistence::{REFERENCES_VERSION, ResolvedElement};

const NS: &str = "http://www.eclipse.org/syson/sysml";
fn view(data: Value) -> Value {
    json!({"id":"view-node","eClass":"sysml:ViewUsage","data":data})
}
fn resource(content: Value) -> Value {
    json!({"json":{"version":"1.0","encoding":"utf-8"},
        "ns":{"sysml":NS},"content":content})
}
fn run(value: &Value) -> Value {
    b25c::prepare(&value.to_string()).unwrap().json
}

#[test]
fn exact_feature_is_removed_while_spelling_neighbors_and_ids_survive() {
    let input = resource(json!([view(json!({
        "elementId":"view-semantic","declaredName":"View",
        "exposedElement":["sysml:Element memory:/a#old"],
        "exposeElement":"the native comment's misspelling is not the feature",
        "exposedElements":"plural is unrelated", "source":"source stays"
    }))]));
    let before = input.clone();
    let output = b25c::prepare(&input.to_string()).unwrap();
    assert_eq!(output.participant_version, b25c::VERSION);
    assert_eq!(output.participant_version, "2025.6.0-202506110000");
    assert!(output.matched_resource);
    let mut expected = input.clone();
    expected["content"][0]["data"]
        .as_object_mut()
        .unwrap()
        .remove("exposedElement");
    assert_eq!(output.json, expected);
    assert_eq!(input, before);
}

#[test]
fn child_object_collection_includes_vendor_objects_but_not_parent_fields() {
    let input = resource(json!([{
        "eClass":"sysml:ViewUsage", "exposedElement":"parent survives",
        "data":{"elementId":"view-semantic","exposedElement":7},
        "vendor":{"exposedElement":null,"label":"keep"},
        "array":[{"exposedElement":"untyped array row survives"}],
        "primitive":"not a target"
    }]));
    let output = run(&input);
    assert_eq!(output["content"][0]["exposedElement"], "parent survives");
    assert!(output["content"][0]["data"].get("exposedElement").is_none());
    assert!(
        output["content"][0]["vendor"]
            .get("exposedElement")
            .is_none()
    );
    assert_eq!(output["content"][0]["vendor"]["label"], "keep");
    assert_eq!(
        output["content"][0]["array"][0]["exposedElement"],
        "untyped array row survives"
    );
}

#[test]
fn traversal_reaches_arbitrary_nested_arrays_and_objects() {
    let target = view(json!({"elementId":"nested-semantic","exposedElement":["x"]}));
    let input = resource(json!([]));
    let mut input = input;
    input["extension"] = json!({"rows":[[[{"wrapper":target}]]],"exposedElement":"not typed"});
    let output = run(&input);
    let data = &output["extension"]["rows"][0][0][0]["wrapper"]["data"];
    assert!(data.get("exposedElement").is_none());
    assert_eq!(data["elementId"], "nested-semantic");
    assert_eq!(output["extension"]["exposedElement"], "not typed");
}

#[test]
fn sibling_unknown_enum_and_foreign_classes_preserve_the_same_property() {
    // Pinned Ecore has no declared ViewUsage subclasses. PartUsage is its
    // superclass, not a positive subtype, and ViewDefinition is a separate class.
    for class in [
        "sysml:PartUsage",
        "sysml:ViewDefinition",
        "sysml:NoSuchClass",
        "sysml:VisibilityKind",
        "other:ViewUsage",
    ] {
        let input = resource(json!([{"eClass":class,"data":{"exposedElement":["keep"]}}]));
        assert_eq!(run(&input), input, "class {class}");
    }
    for eclass in [
        json!(false),
        json!(42),
        Value::Null,
        json!({"name":"sysml:ViewUsage"}),
    ] {
        let input = resource(json!([{"eClass":eclass,"data":{"exposedElement":["keep"]}}]));
        assert_eq!(run(&input), input);
    }
}

#[test]
fn first_namespace_entry_decides_in_original_json_order() {
    let suffix = r#", "content":[{"eClass":"sysml:ViewUsage","data":{"exposedElement":"old"}}]}"#;
    let first = format!(r#"{{"ns":{{"z":"{NS}","a":"urn:foreign"}}{suffix}"#);
    let second = format!(r#"{{"ns":{{"z":"urn:foreign","a":"{NS}"}}{suffix}"#);
    let output = b25c::prepare(&first).unwrap();
    assert!(output.matched_resource);
    assert!(
        output.json["content"][0]["data"]
            .get("exposedElement")
            .is_none()
    );
    let repeated = b25c::prepare(&output.to_json().unwrap()).unwrap();
    assert!(repeated.matched_resource);
    assert_eq!(repeated.json, output.json);
    let output = b25c::prepare(&second).unwrap();
    assert!(!output.matched_resource);
    assert_eq!(output.json, serde_json::from_str::<Value>(&second).unwrap());
    let repeated = b25c::prepare(&output.to_json().unwrap()).unwrap();
    assert!(!repeated.matched_resource);
    assert_eq!(repeated.json, output.json);
}

#[test]
fn absent_empty_and_nonobject_namespace_are_noops() {
    for namespace in [Value::Null, json!({}), json!([]), json!("not an object")] {
        let input = json!({"ns":namespace,"content":[view(json!({"exposedElement":"stay"}))]});
        let output = b25c::prepare(&input.to_string()).unwrap();
        assert!(!output.matched_resource);
        assert_eq!(output.json, input);
    }
    let input = json!({"content":[view(json!({"exposedElement":"stay"}))]});
    assert_eq!(run(&input), input);
}

#[test]
fn null_array_and_primitive_feature_values_are_removed_identically() {
    for value in [
        Value::Null,
        json!(false),
        json!(9),
        json!([]),
        json!({"ref":"old"}),
    ] {
        let input = resource(json!([view(
            json!({"exposedElement":value,"declaredName":"keep"})
        )]));
        let mut expected = input.clone();
        expected["content"][0]["data"]
            .as_object_mut()
            .unwrap()
            .remove("exposedElement");
        assert_eq!(run(&input), expected);
    }
}

#[test]
fn repeat_is_idempotent_and_failure_cannot_replace_prior_publication() {
    let input = resource(json!([view(json!({"exposedElement":"old"}))]));
    let prepared = b25c::prepare(&input.to_string()).unwrap();
    let first = prepared.json.clone();
    assert_eq!(
        b25c::prepare(&prepared.to_json().unwrap()).unwrap().json,
        first
    );
    let mut published = first.clone();
    for invalid in ["{", "[]", r#"{"ns":{"sysml":null}}"#] {
        if let Ok(prepared) = b25c::prepare(invalid) {
            published = prepared.json;
        }
        assert_eq!(published, first);
    }
    let oversized = resource(json!([view(
        json!({"exposedElement":"old","blob":"x".repeat(1_048_576)})
    )]));
    assert!(b25c::prepare(&oversized.to_string()).is_err());
    let mut deep = view(json!({"exposedElement":"old"}));
    for _ in 0..70 {
        deep = json!([deep]);
    }
    assert!(b25c::prepare(&resource(deep).to_string()).is_err());
    assert_eq!(published, first);
}

#[test]
fn actual_document_and_b01_preparation_keep_node_and_semantic_identity() {
    struct NoLoad;
    impl Resolver for NoLoad {
        fn resolve(&mut self, _: &str) -> Result<Option<ResolvedElement>, String> {
            Err("no resolver call is expected for this reference-free document".into())
        }
    }
    let input = resource(json!([view(
        json!({"elementId":"view-semantic","declaredName":"View","exposedElement":["old"]})
    )]));
    let prepared = b25c::prepare(&input.to_string()).unwrap();
    let document = Document::from_json(&prepared.to_json().unwrap()).unwrap();
    let project = Project {
        documents: vec![SemanticDocument {
            document,
            references_version: None,
        }],
        representations: vec![],
    };
    let before = project.clone();
    let output = syson_migration_pipeline::prepare(&project, &HashMap::new(), &mut NoLoad).unwrap();
    assert_eq!(project, before);
    assert_eq!(output.documents[0].document.value(), &prepared.json);
    assert_eq!(
        output.documents[0].document.value()["content"][0]["id"],
        "view-node"
    );
    assert_eq!(
        output.documents[0].document.value()["content"][0]["data"]["elementId"],
        "view-semantic"
    );
    assert_eq!(
        output.documents[0].references_version.as_deref(),
        Some(REFERENCES_VERSION)
    );
    assert_eq!(
        syson_migration_pipeline::prepare(&output, &HashMap::new(), &mut NoLoad).unwrap(),
        output
    );
    // B25c's participant_version remains a separate preparation result; B01
    // does not persist it or claim native migration-history compatibility.
    assert_eq!(prepared.participant_version, b25c::VERSION);
}
