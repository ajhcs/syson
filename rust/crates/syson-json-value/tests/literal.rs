// SPDX-License-Identifier: EPL-2.0
use serde_json::{Map, Value};
use syson_json_value::{Error, HARD_DEPTH, Limit, Limits, Node, Statistics, parse};
const NUMBER: &str = "$serde_json::private::Number";
const RAW: &str = "$serde_json::private::RawValue";
fn materialize(raw: &str) -> Value {
    parse(raw, Limits::default(), || false)
        .unwrap()
        .into_value(|| false)
        .unwrap()
        .0
}
fn object(pairs: Vec<(&str, Value)>) -> Value {
    let mut map = Map::new();
    for (key, value) in pairs {
        map.insert(key.to_owned(), value);
    }
    Value::Object(map)
}
fn refusal(raw: &str, limits: Limits, expected: Error) {
    let result = parse(raw, limits, || false);
    assert!(matches!(result, Err(e) if e == expected));
}
#[test]
fn literal_private_keys_in_all_member_positions_use_a_manual_object_oracle() {
    for marker in [NUMBER, RAW] {
        for raw in [
            format!(r#"{{"{marker}":"123","a":false,"z":null}}"#),
            format!(r#"{{"a":false,"{marker}":"123","z":null}}"#),
            format!(r#"{{"a":false,"z":null,"{marker}":"123"}}"#),
        ] {
            let expected = object(vec![
                (marker, Value::String("123".into())),
                ("a", Value::Bool(false)),
                ("z", Value::Null),
            ]);
            assert_eq!(materialize(&raw), expected);
            let encoded = serde_json::to_string(&expected).unwrap();
            assert_eq!(materialize(&encoded), expected);
        }
        for value in ["null", "false", "0", "[]", "{}"] {
            let raw = format!(r#"{{"{marker}":{value}}}"#);
            assert!(materialize(&raw).is_object());
        }
    }
}
#[test]
fn nested_markers_and_encoded_node_strings_remain_literal_and_cannot_create_nodes() {
    let raw = r#"{"content":[],"vendor":{"$serde_json::private::RawValue":"{\"id\":\"A\",\"eClass\":\"sysml:Namespace\",\"data\":{\"elementId\":\"B\"}}"},"other":[{"$serde_json::private::Number":"-0"}]}"#;
    let encoded_node = r#"{"id":"A","eClass":"sysml:Namespace","data":{"elementId":"B"}}"#;
    let expected = object(vec![
        ("content", Value::Array(vec![])),
        (
            "vendor",
            object(vec![(RAW, Value::String(encoded_node.into()))]),
        ),
        (
            "other",
            Value::Array(vec![object(vec![(NUMBER, Value::String("-0".into()))])]),
        ),
    ]);
    assert_eq!(materialize(raw), expected);
    fn physical_nodes(node: &Node<'_>) -> usize {
        match node {
            Node::Object(rows) => {
                usize::from(rows.iter().any(|(k, _)| k == "eClass"))
                    + rows.iter().map(|(_, v)| physical_nodes(v)).sum::<usize>()
            }
            Node::Array(rows) => rows.iter().map(physical_nodes).sum(),
            _ => 0,
        }
    }
    let parsed = parse(raw, Limits::default(), || false).unwrap();
    assert_eq!(physical_nodes(parsed.root()), 0);
    assert_eq!(parsed.raw(), raw);
}
#[test]
fn raw_order_whitespace_string_escape_spelling_and_unicode_have_distinct_authorities() {
    let raw = " \r\n{\"z\":\"\\u00e9\",\"a\":\"é🚗\",\"e\\u0301\":null,\"é\":false} \t";
    let parsed = parse(raw, Limits::default(), || false).unwrap();
    assert_eq!(parsed.raw().as_ptr(), raw.as_ptr());
    let Node::Object(rows) = parsed.root() else {
        panic!("object expected")
    };
    assert_eq!(
        rows.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
        ["z", "a", "e\u{301}", "é"]
    );
    assert_eq!(rows[0].1, Node::String("é".into()));
    let converted = parsed.into_value(|| false).unwrap();
    assert_eq!(converted.1, raw);
    assert_eq!(converted.0["a"], Value::String("é🚗".into()));
    assert_eq!(
        converted
            .0
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["a", "e\u{301}", "z", "é"]
    );
    assert!(converted.0.get("e\u{301}").is_some());
}
#[test]
fn exact_number_tokens_survive_tree_materialization_and_safe_reparse() {
    for raw in [
        "-0",
        "0",
        "0.00",
        "1e+00",
        "1E-999999",
        "1e-000",
        "-9223372036854775809",
        "900719925474099312345",
        "1.7976931348623159e308",
        "4.9e-324",
        "0.0000000001",
    ] {
        let parsed = parse(raw, Limits::default(), || false).unwrap();
        assert_eq!(parsed.root(), &Node::Number(raw));
        let result = parsed.into_value(|| false).unwrap();
        let Value::Number(n) = &result.0 else {
            panic!("number expected")
        };
        assert_eq!(n.to_string(), raw);
        let encoded = serde_json::to_string(&result.0).unwrap();
        assert_eq!(encoded, raw);
        assert_eq!(materialize(&encoded).to_string(), raw);
    }
    let enormous = "9".repeat(10_000);
    assert_eq!(materialize(&enormous).to_string(), enormous);
}
#[test]
fn strict_syntax_blocks_unchecked_number_construction_and_malformed_adjacent_values() {
    for raw in [
        "",
        "01",
        "-01",
        "1x",
        "1e",
        "1e+",
        "0.",
        "-0x1",
        "NaN",
        "Infinity",
        "truefalse",
        "{}[]",
        "1 2",
        "\"a\" \"b\"",
        "[1,]",
        "{\"a\",1}",
        "{\"a\":1 \"b\":2}",
        r#""\x41""#,
        r#""\uD800""#,
    ] {
        assert!(parse(raw, Limits::default(), || false).is_err(), "{raw}");
    }
}
#[test]
fn decoded_duplicate_keys_refuse_including_escaped_and_private_names() {
    for raw in [
        r#"{"a":0,"a":1}"#,
        r#"{"a":0,"\u0061":1}"#,
        r#"{"x":[{"$serde_json::private::Number":"1","$serde_json::private::Number":"2"}]}"#,
        r#"{"$serde_json::private::RawValue":{},"$serde_json::private::RawValue":null}"#,
    ] {
        refusal(raw, Limits::default(), Error::DuplicateKey);
    }
    assert!(
        parse(
            r#"{"a":0,"A":1,"é":2,"e\u0301":3}"#,
            Limits::default(),
            || false
        )
        .is_ok()
    );
}
#[test]
fn exact_byte_value_key_and_scalar_charges_have_one_less_refusal_boundaries() {
    let raw = r#"{"a":"é","b":-0}"#;
    let exact = Limits {
        bytes: raw.len(),
        depth: 2,
        values: 3,
        keys: 2,
        scalar_bytes: 8,
    };
    let parsed = parse(raw, exact, || false).unwrap();
    assert_eq!(
        parsed.statistics(),
        Statistics {
            values: 3,
            keys: 2,
            scalar_bytes: 8
        }
    );
    let converted = parsed.into_value(|| false).unwrap();
    assert_eq!(converted.1, raw);
    for (limits, kind) in [
        (
            Limits {
                bytes: raw.len() - 1,
                ..exact
            },
            Limit::Bytes,
        ),
        (Limits { depth: 1, ..exact }, Limit::Depth),
        (Limits { values: 2, ..exact }, Limit::Values),
        (Limits { keys: 1, ..exact }, Limit::Keys),
        (
            Limits {
                scalar_bytes: 7,
                ..exact
            },
            Limit::ScalarBytes,
        ),
    ] {
        refusal(raw, limits, Error::Limit(kind));
    }
    assert!(
        parse(
            "{}",
            Limits {
                values: 1,
                keys: 0,
                scalar_bytes: 0,
                ..Limits::default()
            },
            || false
        )
        .is_ok()
    );
    assert!(
        parse(
            "true",
            Limits {
                values: 1,
                keys: 0,
                scalar_bytes: 0,
                ..Limits::default()
            },
            || false
        )
        .is_ok()
    );
}
#[test]
fn escaped_keys_are_charged_before_decoding_and_resource_limits_precede_copying() {
    let raw = r#"{"\u0061":"\u0062"}"#;
    let limits = Limits {
        scalar_bytes: 18,
        ..Limits::default()
    };
    assert_eq!(
        parse(raw, limits, || false)
            .unwrap()
            .statistics()
            .scalar_bytes,
        18
    );
    refusal(
        raw,
        Limits {
            scalar_bytes: 17,
            ..limits
        },
        Error::Limit(Limit::ScalarBytes),
    );
    // Malformed escapes would fail typed decoding; these resource errors happen first.
    refusal(
        r#""\q""#,
        Limits {
            values: 0,
            ..Limits::default()
        },
        Error::Limit(Limit::Values),
    );
    refusal(
        r#"{"\q":null}"#,
        Limits {
            keys: 0,
            ..Limits::default()
        },
        Error::Limit(Limit::Keys),
    );
    refusal(
        r#"{"\q":null}"#,
        Limits {
            scalar_bytes: 0,
            ..Limits::default()
        },
        Error::Limit(Limit::ScalarBytes),
    );
    refusal(
        "not JSON",
        Limits {
            bytes: 0,
            ..Limits::default()
        },
        Error::Limit(Limit::Bytes),
    );
}
#[test]
fn depth208_and_exact_safe_depth_fit_while_over_depth_and_invalid_configuration_refuse() {
    for depth in [208, HARD_DEPTH] {
        let raw = format!("{}null{}", "[".repeat(depth - 1), "]".repeat(depth - 1));
        let parsed = parse(
            &raw,
            Limits {
                depth,
                values: depth,
                ..Limits::default()
            },
            || false,
        )
        .unwrap();
        assert_eq!(parsed.statistics().values, depth);
        assert_eq!(parsed.into_value(|| false).unwrap().1, raw);
        refusal(
            &raw,
            Limits {
                depth: depth - 1,
                ..Limits::default()
            },
            Error::Limit(Limit::Depth),
        );
    }
    refusal(
        "null",
        Limits {
            depth: 0,
            ..Limits::default()
        },
        Error::InvalidLimits,
    );
    refusal(
        "null",
        Limits {
            depth: HARD_DEPTH + 1,
            ..Limits::default()
        },
        Error::InvalidLimits,
    );
    refusal(
        "null",
        Limits {
            values: usize::MAX,
            ..Limits::default()
        },
        Error::InvalidLimits,
    );
}
#[test]
fn cancellation_polls_long_strings_whitespace_and_numbers_and_returns_no_partial_value() {
    assert!(matches!(
        parse("null", Limits::default(), || true),
        Err(Error::Cancelled)
    ));
    for raw in [
        format!("\"{}\"", "x".repeat(50_000)),
        format!("{}null", " ".repeat(50_000)),
        "9".repeat(50_000),
    ] {
        let mut polls = 0;
        assert!(matches!(
            parse(&raw, Limits::default(), || {
                polls += 1;
                polls == 8
            }),
            Err(Error::Cancelled)
        ));
        assert_eq!(polls, 8);
    }
    let raw = "[1,2,3,4,5,6,7,8]";
    let parsed = parse(raw, Limits::default(), || false).unwrap();
    let mut polls = 0;
    let result = parsed.into_value(|| {
        polls += 1;
        polls > 5
    });
    assert!(matches!(result, Err(Error::Cancelled)));
    assert_eq!(raw, "[1,2,3,4,5,6,7,8]");
}
#[test]
fn selected_model_and_catalogue_shapes_can_consume_the_safe_value_without_adoption_claims() {
    let raw = r#"{"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{"id":"node-A","eClass":"sysml:Package","data":{"elementId":"semantic-B","vendor":{"$serde_json::private::Number":"123"}}}]}"#;
    let safe = parse(raw, Limits::default(), || false)
        .unwrap()
        .into_value(|| false)
        .unwrap();
    let (value, carrier) = safe;
    assert_eq!(carrier, raw);
    assert_eq!(value["content"][0]["id"], Value::String("node-A".into()));
    assert_eq!(
        value["content"][0]["data"]["elementId"],
        Value::String("semantic-B".into())
    );
    assert_eq!(
        value["content"][0]["data"]["vendor"],
        object(vec![(NUMBER, Value::String("123".into()))])
    );
    // This exercises the intended return shape, not model/catalogue adoption or native execution.
}
