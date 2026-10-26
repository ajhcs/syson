// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::collections::HashMap;
use syson_migration_pipeline::{Project, Resolver, SemanticDocument};
use syson_migrations::b25a::{VERSION, prepare};
use syson_model::Document;
use syson_persistence::ResolvedElement;

const FIXTURES: &str =
    include_str!("../../../../porting/rust-parity-evidence/B25a/native-fixtures.json");
const PREDICTIONS: &str =
    include_str!("../../../../porting/rust-parity-evidence/B25a/source-predictions.json");

#[test]
fn source_contract_packet_preserves_native_quirks_without_claiming_native_execution() {
    let fixtures: Value = serde_json::from_str(FIXTURES).unwrap();
    let predictions: Value = serde_json::from_str(PREDICTIONS).unwrap();
    let predictions = predictions["predictions"].as_array().unwrap();
    assert_eq!(
        fixtures["cases"].as_array().unwrap().len(),
        predictions.len()
    );
    for case in fixtures["cases"].as_array().unwrap() {
        let input = case["input"].as_str().unwrap();
        let before = input.to_owned();
        let expected =
            &predictions.iter().find(|p| p["id"] == case["id"]).unwrap()["source_prediction"];
        let prepared = prepare(input).unwrap();
        assert_eq!(prepared.json, expected["json"], "case {}", case["id"]);
        assert_eq!(prepared.participant_version, VERSION);
        assert_eq!(
            prepared.matched_resource,
            expected["matched_resource"].as_bool().unwrap()
        );
        assert_eq!(input, before);
        // Replay the prepared wire output with retained namespace ordering;
        // the inspection Value is not the native resource serialization.
        if prepared.matched_resource {
            assert_eq!(
                prepare(&prepared.to_json().unwrap()).unwrap().json,
                prepared.json
            );
        }
    }
    assert_eq!(VERSION, "2025.4.0-202503140000");
}

#[test]
fn invalid_input_and_budget_failures_do_not_replace_an_existing_preparation() {
    let input = r#"{"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{"eClass":"sysml:Annotation","data":{"annotatingElement":"drop","keep":1}}]}"#;
    let published = prepare(input).unwrap();
    let before = published.json.clone();
    for invalid in ["{", "null", "[]", r#"{"ns":{"sysml":null}}"#] {
        assert!(prepare(invalid).is_err(), "{invalid}");
        assert_eq!(published.json, before);
    }
    let oversized = json!({"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"padding":"x".repeat(2*1024*1024)}).to_string();
    assert!(prepare(&oversized).is_err());
    let mut deep = json!(0);
    for _ in 0..100 {
        deep = json!([deep]);
    }
    assert!(
        prepare(
            &json!({"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"deep":deep}).to_string()
        )
        .is_err()
    );
    assert_eq!(published.json, before);
    assert!(published.matched_resource);
    assert_eq!(published.participant_version, VERSION);
}

struct NoResourceLoads(usize);
impl Resolver for NoResourceLoads {
    fn resolve(&mut self, _: &str) -> Result<Option<ResolvedElement>, String> {
        self.0 += 1;
        Err("migration must not request an external resource".into())
    }
}

#[test]
fn participant_output_enters_actual_document_admission_and_b01_preparation() {
    let fixtures: Value = serde_json::from_str(FIXTURES).unwrap();
    let input = fixtures["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "document-admission-b01-chain")
        .unwrap()["input"]
        .as_str()
        .unwrap()
        .to_owned();
    let before = input.clone();
    let migrated = prepare(&input).unwrap();
    let document = Document::from_json(&migrated.to_json().unwrap()).unwrap();
    assert_eq!(document.elements().len(), 3);
    assert!(
        document.element("annotation-semantic").unwrap()["data"]
            .get("annotatingElement")
            .is_none()
    );
    let comment = document.element("comment-semantic").unwrap();
    assert!(comment["data"].get("annotation").is_none());
    assert_eq!(comment["data"]["body"], "Preserve this note 🚗");
    let project = Project {
        documents: vec![SemanticDocument {
            document: document.clone(),
            references_version: None,
        }],
        representations: vec![],
    };
    let original = project.clone();
    let mut resolver = NoResourceLoads(0);
    let prepared =
        syson_migration_pipeline::prepare(&project, &HashMap::new(), &mut resolver).unwrap();
    assert_eq!(prepared.documents[0].document, document);
    assert_eq!(project, original);
    assert_eq!(resolver.0, 0);
    assert_eq!(input, before);
    assert_eq!(
        prepared.documents[0]
            .document
            .element("comment-semantic")
            .unwrap()["id"],
        "comment-node"
    );
    assert_eq!(
        prepared.documents[0]
            .document
            .element("annotation-semantic")
            .unwrap()["id"],
        "annotation-node"
    );
}
