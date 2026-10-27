// SPDX-License-Identifier: EPL-2.0
//! Manual marker maps and raw numeric tokens; no parsed expected marker Value.
use serde_json::{Map, Value};
use syson_native_project_templates::Catalogue;
const NATIVE: &[u8] = include_bytes!("fixtures/native-templates.json");
fn input() -> Value {
    serde_json::from_slice(NATIVE).unwrap()
}
fn marker(key: &str, value: &str) -> Value {
    let mut map = Map::new();
    map.insert(key.into(), Value::String(value.into()));
    Value::Object(map)
}
fn bytes(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}
fn row<'a>(capture: &'a mut Value, id: &str) -> &'a mut Value {
    capture["observations"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == id)
        .unwrap()
}
fn observed<'a>(projection: &'a Value, id: &str) -> &'a Value {
    projection["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .unwrap()
}
fn body(capture: &mut Value, id: &str, edit: impl FnOnce(&mut Value)) {
    let row = row(capture, id);
    let mut body: Value = serde_json::from_str(row["body_utf8"].as_str().unwrap()).unwrap();
    edit(&mut body);
    row["body_utf8"] = Value::String(serde_json::to_string(&body).unwrap());
}
#[test]
fn outer_private_number_object_stays_an_ordinary_object() {
    let mut input = input();
    let expected = marker("$serde_json::private::Number", "123");
    input["context"]["vendor"] = expected.clone();
    let raw = bytes(&input);
    let catalogue = Catalogue::from_capture(&raw).unwrap();
    assert_eq!(catalogue.envelope()["context"]["vendor"], expected);
    assert_eq!(
        catalogue.projection(None).unwrap()["capture"]["context"]["vendor"],
        expected
    );
}
#[test]
fn outer_private_raw_object_keeps_its_literal_string() {
    let mut input = input();
    let expected = marker(
        "$serde_json::private::RawValue",
        r#"{"injected":[true],"label":"雪😀"}"#,
    );
    input["context"]["vendor"] = expected.clone();
    let catalogue = Catalogue::from_capture(&bytes(&input)).unwrap();
    assert_eq!(catalogue.envelope()["context"]["vendor"], expected);
    assert_eq!(
        catalogue.projection(None).unwrap()["capture"]["context"]["vendor"],
        expected
    );
}
#[test]
fn outer_projection_preserves_exact_numeric_token_spelling() {
    let huge = "9".repeat(16_384);
    for token in ["-0", "0.00", "1E+000", "1e-99999", "1e99999", huge.as_str()] {
        let mut input = input();
        input["context"]["vendor"] = Value::String("RR23D_TOKEN".into());
        let raw = String::from_utf8(bytes(&input))
            .unwrap()
            .replace("\"RR23D_TOKEN\"", token);
        let before = raw.clone();
        let output = Catalogue::from_capture(raw.as_bytes())
            .unwrap()
            .projection(None)
            .unwrap();
        let Value::Number(number) = &output["capture"]["context"]["vendor"] else {
            panic!("number required")
        };
        assert_eq!(number.to_string(), token);
        assert_eq!(raw, before);
    }
}
#[test]
fn nested_private_number_object_cannot_be_a_typed_page_count() {
    let mut input = input();
    body(&mut input, "modal-large", |body| {
        body["data"]["viewer"]["projectTemplates"]["pageInfo"]["count"] =
            marker("$serde_json::private::Number", "4");
    });
    assert!(Catalogue::from_capture(&bytes(&input)).is_err());
}
#[test]
fn nested_private_raw_object_cannot_fabricate_an_empty_errors_array() {
    let mut input = input();
    body(&mut input, "all-templates", |body| {
        body["errors"] = marker("$serde_json::private::RawValue", "[]");
    });
    assert!(Catalogue::from_capture(&bytes(&input)).is_err());
}
#[test]
fn original_page_count_negative_zero_acceptance_keeps_raw_body_carrier() {
    let mut input = input();
    body(&mut input, "modal-large", |body| {
        body["data"]["viewer"]["projectTemplates"]["pageInfo"]["count"] =
            Value::String("RR23D_COUNT".into());
    });
    let row = row(&mut input, "modal-large");
    let raw_body = row["body_utf8"]
        .as_str()
        .unwrap()
        .replace("\"RR23D_COUNT\"", "-0");
    row["body_utf8"] = Value::String(raw_body.clone());
    let catalogue = Catalogue::from_capture(&bytes(&input)).unwrap();
    let projection = catalogue.projection(None).unwrap();
    assert_eq!(observed(&projection, "modal-large")["outcome"], "SUCCESS");
    let saved = projection["capture"]["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "modal-large")
        .unwrap();
    assert_eq!(saved["body_utf8"].as_str(), Some(raw_body.as_str()));
}
#[test]
fn original_request_negative_zero_matches_canonical_page_and_limit_zero() {
    for (id, field) in [
        ("modal-large", "page"),
        ("negative-page-zero-limit", "limit"),
    ] {
        let mut input = input();
        row(&mut input, id)["request"]["variables"][field] =
            Value::String("RR23D_REQUEST_ZERO".into());
        let raw = String::from_utf8(bytes(&input))
            .unwrap()
            .replace("\"RR23D_REQUEST_ZERO\"", "-0");
        let catalogue = Catalogue::from_capture(raw.as_bytes()).unwrap();
        assert_eq!(
            observed(&catalogue.projection(None).unwrap(), id)["outcome"],
            "SUCCESS"
        );
    }
}

fn inject(raw: &str, members: &str) -> String {
    format!("{{{members},{}", raw.strip_prefix('{').unwrap())
}
fn set_body(capture: &mut Value, id: &str, raw: String) {
    row(capture, id)["body_utf8"] = Value::String(raw);
}
fn body_text<'a>(capture: &'a Value, id: &str) -> &'a str {
    capture["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .unwrap()["body_utf8"]
        .as_str()
        .unwrap()
}
#[test]
fn nested_unknown_markers_numbers_and_strings_keep_exact_body_in_success_and_errors() {
    let members = r#""vendor":{"$serde_json::private::Number":"123","$serde_json::private::RawValue":"{\"fake\":true}","n":-0,"huge":1e99999,"ordered":[false,null,"雪😀"]}"#;
    for (id, outcome, status) in [
        ("all-templates", "SUCCESS", 200),
        ("browser-small-first", "GRAPHQL_ERROR", 200),
        ("modal-large", "HTTP_ERROR", 503),
    ] {
        let mut input = input();
        let raw = format!(" \n{}\t", inject(body_text(&input, id), members));
        row(&mut input, id)["status"] = Value::from(status);
        set_body(&mut input, id, raw.clone());
        let catalogue = Catalogue::from_capture(&bytes(&input)).unwrap();
        let output = catalogue.projection(None).unwrap();
        assert_eq!(observed(&output, id)["outcome"], outcome);
        assert_eq!(body_text(&output["capture"], id), raw);
        if outcome != "SUCCESS" {
            assert!(observed(&output, id)["choices"].is_null());
            assert!(catalogue.lookup(id, "batmobile-template").is_err());
        }
    }
}
#[test]
fn marker_objects_cannot_replace_outer_or_nested_typed_slots() {
    const RAW: &str = "$serde_json::private::RawValue";
    for field in ["anonymous", "status", "request", "body", "header", "digest"] {
        let mut input = input();
        match field {
            "anonymous" => input["context"]["anonymous"] = marker(RAW, "true"),
            "status" => {
                row(&mut input, "all-templates")["status"] =
                    marker("$serde_json::private::Number", "200")
            }
            "request" => {
                let request = row(&mut input, "all-templates")["request"].to_string();
                row(&mut input, "all-templates")["request"] = marker(RAW, &request);
            }
            "body" => {
                let quoted = serde_json::to_string(body_text(&input, "all-templates")).unwrap();
                row(&mut input, "all-templates")["body_utf8"] = marker(RAW, &quoted);
            }
            "header" => {
                row(&mut input, "all-templates")["headers"][0] =
                    marker(RAW, r#"["x-header","value"]"#)
            }
            "digest" => {
                input["context"]["artifact_sha256"] =
                    marker(RAW, &serde_json::to_string(&"a".repeat(64)).unwrap())
            }
            _ => unreachable!(),
        }
        assert!(
            Catalogue::from_capture(&bytes(&input)).is_err(),
            "outer {field}"
        );
    }
    for field in [
        "id",
        "label",
        "imageURL",
        "requiredLibraries",
        "library-version",
        "root",
        "page-flag",
        "page-cursor",
    ] {
        let mut input = input();
        let id = if field.starts_with("page-") {
            "modal-large"
        } else {
            "all-templates"
        };
        body(&mut input, id, |body| {
            if field == "root" {
                *body = marker(RAW, &body.to_string());
            } else if field.starts_with("page-") {
                let page = &mut body["data"]["viewer"]["projectTemplates"]["pageInfo"];
                if field == "page-flag" {
                    page["hasNextPage"] = marker(RAW, "false");
                } else {
                    page["startCursor"] = marker(RAW, "null");
                }
            } else {
                let node = &mut body["data"]["viewer"]["allProjectTemplates"][0];
                match field {
                    "requiredLibraries" => node[field] = marker(RAW, "[]"),
                    "library-version" => {
                        node["requiredLibraries"] = serde_json::json!([{"id":"id","namespace":"ns","name":"name","version":"1"}]);
                        node["requiredLibraries"][0]["version"] = marker(RAW, "\"1\"");
                    }
                    _ => {
                        let value = serde_json::to_string(node[field].as_str().unwrap()).unwrap();
                        node[field] = marker(RAW, &value);
                    }
                }
            }
        });
        assert!(
            Catalogue::from_capture(&bytes(&input)).is_err(),
            "nested {field}"
        );
    }
}
#[test]
fn zero_compatibility_is_numeric_and_request_only_with_literal_values_untouched() {
    for (id, field) in [
        ("modal-large", "page"),
        ("negative-page-zero-limit", "limit"),
    ] {
        let mut input = input();
        row(&mut input, id)["request"]["variables"][field] = Value::String("ZERO_TOKEN".into());
        let raw = String::from_utf8(bytes(&input))
            .unwrap()
            .replace("\"ZERO_TOKEN\"", "-0");
        let catalogue = Catalogue::from_capture(raw.as_bytes()).unwrap();
        let captured = &catalogue.envelope()["observations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .unwrap()["request"]["variables"][field];
        let Value::Number(number) = captured else {
            panic!("literal number expected")
        };
        assert_eq!(number.to_string(), "-0");
        assert!(catalogue.projection(None).is_ok());
        for token in [
            "0.0",
            "0e0",
            "-0.0",
            "\"-0\"",
            r#"{"$serde_json::private::Number":"0"}"#,
        ] {
            let raw = String::from_utf8(bytes(&input))
                .unwrap()
                .replace("\"ZERO_TOKEN\"", token);
            assert!(
                Catalogue::from_capture(raw.as_bytes()).is_err(),
                "request {field}:{token}"
            );
        }
    }
    for token in ["0.0", "0e0", "-0.0", "-1", "2147483648", "1e99999"] {
        let mut input = input();
        body(&mut input, "modal-large", |b| {
            b["data"]["viewer"]["projectTemplates"]["pageInfo"]["count"] =
                Value::String("COUNT_TOKEN".into())
        });
        let raw = body_text(&input, "modal-large").replace("\"COUNT_TOKEN\"", token);
        set_body(&mut input, "modal-large", raw);
        assert!(
            Catalogue::from_capture(&bytes(&input)).is_err(),
            "count:{token}"
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
fn value_boundary(raw: &str, original_values: usize, extra: usize) -> String {
    use syson_native_project_templates::MAX_LITERAL_VALUES;
    // Original root remains; add an array, its zero scalars, and final null.
    let n = MAX_LITERAL_VALUES - original_values - 2 + extra;
    let discarded = vec!["0"; n].join(",");
    inject(raw, &format!(r#""vendor":[{discarded}],"vendor":null"#))
}
#[test]
fn physical_values_are_bounded_independently_for_outer_and_decoded_body_before_collapse() {
    use syson_native_project_templates::MAX_BYTES;
    let original = input();
    let ordinary_count = values(&original);
    let raw = String::from_utf8(bytes(&original)).unwrap();
    drop(original);
    for extra in [0, 1] {
        let capture = value_boundary(&raw, ordinary_count, extra);
        assert!(capture.len() < MAX_BYTES);
        let result = Catalogue::from_capture(capture.as_bytes());
        if extra == 0 {
            assert!(result.unwrap().envelope()["vendor"].is_null());
        } else {
            assert!(result.unwrap_err().contains("Values"));
        }
    }
    for extra in [0, 1] {
        let mut input = input();
        let raw = body_text(&input, "all-templates");
        let count = values(&serde_json::from_str::<Value>(raw).unwrap());
        let raw = value_boundary(raw, count, extra);
        set_body(&mut input, "all-templates", raw.clone());
        let encoded = bytes(&input);
        assert!(encoded.len() < MAX_BYTES);
        drop(input);
        let result = Catalogue::from_capture(&encoded);
        if extra == 0 {
            let catalogue = result.unwrap();
            assert_eq!(body_text(catalogue.envelope(), "all-templates"), raw);
        } else {
            let error = result.unwrap_err();
            assert!(
                error.contains("all-templates: JSON response") && error.contains("Values"),
                "{error}"
            );
        }
    }
}
#[test]
fn byte_depth_and_duplicate_edges_preserve_final_typed_catalogue() {
    use syson_native_project_templates::{MAX_BYTES, MAX_LITERAL_DEPTH};
    let original = input();
    let raw = String::from_utf8(bytes(&original)).unwrap();
    for extra in [0, 1] {
        let n = MAX_LITERAL_DEPTH - 2 + extra;
        let deep = inject(
            &raw,
            &format!("\"vendor\":{}0{}", "[".repeat(n), "]".repeat(n)),
        );
        let result = Catalogue::from_capture(deep.as_bytes());
        assert_eq!(result.is_ok(), extra == 0);
        if extra != 0 {
            assert!(result.unwrap_err().contains("Depth"));
        }
        let mut input = input();
        let deep = inject(
            body_text(&input, "all-templates"),
            &format!("\"vendor\":{}0{}", "[".repeat(n), "]".repeat(n)),
        );
        set_body(&mut input, "all-templates", deep.clone());
        let result = Catalogue::from_capture(&bytes(&input));
        if extra == 0 {
            assert_eq!(body_text(result.unwrap().envelope(), "all-templates"), deep);
        } else {
            let error = result.unwrap_err();
            assert!(error.contains("all-templates: JSON response") && error.contains("Depth"));
        }
    }
    let mut exact = raw.into_bytes();
    exact.resize(MAX_BYTES, b' ');
    assert!(Catalogue::from_capture(&exact).is_ok());
    exact.push(b' ');
    assert!(
        Catalogue::from_capture(&exact)
            .unwrap_err()
            .contains("capture byte limit")
    );
    let duplicate = inject(std::str::from_utf8(NATIVE).unwrap(), r#""context":null"#);
    assert_eq!(
        Catalogue::from_capture(duplicate.as_bytes())
            .unwrap()
            .envelope(),
        &original
    );
    let mut failed = original.clone();
    row(&mut failed, "negative-page-zero-limit")["body_utf8"] =
        Value::String("late bad JSON".into());
    let before = bytes(&failed);
    assert!(Catalogue::from_capture(&before).is_err());
    assert_eq!(bytes(&failed), before);
    assert_eq!(
        Catalogue::from_capture(NATIVE).unwrap().envelope(),
        &original
    );
}
#[test]
fn actual_capture_keeps_all_errors_choices_headers_raw_bodies_and_empty_pages() {
    let original = input();
    let catalogue = Catalogue::from_capture(NATIVE).unwrap();
    let output = catalogue.projection(None).unwrap();
    assert_eq!(output["capture"], original);
    assert_eq!(output["observations"].as_array().unwrap().len(), 10);
    for original in original["observations"].as_array().unwrap() {
        let id = original["id"].as_str().unwrap();
        assert_eq!(
            body_text(&output["capture"], id),
            original["body_utf8"].as_str().unwrap()
        );
    }
    for id in [
        "browser-small-first",
        "browser-small-next",
        "invalid-context",
        "invalid-page-type",
    ] {
        let row = observed(&output, id);
        assert_eq!(row["outcome"], "GRAPHQL_ERROR");
        assert!(row["choices"].is_null());
        assert!(catalogue.lookup(id, "batmobile-template").is_err());
    }
    assert_eq!(
        observed(&output, "modal-small-next")["choices"],
        Value::Array(vec![])
    );
    assert_eq!(
        catalogue
            .projection(Some(("browser-large", "upload-project")))
            .unwrap()["selection"]["kind"],
        "upload"
    );
    assert_eq!(
        catalogue
            .projection(Some(("browser-large", "blank-project")))
            .unwrap()["selection"]["kind"],
        "template"
    );
    for key in ["C01", "B23", "U03"] {
        assert_eq!(output[key], "OPEN");
    }
}
