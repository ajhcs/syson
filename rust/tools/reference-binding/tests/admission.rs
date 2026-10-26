// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::collections::BTreeMap;
use syson_reference_binding::{TARGET, UnverifiedFacts, admit};

fn strings(value: &Value) -> BTreeMap<String, String> {
    value
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap().into()))
        .collect()
}
fn retained_fixture() -> (Value, UnverifiedFacts) {
    let data: Value =
        serde_json::from_str(include_str!("fixtures/unverified-retained-facts.json")).unwrap();
    let facts = &data["facts"];
    (
        data["request"].clone(),
        UnverifiedFacts {
            files: strings(&facts["files"]),
            documents: facts["documents"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            git_sources: strings(&facts["git_sources"]),
            git_revision: facts["git_revision"].as_str().unwrap().into(),
            delta_from: facts["delta_from"].as_str().unwrap().into(),
            delta_to: facts["delta_to"].as_str().unwrap().into(),
            delta: facts["delta"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().into())
                .collect(),
        },
    )
}
// Deliberately synthetic facts model a new coherent final review. No real file,
// receipt authenticity or fresh inspection is asserted by these positive tests.
fn fixture() -> (Value, UnverifiedFacts) {
    let (mut request, mut facts) = retained_fixture();
    let historical_path = request["files"]["review"]["path"]
        .as_str()
        .unwrap()
        .to_owned();
    facts.files.remove(&historical_path);
    let synthetic_path = "/synthetic/final-scoped-review.json";
    let synthetic_hash = "a".repeat(64);
    request["files"]["review"] = json!({"path":synthetic_path,"sha256":synthetic_hash});
    facts.files.insert(synthetic_path.into(), synthetic_hash);
    facts.documents.get_mut("review").unwrap()["harness_sha256"] =
        request["files"]["harness"]["sha256"].clone();
    (request, facts)
}
#[test]
fn historical_retained_review_contradiction_refuses_without_rewriting_it() {
    let (request, facts) = retained_fixture();
    assert_ne!(
        facts.documents["review"]["harness_sha256"],
        request["files"]["harness"]["sha256"]
    );
    let error = admit(&request, &facts).unwrap_err();
    assert!(error.contains("stale review harness"));
}
#[test]
fn synthetic_coherent_facts_admit_only_policy_consistency_and_keep_parent_open() {
    let (request, facts) = fixture();
    let result = admit(&request, &facts).unwrap();
    assert_eq!(
        result["admission"],
        "POLICY_CONSISTENT_WITH_UNVERIFIED_FACTS"
    );
    assert_eq!(result["file_inspection_performed"], false);
    assert_eq!(result["C01"], "OPEN");
    assert_eq!(result["whole_artifact_equivalence"], false);
    assert_eq!(result["label_matches_target"], false);
    assert_eq!(
        result["source_label_delta_paths"].as_array().unwrap().len(),
        125
    );
    assert_eq!(result["selected_source_classes"], 4);
    assert_eq!(result["identified_dependency_jars"], 9);
    assert!(
        !result["remaining_requirements"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
#[test]
fn stale_invalid_pin_and_claim_escalation_are_refused_without_mutation() {
    let (original, facts) = fixture();
    for pin in [
        "0".repeat(40),
        "6ba2;false".into(),
        "g".repeat(40),
        TARGET.to_uppercase(),
    ] {
        let mut request = original.clone();
        request["target_revision"] = json!(pin);
        assert!(admit(&request, &facts).is_err());
    }
    let mut request = original.clone();
    request["claim"] = json!("whole-artifact-equivalence");
    assert!(admit(&request, &facts).is_err());
    let mut request = original.clone();
    request["C01_closed"] = json!(true);
    assert!(admit(&request, &facts).is_err());
    assert!(admit(&original, &facts).is_ok());
}
#[test]
fn changed_or_missing_artifact_receipt_tool_and_dependency_hashes_refuse() {
    let (request, facts) = fixture();
    let review = &facts.documents["review"];
    let paths = [
        request["files"]["artifact"]["path"].as_str().unwrap(),
        request["files"]["execution"]["path"].as_str().unwrap(),
        review["tools"].as_object().unwrap().keys().next().unwrap(),
        review["dependency_jar_sha256"]
            .as_object()
            .unwrap()
            .keys()
            .next()
            .unwrap(),
    ];
    for path in paths {
        let mut changed = facts.clone();
        changed.files.insert(path.into(), "0".repeat(64));
        assert!(admit(&request, &changed).is_err(), "{path}");
        changed.files.remove(path);
        assert!(admit(&request, &changed).is_err(), "{path}");
    }
}
#[test]
fn partial_duplicate_and_unadmitted_source_coverage_refuse() {
    let (request, facts) = fixture();
    let mut partial = request.clone();
    partial["sources"].as_array_mut().unwrap().pop();
    assert!(admit(&partial, &facts).is_err());
    let mut duplicate = request.clone();
    duplicate["sources"][1] = duplicate["sources"][0].clone();
    assert!(admit(&duplicate, &facts).is_err());
    let mut escape = request.clone();
    escape["sources"][0]["repo_path"] = json!("../outside.java");
    assert!(admit(&escape, &facts).is_err());
    let mut changed = facts.clone();
    changed.git_sources.clear();
    assert!(admit(&request, &changed).is_err());
    changed = facts.clone();
    let map = changed.documents.get_mut("review").unwrap()["dependency_jar_sha256"]
        .as_object_mut()
        .unwrap();
    let first = map.keys().next().unwrap().clone();
    map.remove(&first);
    assert!(admit(&request, &changed).is_err());
}
#[test]
fn source_label_configuration_and_delta_provenance_mismatches_refuse() {
    let (request, facts) = fixture();
    for key in ["image_config", "image_manifest", "reference"] {
        let mut changed = facts.clone();
        changed.documents.insert(key.into(), json!({}));
        assert!(admit(&request, &changed).is_err(), "{key}");
    }
    let mut changed = facts.clone();
    changed.delta.pop();
    assert!(admit(&request, &changed).is_err());
    changed = facts.clone();
    changed.delta.push(changed.delta[0].clone());
    assert!(admit(&request, &changed).is_err());
    changed = facts.clone();
    changed.git_revision = "0".repeat(40);
    assert!(admit(&request, &changed).is_err());
    changed = facts.clone();
    changed.delta_to = TARGET.into();
    assert!(admit(&request, &changed).is_err());
}
#[test]
fn missing_or_mismatched_selected_effective_build_and_runtime_config_refuse() {
    let (request, facts) = fixture();
    let mut changed = facts.clone();
    changed.documents.get_mut("review").unwrap()["annotation_processing"] = json!("enabled");
    assert!(admit(&request, &changed).is_err());
    for args in [json!([]), json!(["javac"])] {
        changed = facts.clone();
        changed.documents.get_mut("execution").unwrap()["commands"][1]["argv"] = args;
        assert!(admit(&request, &changed).is_err());
    }
    changed = facts.clone();
    changed.documents.get_mut("execution").unwrap()["commands"][1]["argv"][3] = json!("-proc:full");
    assert!(admit(&request, &changed).is_err());
    changed = facts.clone();
    changed.documents.get_mut("execution").unwrap()["commands"][3]["argv"]
        .as_array_mut()
        .unwrap()
        .push(json!("--extra"));
    assert!(admit(&request, &changed).is_err());
    changed = facts.clone();
    changed.documents.get_mut("execution").unwrap()["source_unchanged"] = json!(false);
    assert!(admit(&request, &changed).is_err());
}
#[test]
fn missing_class_and_fixture_inputs_or_failed_compilation_refuse() {
    let (request, facts) = fixture();
    let mut changed = facts.clone();
    let map = changed.documents.get_mut("execution").unwrap()["class_sha256"]
        .as_object_mut()
        .unwrap();
    let first = map.keys().next().unwrap().clone();
    map.remove(&first);
    assert!(admit(&request, &changed).is_err());
    changed = facts.clone();
    let input = changed.documents["execution"]["commands"][3]["argv"][8]
        .as_str()
        .unwrap()
        .to_owned();
    changed.files.remove(&input);
    assert!(admit(&request, &changed).is_err());
    changed = facts.clone();
    changed.documents.get_mut("execution").unwrap()["commands"][1]["exit_code"] = json!(1);
    assert!(admit(&request, &changed).is_err());
}
#[test]
fn malformed_and_missing_required_structure_returns_errors() {
    let (request, facts) = fixture();
    for bad in [
        Value::Null,
        json!([]),
        json!({}),
        json!({"claim":"selected-source-runtime"}),
    ] {
        assert!(admit(&bad, &facts).is_err());
    }
    for role in ["harness", "artifact", "review", "execution"] {
        let mut bad = request.clone();
        bad["files"].as_object_mut().unwrap().remove(role);
        assert!(admit(&bad, &facts).is_err());
    }
}
