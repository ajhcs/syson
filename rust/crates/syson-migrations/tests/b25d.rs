// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::collections::HashMap;
use syson_migration_pipeline::{Project, Representation, Resolver};
use syson_migrations::{
    MAX_BYTES, MAX_VALUES,
    b25d::{self, Error, KIND, MAPPINGS, VERSION},
};
use syson_persistence::{DIAGRAM_VERSION, ResolvedElement};

const FIXTURES: &str =
    include_str!("../../../../porting/rust-parity-evidence/B25d/native-fixtures.json");
const PREDICTIONS: &str =
    include_str!("../../../../porting/rust-parity-evidence/B25d/source-predictions.json");

#[test]
fn explicit_callback_and_inherited_traversal_source_contract_corpus() {
    let fixtures: Value = serde_json::from_str(FIXTURES).unwrap();
    let predictions: Value = serde_json::from_str(PREDICTIONS).unwrap();
    for case in fixtures["cases"].as_array().unwrap() {
        let input = case["input"].as_str().unwrap();
        let before = input.to_owned();
        let expected = &predictions["predictions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == case["id"])
            .unwrap()["source_prediction"];
        if case["mode"] == "callback" {
            let mut root: Value = serde_json::from_str(input).unwrap();
            b25d::replace_json_node(
                root.as_object_mut().unwrap(),
                case["current_attribute"].as_str().unwrap(),
                &case["current_value"],
            );
            assert_eq!(root, expected["json"], "{}", case["id"]);
        } else {
            let result = b25d::prepare(KIND, input, None);
            if expected["status"] == "REFUSED" {
                assert!(matches!(result, Err(Error::NestedArray)), "{}", case["id"]);
                assert!(expected.get("native_partial_json").is_some());
            } else {
                let output = result.unwrap();
                assert_eq!(output.json, expected["json"], "{}", case["id"]);
                assert_eq!(output.participant_version.as_deref(), Some(VERSION));
                assert!(output.matched_kind);
                assert_eq!(
                    b25d::prepare(KIND, &output.to_json().unwrap(), Some(VERSION))
                        .unwrap()
                        .json,
                    output.json
                );
            }
        }
        assert_eq!(input, before);
        assert_eq!(expected["version"], VERSION);
        assert_eq!(expected["kind"], KIND);
    }
    assert_eq!(fixtures["cases"].as_array().unwrap().len(), 19);
}

#[test]
fn low_level_callback_uses_explicit_value_and_has_no_read_only_or_root_value_fallback() {
    let (old, new) = MAPPINGS[0];
    let mut root = json!({"readOnly":true,"descriptionId":MAPPINGS[1].0});
    assert!(b25d::replace_json_node(
        root.as_object_mut().unwrap(),
        "descriptionId",
        &json!(old)
    ));
    assert_eq!(root["descriptionId"], new);
    assert_eq!(root["readOnly"], true);
    // The native callback writes the same new value even if it is already there.
    assert!(!b25d::replace_json_node(
        root.as_object_mut().unwrap(),
        "descriptionId",
        &json!(old)
    ));
    root["descriptionId"] = json!(old);
    assert!(!b25d::replace_json_node(
        root.as_object_mut().unwrap(),
        "descriptionId",
        &json!(new)
    ));
    assert_eq!(root["descriptionId"], old);
}

#[test]
fn selected_kind_watermark_replay_and_unknown_policy_are_separate_from_framework_history() {
    let input = json!({"descriptionId":MAPPINGS[0].0}).to_string();
    let first = b25d::prepare(KIND, &input, None).unwrap();
    assert!(first.changed);
    let replay = b25d::prepare(KIND, &first.to_json().unwrap(), Some(VERSION)).unwrap();
    assert!(!replay.changed);
    assert_eq!(replay.json, first.json);
    // A selected watermark skips this callback even if the caller supplies an
    // unmigrated value. It is not proof of native global history or validity.
    assert_eq!(
        b25d::prepare(KIND, &input, Some(VERSION)).unwrap().json["descriptionId"],
        MAPPINGS[0].0
    );
    assert!(
        matches!(b25d::prepare(KIND, &input, Some("future")), Err(Error::UnsupportedVersion(v)) if v == "future")
    );
    let foreign = b25d::prepare(
        "siriusComponents://representation?type=Table",
        &input,
        Some("future"),
    )
    .unwrap();
    assert!(!foreign.matched_kind && !foreign.changed);
    assert_eq!(foreign.participant_version.as_deref(), Some("future"));
    assert_eq!(foreign.json, serde_json::from_str::<Value>(&input).unwrap());
    assert_eq!(
        b25d::prepare("foreign", "[[]]", None).unwrap().json,
        json!([[]])
    );
}

#[test]
fn current_unknown_and_absent_description_ids_are_noop_without_inserting_null_fields() {
    let input = json!({"nodes":[{"descriptionId":MAPPINGS[0].1},{"descriptionId":"unknown"},{"label":"no ID"}]}).to_string();
    let output = b25d::prepare(KIND, &input, None).unwrap();
    assert!(!output.changed);
    assert_eq!(output.json, serde_json::from_str::<Value>(&input).unwrap());
    assert_eq!(output.participant_version.as_deref(), Some(VERSION));
    assert!(output.json["nodes"][2].get("descriptionId").is_none());
}

#[test]
fn failure_after_earlier_private_changes_publishes_nothing_and_preserves_existing_output() {
    let input = json!({"descriptionId":MAPPINGS[0].0}).to_string();
    let published = b25d::prepare(KIND, &input, None).unwrap();
    let before = published.to_json().unwrap();
    let late = json!({"a":{"descriptionId":MAPPINGS[0].0},"z":[[]]}).to_string();
    let original = late.clone();
    assert_eq!(b25d::prepare(KIND, &late, None), Err(Error::NestedArray));
    assert_eq!(late, original);
    assert_eq!(published.to_json().unwrap(), before);
    for malformed in ["{", "[]", "null", "17"] {
        assert!(b25d::prepare(KIND, malformed, None).is_err());
        assert_eq!(published.to_json().unwrap(), before);
    }
}

#[test]
fn preparation_and_serialization_enforce_candidate_byte_depth_and_value_budgets() {
    let oversized = json!({"padding":"x".repeat(MAX_BYTES)}).to_string();
    assert!(matches!(
        b25d::prepare(KIND, &oversized, None),
        Err(Error::Limit)
    ));
    let many = json!({"values":vec![0;MAX_VALUES]}).to_string();
    assert!(matches!(
        b25d::prepare(KIND, &many, None),
        Err(Error::Limit)
    ));
    let mut deep = json!(0);
    for _ in 0..100 {
        deep = json!({"child":deep});
    }
    assert!(b25d::prepare(KIND, &deep.to_string(), None).is_err());
    let mut prepared = b25d::prepare(KIND, "{}", None).unwrap();
    prepared.json["padding"] = json!("x".repeat(MAX_BYTES));
    assert_eq!(prepared.to_json(), Err(Error::Limit));
    assert!(matches!(
        b25d::prepare("foreign", "{}", Some(&"v".repeat(4097))),
        Err(Error::Limit)
    ));
}

struct NoLoad(usize);
impl Resolver for NoLoad {
    fn resolve(&mut self, _: &str) -> Result<Option<ResolvedElement>, String> {
        self.0 += 1;
        Ok(None)
    }
}
#[test]
fn actual_b01_representation_handoff_migrates_target_ids_without_reusing_watermarks() {
    let input =
        json!({"nodes":[{"descriptionId":MAPPINGS[0].0,"targetObjectId":"old"}]}).to_string();
    let control = b25d::prepare(KIND, &input, None).unwrap();
    let project = Project {
        documents: vec![],
        representations: vec![Representation {
            kind: KIND.into(),
            content: serde_json::from_str(&control.to_json().unwrap()).unwrap(),
            diagram_version: None,
        }],
    };
    let before = project.clone();
    let mut resolver = NoLoad(0);
    let ids = HashMap::from([("old".into(), "new".into())]);
    let output = syson_migration_pipeline::prepare(&project, &ids, &mut resolver).unwrap();
    assert_eq!(project, before);
    assert_eq!(resolver.0, 1);
    assert_eq!(
        output.representations[0].content["nodes"][0]["descriptionId"],
        MAPPINGS[0].1
    );
    assert_eq!(
        output.representations[0].content["nodes"][0]["targetObjectId"],
        "new"
    );
    assert_eq!(
        output.representations[0].diagram_version.as_deref(),
        Some(DIAGRAM_VERSION)
    );
    assert_eq!(control.participant_version.as_deref(), Some(VERSION));
    assert_ne!(VERSION, DIAGRAM_VERSION);
    assert_eq!(
        b25d::prepare(
            KIND,
            &output.representations[0].content.to_string(),
            Some(VERSION)
        )
        .unwrap()
        .json,
        output.representations[0].content
    );
}
