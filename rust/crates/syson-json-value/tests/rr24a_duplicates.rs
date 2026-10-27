// SPDX-License-Identifier: EPL-2.0
use std::cell::Cell;
use syson_json_value::{
    DuplicateKeys, Error, Limit, Limits, Node, parse, parse_with_duplicate_keys,
};
fn compatible(text: &str) -> syson_json_value::Literal<'_> {
    parse_with_duplicate_keys(text, Limits::default(), DuplicateKeys::LastWins, || false).unwrap()
}
#[test]
fn strict_api_stays_strict_and_explicit_last_wins_keeps_first_position() {
    let text = r#"{ "z":0,"a":1,"z":2,"nested":{"b":false,"b":true} }"#;
    assert!(matches!(
        parse(text, Limits::default(), || false),
        Err(Error::DuplicateKey)
    ));
    assert!(matches!(
        parse_with_duplicate_keys(text, Limits::default(), DuplicateKeys::Reject, || false),
        Err(Error::DuplicateKey)
    ));
    let literal = compatible(text);
    assert_eq!(literal.raw(), text);
    let Node::Object(rows) = literal.root() else {
        panic!("object expected")
    };
    assert_eq!(
        rows.iter().map(|(key, _)| key.as_str()).collect::<Vec<_>>(),
        ["z", "a", "nested"]
    );
    assert_eq!(rows[0].1, Node::Number("2"));
    assert_eq!(
        rows[2].1,
        Node::Object(vec![("b".into(), Node::Boolean(true))])
    );
    let (value, raw) = literal.into_value(|| false).unwrap();
    assert_eq!(raw, text);
    assert_eq!(value["z"].to_string(), "2");
    assert_eq!(value["nested"]["b"], true);
}
#[test]
fn escaped_equivalent_keys_and_replaced_containers_keep_literal_numeric_and_marker_values() {
    let text = r#"{"ns":{"z":"wrong","a":"other","\u007a":"sysml"},"vendor":false,"vendor":{"$serde_json::private::Number":"123","$serde_json::private::RawValue":"{\"fake\":true}"},"n":0,"n":-0,"big":1e-99999}"#;
    let literal = compatible(text);
    let Node::Object(rows) = literal.root() else {
        panic!("object expected")
    };
    assert_eq!(
        rows[0].1,
        Node::Object(vec![
            ("z".into(), Node::String("sysml".into())),
            ("a".into(), Node::String("other".into()))
        ])
    );
    let (value, _) = literal.into_value(|| false).unwrap();
    let mut expected = serde_json::Map::new();
    expected.insert(
        "$serde_json::private::Number".into(),
        serde_json::Value::String("123".into()),
    );
    expected.insert(
        "$serde_json::private::RawValue".into(),
        serde_json::Value::String(r#"{"fake":true}"#.into()),
    );
    assert_eq!(value["vendor"], serde_json::Value::Object(expected));
    assert_eq!(value["n"].to_string(), "-0");
    assert_eq!(value["big"].to_string(), "1e-99999");
}
#[test]
fn overwritten_values_still_consume_each_source_budget_before_collapse() {
    let text = r#"{"a":1,"a":2}"#;
    let exact = Limits {
        bytes: text.len(),
        depth: 2,
        values: 3,
        keys: 2,
        scalar_bytes: 6,
    };
    let literal =
        parse_with_duplicate_keys(text, exact, DuplicateKeys::LastWins, || false).unwrap();
    assert_eq!(
        (
            literal.statistics().values,
            literal.statistics().keys,
            literal.statistics().scalar_bytes
        ),
        (3, 2, 6)
    );
    for (limits, expected) in [
        (
            Limits {
                bytes: text.len() - 1,
                ..exact
            },
            Limit::Bytes,
        ),
        (Limits { depth: 1, ..exact }, Limit::Depth),
        (Limits { values: 2, ..exact }, Limit::Values),
        (Limits { keys: 1, ..exact }, Limit::Keys),
        (
            Limits {
                scalar_bytes: 5,
                ..exact
            },
            Limit::ScalarBytes,
        ),
    ] {
        assert!(
            matches!(parse_with_duplicate_keys(text, limits, DuplicateKeys::LastWins, || false), Err(Error::Limit(actual)) if actual==expected)
        );
    }
    let overwritten = r#"{"a":[[[null]]],"a":0}"#;
    assert!(matches!(
        parse_with_duplicate_keys(
            overwritten,
            Limits {
                depth: 3,
                ..Limits::default()
            },
            DuplicateKeys::LastWins,
            || false
        ),
        Err(Error::Limit(Limit::Depth))
    ));
}
#[test]
fn cancellation_reaches_build_and_materialize_without_returning_partial_values() {
    let text = r#"{"a":[1,2,3],"a":[4,5,6]}"#;
    let polls = Cell::new(0);
    compatible(text); // Positive control and stable input independent of callback.
    parse_with_duplicate_keys(text, Limits::default(), DuplicateKeys::LastWins, || {
        polls.set(polls.get() + 1);
        false
    })
    .unwrap();
    let total = polls.get();
    assert!(total > 5);
    for stop in [1, total / 2, total] {
        let calls = Cell::new(0);
        assert!(matches!(
            parse_with_duplicate_keys(text, Limits::default(), DuplicateKeys::LastWins, || {
                calls.set(calls.get() + 1);
                calls.get() >= stop
            }),
            Err(Error::Cancelled)
        ));
    }
    assert!(matches!(
        compatible(text).into_value(|| true),
        Err(Error::Cancelled)
    ));
    assert_eq!(
        compatible(text).into_value(|| false).unwrap().0["a"][0].to_string(),
        "4"
    );
}
#[test]
fn complete_syntax_validation_still_refuses_malformed_overwritten_input() {
    for text in [
        r#"{"a":01,"a":2}"#,
        r#"{"a":"\uD800","a":"ok"}"#,
        r#"{"a":1,"a":2} trailing"#,
    ] {
        assert!(
            parse_with_duplicate_keys(text, Limits::default(), DuplicateKeys::LastWins, || false)
                .is_err()
        );
    }
}
