// SPDX-License-Identifier: EPL-2.0
//! Evidence-only: four distinct read-only consumers linked into one test process.
use serde_json::{Map, Value};
use syson_native_db_contract as db;
use syson_native_graphql_contract as graphql;
use syson_native_http_contract as http;
use syson_native_project_templates::Catalogue;
const GRAPHQL: &str = include_str!(
    "../../../../../rust/tools/native-graphql-contract/tests/fixtures/native-official-label-introspection.json"
);
const BAD_FAITH: &str = include_str!(
    "../../../../../rust/tools/native-graphql-contract/tests/fixtures/native-badfaith-introspection.json"
);
const DATABASE: &str = include_str!(
    "../../../../../rust/tools/native-db-contract/tests/fixtures/native-01-catalog.json"
);
const DB_ORACLE: &str = include_str!("../../../C01d/c01d-cli-02/native-stdout.txt");
const MAPPINGS: &str = include_str!(
    "../../../../../rust/tools/native-http-contract/tests/fixtures/native-mappings.json"
);
const PROBES: &str = include_str!(
    "../../../../../rust/tools/native-http-contract/tests/fixtures/native-probes.json"
);
const TEMPLATES: &str = include_str!(
    "../../../../../rust/tools/native-project-templates/tests/fixtures/native-templates.json"
);
fn ordinary(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap()
}
fn encode(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}
fn inject(raw: &str, member: &str) -> String {
    format!("{{{member},{}", raw.strip_prefix('{').unwrap())
}
fn marker(key: &str, value: &str) -> Value {
    let mut map = Map::new();
    map.insert(key.into(), Value::String(value.into()));
    Value::Object(map)
}
fn numeric(value: &Value, expected: &str) {
    let Value::Number(number) = value else {
        panic!("literal number required")
    };
    assert_eq!(number.as_str(), expected);
}
fn opaque_outputs(member: &str) -> Vec<Value> {
    let needle = r#""__schema":{"#;
    assert_eq!(GRAPHQL.matches(needle).count(), 1);
    let graph = GRAPHQL.replacen(needle, &(needle.to_owned() + member + ","), 1);
    let templates = inject(TEMPLATES, member);
    vec![
        graphql::inspect(&graph).unwrap()["schema"].clone(),
        http::inspect_mappings(&inject(MAPPINGS, member)).unwrap()["mappings"].clone(),
        http::inspect_probes(&inject(PROBES, member)).unwrap()["capture"].clone(),
        Catalogue::from_capture(templates.as_bytes())
            .unwrap()
            .projection(None)
            .unwrap()["capture"]
            .clone(),
    ]
}
fn row<'a>(value: &'a mut Value, id: &str) -> &'a mut Value {
    value["observations"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == id)
        .unwrap()
}
fn observed<'a>(value: &'a Value, id: &str) -> &'a Value {
    value["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .unwrap()
}
fn substitute(value: &Value, token: &str) -> Vec<u8> {
    let text = String::from_utf8(encode(value)).unwrap();
    assert_eq!(text.matches("\"RR23_TOKEN\"").count(), 1);
    text.replace("\"RR23_TOKEN\"", token).into_bytes()
}
#[test]
fn all_four_actual_capture_projections_match_their_original_contracts() {
    assert_eq!(
        graphql::inspect(GRAPHQL).unwrap()["schema"],
        ordinary(GRAPHQL)["data"]["__schema"]
    );
    let database = db::project(DATABASE.as_bytes()).unwrap();
    assert_eq!(database, ordinary(DB_ORACLE));
    assert_eq!(database["obligations"].as_array().unwrap().len(), 164);
    assert_eq!(database["migration_order"].as_array().unwrap().len(), 20);
    assert_eq!(
        http::inspect_mappings(MAPPINGS).unwrap()["mappings"],
        ordinary(MAPPINGS)
    );
    assert_eq!(
        http::inspect_probes(PROBES).unwrap()["capture"],
        ordinary(PROBES)
    );
    assert_eq!(
        Catalogue::from_capture(TEMPLATES.as_bytes())
            .unwrap()
            .envelope(),
        &ordinary(TEMPLATES)
    );
}
#[test]
fn opaque_slots_keep_manual_marker_objects_strings_and_numeric_tokens() {
    let member = r#""vendor":{"number":{"$serde_json::private::Number":"123"},"raw":{"$serde_json::private::RawValue":"{\"trap\":[true]}"},"text":"雪😀\n\u0041"}"#;
    for output in opaque_outputs(member) {
        assert_eq!(
            output["vendor"]["number"],
            marker("$serde_json::private::Number", "123")
        );
        assert_eq!(
            output["vendor"]["raw"],
            marker("$serde_json::private::RawValue", r#"{"trap":[true]}"#)
        );
        assert_eq!(output["vendor"]["text"].as_str(), Some("雪😀\nA"));
    }
    let huge = "9".repeat(4096);
    for token in ["-0", "0.00", "1E+000", "1e-99999", "1e99999", huge.as_str()] {
        for output in opaque_outputs(&(r#""vendor":"#.to_owned() + token)) {
            numeric(&output["vendor"], token);
        }
    }
}
#[test]
fn database_typed_refusals_and_literal_sql_strings_are_distinct_from_opaque_slots() {
    for raw in [
        marker("$serde_json::private::Number", "123"),
        marker("$serde_json::private::RawValue", "123"),
    ] {
        let mut input = ordinary(DATABASE);
        input["tables"][0]["oid"] = raw;
        let before = encode(&input);
        assert!(db::project(&before).is_err());
        assert_eq!(encode(&input), before);
    }
    let text = r#"雪😀 SQL '{"$serde_json::private::Number":"123","$serde_json::private::RawValue":"null"}'::text"#;
    let mut input = ordinary(DATABASE);
    input["columns"][0]["default_expr"] = Value::String(text.into());
    let output = db::project(&encode(&input)).unwrap();
    assert!(output["obligations"].as_array().unwrap().iter().any(
        |row| row["kind"] == "column" && row["observed"]["default_expr"].as_str() == Some(text)
    ));
    for token in ["-0", "0.0", "1e0", "9223372036854775808"] {
        let mut input = ordinary(DATABASE);
        input["changelog"][0]["orderexecuted"] = Value::String("RR23_TOKEN".into());
        assert!(db::project(&substitute(&input, token)).is_err());
    }
}
#[test]
fn typed_zero_policies_remain_reader_specific_and_raw_tokens_survive() {
    let source = ordinary(PROBES).to_string();
    assert!(source.contains(r#""body_bytes":0"#));
    let input = source.replacen(r#""body_bytes":0"#, r#""body_bytes":-0"#, 1);
    let output = http::inspect_probes(&input).unwrap();
    assert!(
        output["capture"]["observations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["response"]["body_bytes"]
                .as_number()
                .is_some_and(|n| n.as_str() == "-0"))
    );
    assert!(source.contains(r#""status":200"#));
    assert!(
        http::inspect_probes(&source.replacen(r#""status":200"#, r#""status":-0"#, 1)).is_err()
    );
    for token in [
        "-0",
        "0.0",
        "0e0",
        r#""-0""#,
        r#"{"$serde_json::private::Number":"0"}"#,
    ] {
        let mut input = ordinary(TEMPLATES);
        row(&mut input, "modal-large")["request"]["variables"]["page"] =
            Value::String("RR23_TOKEN".into());
        let raw = substitute(&input, token);
        let result = Catalogue::from_capture(&raw);
        if token == "-0" {
            let catalogue = result.unwrap();
            numeric(
                &observed(catalogue.envelope(), "modal-large")["request"]["variables"]["page"],
                "-0",
            );
        } else {
            assert!(result.is_err());
        }
    }
    let mut input = ordinary(TEMPLATES);
    let row = row(&mut input, "modal-large");
    let mut body = ordinary(row["body_utf8"].as_str().unwrap());
    body["data"]["viewer"]["projectTemplates"]["pageInfo"]["count"] =
        Value::String("RR23_TOKEN".into());
    let body = String::from_utf8(substitute(&body, "-0")).unwrap();
    row["body_utf8"] = Value::String(body.clone());
    let catalogue = Catalogue::from_capture(&encode(&input)).unwrap();
    assert_eq!(
        observed(catalogue.envelope(), "modal-large")["body_utf8"].as_str(),
        Some(body.as_str())
    );
}
#[test]
fn reader_last_wins_does_not_change_the_shared_strict_default() {
    for output in opaque_outputs(r#""vendor":1,"\u0076endor":2"#) {
        numeric(&output["vendor"], "2");
    }
    let duplicate = inject(DATABASE, r#""\u0073chema_version":999"#);
    assert_eq!(
        db::project(duplicate.as_bytes()).unwrap(),
        db::project(DATABASE.as_bytes()).unwrap()
    );
    assert!(matches!(
        syson_json_value::parse(
            r#"{"x":1,"\u0078":2}"#,
            syson_json_value::Limits::default(),
            || false
        ),
        Err(syson_json_value::Error::DuplicateKey)
    ));
}
#[test]
fn errors_frontend_shells_and_empty_template_pages_are_not_normalized() {
    assert!(
        graphql::inspect(BAD_FAITH)
            .unwrap_err()
            .contains("GraphQL errors")
    );
    let probes = http::inspect_probes(PROBES).unwrap();
    let rows = probes["capture"]["observations"].as_array().unwrap();
    let shell =
        rows.iter().find(|row| row["id"] == "graphql-get").unwrap()["response"]["body_utf8"]
            .as_str()
            .unwrap();
    assert_eq!(shell.len(), 504);
    assert_eq!(
        rows.iter()
            .filter(|row| row["response"]["body_utf8"].as_str() == Some(shell))
            .count(),
        9
    );
    let catalogue = Catalogue::from_capture(TEMPLATES.as_bytes()).unwrap();
    let projection = catalogue.projection(None).unwrap();
    assert_eq!(projection["observations"].as_array().unwrap().len(), 10);
    for id in [
        "browser-small-first",
        "browser-small-next",
        "invalid-context",
        "invalid-page-type",
    ] {
        assert_eq!(observed(&projection, id)["outcome"], "GRAPHQL_ERROR");
        assert!(catalogue.lookup(id, "blank-project").is_err());
    }
    assert_eq!(
        observed(&projection, "modal-small-next")["choices"],
        Value::Array(vec![])
    );
    assert_eq!(
        catalogue
            .projection(Some(("browser-large", "blank-project")))
            .unwrap()["selection"]["kind"],
        "template"
    );
    assert_eq!(
        catalogue
            .projection(Some(("browser-large", "upload-project")))
            .unwrap()["selection"]["kind"],
        "upload"
    );
}
