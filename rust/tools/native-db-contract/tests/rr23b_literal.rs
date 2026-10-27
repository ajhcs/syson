// SPDX-License-Identifier: EPL-2.0
//! Typed-slot literal oracles. Native DB descriptors have no opaque object slot.
use serde_json::{Map, Value};
use syson_native_db_contract::project;
const NATIVE: &[u8] = include_bytes!("fixtures/native-01-catalog.json");
fn input() -> Value {
    serde_json::from_slice(NATIVE).unwrap()
}
fn marker(key: &str, value: String) -> Value {
    let mut fields = Map::new();
    fields.insert(key.into(), Value::String(value));
    Value::Object(fields)
}
fn bytes(input: &Value) -> Vec<u8> {
    serde_json::to_vec(input).unwrap()
}
#[test]
fn private_number_object_is_not_an_unsigned_oid() {
    let mut input = input();
    let oid = input["tables"][0]["oid"].to_string();
    input["tables"][0]["oid"] = marker("$serde_json::private::Number", oid);
    let raw = bytes(&input);
    let before = raw.clone();
    assert!(
        project(&raw).is_err(),
        "literal marker object was accepted as a PostgreSQL OID"
    );
    assert_eq!(raw, before);
}
#[test]
fn private_raw_object_is_not_a_capture_descriptor() {
    let mut input = input();
    let capture = serde_json::to_string(&input["capture"]).unwrap();
    input["capture"] = marker("$serde_json::private::RawValue", capture);
    let raw = bytes(&input);
    assert!(
        project(&raw).is_err(),
        "literal marker object was reinterpreted as valid capture metadata"
    );
}
fn raw_order(token: &str) -> Vec<u8> {
    let mut input = input();
    input["changelog"][0]["orderexecuted"] = Value::String("RR23B_INTEGER_TOKEN".into());
    String::from_utf8(bytes(&input))
        .unwrap()
        .replace("\"RR23B_INTEGER_TOKEN\"", token)
        .into_bytes()
}
#[test]
fn signed_negative_zero_retains_the_original_typed_integer_refusal() {
    let error = project(&raw_order("-0")).unwrap_err();
    assert!(
        error.contains("invalid migration execution order"),
        "{error}"
    );
}
fn first_migration(output: &Value) -> &Value {
    &output["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["kind"] == "migration" && row["observed"]["position"] == 0)
        .unwrap()["observed"]
}
#[test]
fn canonical_typed_integer_extremes_and_native_numeric_identities_survive() {
    for token in ["-9223372036854775808", "0", "9223372036854775807"] {
        let output = project(&raw_order(token)).unwrap();
        let Value::Number(number) = &first_migration(&output)["orderexecuted"] else {
            panic!("number expected")
        };
        assert_eq!(number.to_string(), token);
        assert_eq!(output["migration_order"].as_array().unwrap().len(), 20);
        assert_eq!(output["obligations"].as_array().unwrap().len(), 164);
    }
    let mut input = input();
    input["capture"]["server_version_num"] = Value::from(u64::MAX);
    let output = project(&bytes(&input)).unwrap();
    assert_eq!(
        output["capture"]["server_version_num"].to_string(),
        "18446744073709551615"
    );
}
#[test]
fn marker_named_json_sql_and_unicode_stay_exact_literal_strings() {
    let text = r#"雪😀\n SQL '{"$serde_json::private::Number":"123","$serde_json::private::RawValue":"{\"fake\":true}"}'::text -- literal only"#;
    let mut input = input();
    input["changelog"][0]["comments"] = Value::String(text.into());
    input["columns"][0]["default_expr"] = Value::String(text.into());
    let output = project(&bytes(&input)).unwrap();
    assert_eq!(first_migration(&output)["comments"].as_str(), Some(text));
    let column = output["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| {
            row["kind"] == "column" && row["observed"]["default_expr"].as_str() == Some(text)
        })
        .unwrap();
    assert_eq!(
        column["observed"]["default_expr"],
        Value::String(text.into())
    );
}

