// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::hint::black_box;
use std::time::Instant;
use syson_migrations::b25d::{self, Error, KIND, VERSION};

const INPUTS: &str =
    include_str!("../../../../porting/rust-parity-evidence/B25d/native-fixtures.json");
const EXPECTED: &str =
    include_str!("../../../../porting/rust-parity-evidence/B25d/source-predictions.json");

fn main() {
    assert_eq!(
        std::env::args().collect::<Vec<_>>().len(),
        2,
        "one fixed mode required"
    );
    assert_eq!(std::env::args().nth(1).as_deref(), Some("full-corpus"));
    assert!(INPUTS.len() <= 1024 * 1024 && EXPECTED.len() <= 1024 * 1024);
    let fixtures: Value = serde_json::from_str(INPUTS).expect("fixed input JSON");
    let predictions: Value = serde_json::from_str(EXPECTED).expect("fixed prediction JSON");
    assert_eq!(fixtures["status"], "INPUT_ONLY_NATIVE_ORACLE_PACKET");
    assert_eq!(
        predictions["status"],
        "SOURCE_PREDICTIONS_NOT_OBSERVED_NATIVE_RESULTS"
    );
    let cases = fixtures["cases"].as_array().expect("fixed cases");
    let expected = predictions["predictions"]
        .as_array()
        .expect("fixed predictions");
    assert_eq!(cases.len(), 19);
    assert_eq!(expected.len(), 19);
    for repetition in 0..9 {
        let whole = Instant::now();
        let (mut callback_ns, mut prepare_ns, mut serialize_ns) = (0, 0, 0);
        let (mut callbacks, mut traversals, mut refusals) = (0, 0, 0);
        let (mut input_bytes, mut output_bytes) = (0, 0);
        for (index, case) in cases.iter().enumerate() {
            assert_eq!(case["id"], expected[index]["id"]);
            let oracle = &expected[index]["source_prediction"];
            assert_eq!(oracle["version"], VERSION);
            assert_eq!(oracle["kind"], KIND);
            let input = case["input"].as_str().expect("fixed input text");
            let original = input.to_owned();
            input_bytes += input.len();
            if case["mode"] == "callback" {
                let mut root: Value = serde_json::from_str(input).expect("callback object");
                let before = root.clone();
                let attribute = case["current_attribute"].as_str().expect("attribute");
                let clock = Instant::now();
                let changed = b25d::replace_json_node(
                    root.as_object_mut().expect("root object"),
                    attribute,
                    &case["current_value"],
                );
                callback_ns += clock.elapsed().as_nanos();
                assert_eq!(oracle["status"], "PREPARED");
                assert!(root == oracle["json"], "callback full-tree prediction");
                assert_eq!(changed, root != before);
                black_box(&root);
                callbacks += 1;
            } else {
                assert_eq!(case["mode"], "traversal");
                let clock = Instant::now();
                let result = b25d::prepare(KIND, input, None);
                prepare_ns += clock.elapsed().as_nanos();
                traversals += 1;
                if oracle["status"] == "REFUSED" {
                    assert!(
                        matches!(result, Err(Error::NestedArray)),
                        "nested-array refusal"
                    );
                    assert!(oracle["native_partial_json"].is_object());
                    refusals += 1;
                } else {
                    assert_eq!(oracle["status"], "PREPARED");
                    let prepared = result.expect("selected traversal");
                    assert!(
                        prepared.json == oracle["json"],
                        "traversal full-tree prediction"
                    );
                    assert!(prepared.matched_kind);
                    assert_eq!(prepared.participant_version.as_deref(), Some(VERSION));
                    let clock = Instant::now();
                    let wire = prepared.to_json().expect("bounded serialization");
                    serialize_ns += clock.elapsed().as_nanos();
                    output_bytes += wire.len();
                    let replay = b25d::prepare(KIND, &wire, Some(VERSION)).expect("replay");
                    assert!(
                        !replay.changed && replay.json == prepared.json,
                        "replay state"
                    );
                    assert_eq!(replay.to_json().expect("replay wire"), wire);
                    black_box(wire);
                }
            }
            assert_eq!(input, original);
        }
        let whole_ns = whole.elapsed().as_nanos();
        assert_eq!((callbacks, traversals, refusals), (14, 5, 2));
        println!(
            "{}",
            json!({
                "schema": "B25D_CORPUS_SAMPLE/1",
                "scope": "RUST_SOURCE_PREDICTIONS_NOT_NATIVE_ACCEPTANCE",
                "mode": "full-corpus",
                "phase": if repetition < 2 { "warmup" } else { "raw" },
                "sample": if repetition < 2 { repetition } else { repetition - 2 },
                "pid": std::process::id(),
                "callbacks": callbacks,
                "traversals": traversals,
                "refusals": refusals,
                "input_bytes": input_bytes,
                "serialized_traversal_bytes": output_bytes,
                "callback_ns": callback_ns,
                "prepare_including_preflight_ns": prepare_ns,
                "serialization_ns": serialize_ns,
                "whole_including_validation_ns": whole_ns
            })
        );
    }
}
