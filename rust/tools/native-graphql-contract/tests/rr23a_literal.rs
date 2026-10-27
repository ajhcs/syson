// SPDX-License-Identifier: EPL-2.0
//! Independent literal oracles: marker expectations never cross a Value parser.
use serde_json::{Map, Value};
use syson_native_graphql_contract::inspect;
const NATIVE: &str = include_str!("fixtures/native-official-label-introspection.json");
fn inserted(member: &str) -> String {
    let needle = r#""__schema":{"#;
    assert_eq!(NATIVE.matches(needle).count(), 1);
    NATIVE.replacen(needle, &(needle.to_owned() + member + ","), 1)
}
fn literal_object(key: &str, value: &str) -> Value {
    let mut map = Map::new();
    map.insert(key.into(), Value::String(value.into()));
    Value::Object(map)
}
#[test]
fn private_number_named_object_is_preserved_as_an_ordinary_object() {
    let input = inserted(r#""vendor":{"$serde_json::private::Number":"123"}"#);
    let before = input.clone();
    let output = inspect(&input).unwrap();
    assert_eq!(
        output["schema"]["vendor"],
        literal_object("$serde_json::private::Number", "123")
    );
    assert_eq!(input, before);
}
#[test]
fn private_raw_named_object_preserves_its_string_without_injecting_content() {
    let input = inserted(
        r#""vendor":{"$serde_json::private::RawValue":"{\"nested\":[true],\"label\":\"雪😀\"}"}"#,
    );
    let before = input.clone();
    let output = inspect(&input).unwrap();
    assert_eq!(
        output["schema"]["vendor"],
        literal_object(
            "$serde_json::private::RawValue",
            r#"{"nested":[true],"label":"雪😀"}"#
        )
    );
    assert_eq!(input, before);
}
#[test]
fn unknown_numeric_values_keep_their_exact_valid_source_spelling() {
    let huge = "9".repeat(4096);
    for token in ["-0", "0.00", "1E+000", "1e-99999", "1e99999", huge.as_str()] {
        let input = inserted(&(r#""vendor":"#.to_owned() + token));
        let output = inspect(&input).unwrap();
        let Value::Number(number) = &output["schema"]["vendor"] else {
            panic!("numeric token became a non-number")
        };
        assert_eq!(number.to_string(), token);
    }
}

#[test]
fn selected_strings_and_graphql_default_literals_remain_opaque_strings() {
    let input = NATIVE.replacen(
        r#""description":null"#,
        r#""description":"{\"$serde_json::private::Number\":\"123\"} 雪😀\n\u0041""#,
        1,
    );
    let output = inspect(&input).unwrap();
    assert_eq!(
        output["schema"]["description"].as_str(),
        Some("{\"$serde_json::private::Number\":\"123\"} 雪😀\nA")
    );
    let needle = r#""defaultValue":null"#;
    assert!(NATIVE.contains(needle));
    let input = NATIVE.replacen(
        needle,
        r#""defaultValue":"{value: -0, list: [1E+000], text: \"雪😀\"}""#,
        1,
    );
    let output = inspect(&input).unwrap();
    fn defaults(value: &Value) -> Vec<&str> {
        let mut found = Vec::new();
        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    if key == "defaultValue"
                        && let Some(s) = value.as_str()
                    {
                        found.push(s);
                    }
                    found.extend(defaults(value));
                }
            }
            Value::Array(rows) => {
                for row in rows {
                    found.extend(defaults(row));
                }
            }
            _ => (),
        }
        found
    }
    assert!(defaults(&output["schema"]).contains(&"{value: -0, list: [1E+000], text: \"雪😀\"}"));
}

#[test]
fn markers_and_ordinary_numbers_cannot_replace_typed_metadata() {
    let markers = [
        r#"{"$serde_json::private::Number":"123"}"#,
        r#"{"$serde_json::private::RawValue":"\"description\""}"#,
        r#"{"$serde_json::private::RawValue":"false"}"#,
    ];
    for raw in markers.into_iter().chain(["-0", "0.0", "1e0"]) {
        for needle in [r#""description":null"#, r#""isOneOf":null"#] {
            assert!(NATIVE.contains(needle));
            let key = needle.strip_suffix("null").unwrap();
            assert!(inspect(&NATIVE.replacen(needle, &(key.to_owned() + raw), 1)).is_err());
        }
    }
}

#[test]
fn decoded_duplicate_keys_keep_last_values_without_marker_coercion() {
    let input = inserted(r#""vendor":{"x":0,"\u0078":{"$serde_json::private::Number":"123"}}"#);
    let output = inspect(&input).unwrap();
    let vendor = output["schema"]["vendor"].as_object().unwrap();
    assert_eq!(vendor.len(), 1);
    assert_eq!(
        vendor["x"],
        literal_object("$serde_json::private::Number", "123")
    );
}

fn values(value: &Value) -> usize {
    // Independent ordinary-corpus DOM count; no duplicate or marker oracle is
    // parsed here. Overwritten occurrences below are counted arithmetically.
    1 + match value {
        Value::Object(map) => map.values().map(values).sum(),
        Value::Array(array) => array.iter().map(values).sum(),
        _ => 0,
    }
}
#[test]
fn physical_overwritten_values_are_bounded_before_last_wins_collapse() {
    use syson_native_graphql_contract::MAX_LITERAL_VALUES;
    let ordinary: Value = serde_json::from_str(NATIVE).unwrap();
    let repetitions = MAX_LITERAL_VALUES - values(&ordinary) - 1;
    let member = r#""x":null"#;
    let input =
        inserted(&(r#""vendor":{"#.to_owned() + &vec![member; repetitions].join(",") + "}"));
    let output = inspect(&input).unwrap();
    assert_eq!(output["schema"]["vendor"].as_object().unwrap().len(), 1);
    assert!(output["schema"]["vendor"]["x"].is_null());
    let input =
        inserted(&(r#""vendor":{"#.to_owned() + &vec![member; repetitions + 1].join(",") + "}"));
    assert!(inspect(&input).unwrap_err().contains("Values"));
}
#[test]
fn byte_and_whole_value_depth_edges_are_explicit() {
    use syson_native_graphql_contract::{MAX_BYTES, MAX_LITERAL_DEPTH};
    let mut input = NATIVE.to_owned();
    input.push_str(&" ".repeat(MAX_BYTES - input.len()));
    assert!(inspect(&input).is_ok());
    input.push(' ');
    assert!(inspect(&input).unwrap_err().contains("byte budget"));
    for extra in [0, 1] {
        // Response/data/schema occupy depths1/2/3; the scalar is depth4+n.
        let n = MAX_LITERAL_DEPTH - 4 + extra;
        let input = inserted(&(r#""vendor":"#.to_owned() + &"[".repeat(n) + "0" + &"]".repeat(n)));
        let output = inspect(&input);
        assert_eq!(output.is_ok(), extra == 0);
        if extra != 0 {
            assert!(output.unwrap_err().contains("Depth"));
        }
    }
}
#[test]
fn unchanged_native_corpus_remains_selected_schema_and_badfaith_remains_error() {
    let original: Value = serde_json::from_str(NATIVE).unwrap();
    let output = inspect(NATIVE).unwrap();
    assert_eq!(output["schema"], original["data"]["__schema"]);
    assert_eq!(output["counts"]["types"], 671);
    assert_eq!(output["counts"]["members"], 3148);
    assert_eq!(output["C01"], "OPEN");
    let refusal = include_str!("fixtures/native-badfaith-introspection.json");
    assert!(inspect(refusal).unwrap_err().contains("GraphQL errors"));
}
