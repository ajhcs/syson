// SPDX-License-Identifier: EPL-2.0
use serde_json::json;
use std::time::Instant;
use syson_migrations::{
    MAX_BYTES,
    b25d::{self, KIND, MAPPINGS, VERSION},
};

fn main() {
    for (workload, count) in [("small-control", 1), ("repeated-controls-1024", 1024)] {
        let node = format!(
            r#"{{"descriptionId":"{}","targetObjectId":"fixture-target"}}"#,
            MAPPINGS[0].0
        );
        let input = format!(
            r#"{{"token":-0,"opaque":{{"$serde_json::private::Number":"literal"}},"nodes":[{}]}}"#,
            vec![node; count].join(",")
        );
        assert!(input.len() <= MAX_BYTES);
        let starting_input = input.clone();
        let mut first_wire = None;
        let mut samples = Vec::with_capacity(7);
        for iteration in 0..9 {
            let started = Instant::now();
            let prepared = b25d::prepare(KIND, &input, None).unwrap();
            let nanos = started.elapsed().as_nanos();
            // Every oracle and serialization below is outside the measured interval.
            assert!(prepared.matched_kind && prepared.changed);
            assert_eq!(prepared.participant_version.as_deref(), Some(VERSION));
            assert_eq!(prepared.json["token"].to_string(), "-0");
            assert_eq!(
                prepared.json["opaque"]["$serde_json::private::Number"],
                "literal"
            );
            let nodes = prepared.json["nodes"].as_array().unwrap();
            assert_eq!(nodes.len(), count);
            for node in nodes {
                assert_eq!(node["descriptionId"], MAPPINGS[0].1);
                assert_eq!(node["targetObjectId"], "fixture-target");
            }
            let wire = prepared.to_json().unwrap();
            assert!(wire.len() <= MAX_BYTES);
            assert!(wire.contains(r#""token":-0"#));
            if let Some(first) = &first_wire {
                assert_eq!(&wire, first);
            } else {
                first_wire = Some(wire.clone());
            }
            assert_eq!(input, starting_input);
            let phase = if iteration < 2 { "warmup" } else { "raw" };
            let sample = if iteration < 2 {
                iteration
            } else {
                iteration - 2
            };
            println!(
                "{}",
                json!({
                    "workload": workload,
                    "phase": phase,
                    "sample": sample,
                    "nodes": count,
                    "input_bytes": input.len(),
                    "output_bytes": wire.len(),
                    "prepare_ns": nanos
                })
            );
            if iteration >= 2 {
                samples.push(nanos);
            }
        }
        samples.sort_unstable();
        println!(
            "{}",
            json!({
                "workload": workload,
                "raw_samples": 7,
                "min_ns": samples[0],
                "median_ns": samples[3],
                "max_ns": samples[6],
                "timed_phase": "prepare_only_including_internal_output_validation"
            })
        );
    }
}
