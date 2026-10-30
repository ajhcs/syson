// SPDX-License-Identifier: EPL-2.0
//! Explicit compatibility workload; no matched strict/native speed comparison.
use serde_json::{Value, json};
use std::time::Instant;
use syson_json_value::{
    DuplicateKeys, Error, Limit, Limits, Node, parse, parse_with_duplicate_keys,
};
fn summary(raw: &[u128]) -> Value {
    assert_eq!(raw.len(), 7);
    let mut sorted = raw.to_vec();
    sorted.sort_unstable();
    json!({"raw_ns":raw,"min_ns":sorted[0],"median_ns":sorted[3],"max_ns":sorted[6]})
}
fn main() {
    for keys in [1_000usize, 5_000, 10_000] {
        let distinct = keys / 4;
        let fields: Vec<_> = (0..keys)
            .map(|i| format!("\"k{}\":{i}", i % distinct))
            .collect();
        let source = format!("{{{}}}", fields.join(","));
        let scalar_bytes: usize = (0..keys)
            .map(|i| 2 * format!("k{}", i % distinct).len() + i.to_string().len())
            .sum();
        assert!(matches!(
            parse(&source, Limits::default(), || false),
            Err(Error::DuplicateKey)
        ));
        assert!(matches!(
            parse_with_duplicate_keys(
                &source,
                Limits {
                    keys: keys - 1,
                    ..Limits::default()
                },
                DuplicateKeys::LastWins,
                || false
            ),
            Err(Error::Limit(Limit::Keys))
        ));
        let (mut parsing, mut materializing) = (Vec::new(), Vec::new());
        for repetition in 0..9 {
            let started = Instant::now();
            let literal = parse_with_duplicate_keys(
                &source,
                Limits::default(),
                DuplicateKeys::LastWins,
                || false,
            )
            .unwrap();
            let parse_ns = started.elapsed().as_nanos();
            let stats = literal.statistics();
            assert_eq!(
                (stats.keys, stats.values, stats.scalar_bytes),
                (keys, keys + 1, scalar_bytes)
            );
            assert_eq!(literal.raw(), source);
            let Node::Object(rows) = literal.root() else {
                panic!("object expected")
            };
            assert_eq!(rows.len(), distinct);
            for (i, (key, node)) in rows.iter().enumerate() {
                let number = (i + 3 * distinct).to_string();
                assert_eq!(key, &format!("k{i}"));
                assert_eq!(node, &Node::Number(&number));
            }
            let started = Instant::now();
            let (value, raw) = literal.into_value(|| false).unwrap();
            let materialize_ns = started.elapsed().as_nanos();
            assert_eq!(raw, source);
            assert_eq!(value.as_object().unwrap().len(), distinct);
            for i in 0..distinct {
                assert_eq!(
                    value[format!("k{i}")].as_u64(),
                    Some((i + 3 * distinct) as u64)
                );
            }
            if repetition >= 2 {
                parsing.push(parse_ns);
                materializing.push(materialize_ns);
            }
        }
        println!(
            "{}",
            json!({"policy":"last_wins","physical_keys":keys,"distinct_keys":distinct,
            "source_bytes":source.len(),"warmups":2,"samples":7,
            "parse":summary(&parsing),"materialize":summary(&materializing)})
        );
    }
}