#[test]
fn literal_objects_refuse_other_typed_slots_without_becoming_flags_strings_or_arrays() {
    for slot in ["flag", "nullable", "array", "root", "schema"] {
        let mut input = input();
        match slot {
            "flag" => {
                input["columns"][0]["not_null"] =
                    marker("$serde_json::private::RawValue", "true".into())
            }
            "nullable" => {
                input["columns"][0]["default_expr"] =
                    marker("$serde_json::private::RawValue", "\"opaque SQL\"".into())
            }
            "array" => {
                let raw = serde_json::to_string(&input["tables"]).unwrap();
                input["tables"] = marker("$serde_json::private::RawValue", raw);
            }
            "root" => {
                input = marker(
                    "$serde_json::private::RawValue",
                    serde_json::to_string(&input).unwrap(),
                )
            }
            "schema" => {
                input["schema_version"] = marker("$serde_json::private::Number", "1".into())
            }
            _ => unreachable!(),
        }
        assert!(
            project(&bytes(&input)).is_err(),
            "{slot} marker was reinterpreted"
        );
    }
}
#[test]
fn noninteger_and_out_of_range_tokens_preserve_typed_refusals() {
    let huge = "9".repeat(16_384);
    for token in [
        "-0",
        "0.0",
        "1e0",
        "1E+000",
        "1e99999",
        "9223372036854775808",
        "-9223372036854775809",
        huge.as_str(),
    ] {
        assert!(
            project(&raw_order(token)).is_err(),
            "accepted signed token {token}"
        );
    }
    for token in [
        "-0",
        "0",
        "4294967296",
        "1.0",
        "1e0",
        "18446744073709551616",
    ] {
        let mut input = input();
        input["tables"][0]["oid"] = Value::String("RR23B_OID_TOKEN".into());
        let raw = String::from_utf8(bytes(&input))
            .unwrap()
            .replace("\"RR23B_OID_TOKEN\"", token);
        assert!(
            project(raw.as_bytes()).is_err(),
            "accepted OID token {token}"
        );
    }
}
fn values(value: &Value) -> usize {
    1 + match value {
        Value::Object(rows) => rows.values().map(values).sum(),
        Value::Array(rows) => rows.iter().map(values).sum(),
        _ => 0,
    }
}
#[test]
fn exact_physical_value_budget_admits_valid_typed_indexes_and_duplicate_plus_one_refuses() {
    use syson_native_db_contract::{MAX_BYTES, MAX_VALUES};
    let mut input = input();
    let mut count = values(&input);
    let template = input["indexes"][0].clone();
    let rows = input["indexes"].as_array_mut().unwrap();
    let mut serial = 0u32;
    loop {
        let mut row = template.clone();
        row["oid"] = Value::from(3_000_000_000u32 + serial);
        row["name"] = Value::String(format!("rr23b_synthetic_{serial}"));
        row["key_attnums"] = Value::Array(vec![Value::from(0); 32_767]);
        let added = values(&row);
        if count + added > MAX_VALUES {
            break;
        }
        rows.push(row);
        count += added;
        serial += 1;
    }
    // Each numeric zero is the existing expression-column sentinel. Typed
    // relation/column/index integrity and collection caps still apply normally.
    for row in rows {
        let array = row["key_attnums"].as_array_mut().unwrap();
        let added = (MAX_VALUES - count).min(32_767 - array.len());
        array.extend((0..added).map(|_| Value::from(0)));
        count += added;
        if count == MAX_VALUES {
            break;
        }
    }
    assert_eq!(values(&input), MAX_VALUES);
    assert!(input["indexes"].as_array().unwrap().len() < 8192);
    let raw = bytes(&input);
    assert!(raw.len() < MAX_BYTES);
    eprintln!("RR23b physical_values={count}; raw_bytes={}", raw.len());
    drop(input); // Avoid retaining the fixture DOM during the actual reader.
    let output = project(&raw).unwrap();
    assert_eq!(output["migration_order"].as_array().unwrap().len(), 20);
    assert!(
        output["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["kind"] == "index" && row["key"][1] == "rr23b_synthetic_0")
    );
    drop(output);
    // Same final descriptor tree, one extra physical source value before the
    // original schema_version: LastWins must not refund admission statistics.
    let mut over = br#"{"schema_version":1,"#.to_vec();
    over.extend_from_slice(&raw[1..]);
    drop(raw);
    assert!(over.len() < MAX_BYTES);
    let error = project(&over).unwrap_err();
    assert!(error.contains("Values"), "{error}");
}
#[test]
fn duplicate_compatibility_byte_depth_and_late_failure_keep_previous_output_intact() {
    use syson_native_db_contract::{MAX_BYTES, MAX_DEPTH};
    let output = project(NATIVE).unwrap();
    let before = output.to_string();
    let mut duplicate = br#"{"\u0073chema_version":999,"#.to_vec();
    let compact = bytes(&input());
    duplicate.extend_from_slice(&compact[1..]);
    assert_eq!(project(&duplicate).unwrap(), output);
    for extra in [0, 1] {
        // Root depth1 plus126arrays plus a scalar is exact depth128. The
        // original valid schema_version overwrites this value afterwards.
        let nesting = MAX_DEPTH - 2 + extra;
        let mut deep = format!(
            "{{\"schema_version\":{}0{},",
            "[".repeat(nesting),
            "]".repeat(nesting)
        )
        .into_bytes();
        deep.extend_from_slice(&compact[1..]);
        if extra == 0 {
            assert_eq!(project(&deep).unwrap(), output);
        } else {
            assert!(project(&deep).unwrap_err().contains("Depth"));
        }
    }
    let mut exact = compact;
    exact.resize(MAX_BYTES, b' ');
    assert_eq!(project(&exact).unwrap(), output);
    exact.push(b' ');
    assert!(project(&exact).unwrap_err().contains("byte limit"));
    let mut input = input();
    let last = input["changelog"].as_array().unwrap().len() - 1;
    input["changelog"][last]["orderexecuted"] = Value::String("late invalid integer".into());
    assert!(
        project(&bytes(&input))
            .unwrap_err()
            .contains("invalid migration execution order")
    );
    assert_eq!(output.to_string(), before);
}
#[test]
fn actual_corpus_retains_all_164_descriptors_and_20_ordered_migrations() {
    let input = input();
    let output = project(NATIVE).unwrap();
    assert_eq!(output["capture"], input["capture"]);
    let obligations = output["obligations"].as_array().unwrap();
    for (field, kind, expected) in [
        ("tables", "table", 15),
        ("columns", "column", 87),
        ("constraints", "constraint", 26),
        ("indexes", "index", 16),
        ("changelog", "migration", 20),
    ] {
        let raw_rows = input[field].as_array().unwrap();
        let projected: Vec<_> = obligations
            .iter()
            .filter(|row| row["kind"] == kind)
            .collect();
        assert_eq!(raw_rows.len(), expected);
        assert_eq!(projected.len(), expected);
        for row in raw_rows {
            assert!(projected.iter().any(|item| item["observed"] == *row));
        }
    }
    for row in input["changelog"].as_array().unwrap() {
        let obligation = obligations
            .iter()
            .find(|item| item["kind"] == "migration" && item["observed"] == *row)
            .unwrap();
        let position = row["position"].as_u64().unwrap() as usize;
        assert_eq!(output["migration_order"][position], obligation["id"]);
    }
    assert_eq!(output["schema_parity"], "NOT_ESTABLISHED");
    assert_eq!(output["parent_C01"], "OPEN");
    assert_eq!(output["whole_build_source_binding"], "OPEN");
}
