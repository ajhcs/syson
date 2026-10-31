// SPDX-License-Identifier: EPL-2.0
use serde_json::Value;
use syson_migrations::b25d::{self, Error, KIND, VERSION};

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
