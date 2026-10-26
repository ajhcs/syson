// SPDX-License-Identifier: EPL-2.0
use syson_exact_json_diff::{MAX_CHILDREN, MAX_DEPTH, MAX_DIFFS, MAX_INPUT, MAX_POINTER, compare};
const ACTUAL: &[u8] = include_bytes!("fixtures/native-mappings.json");
fn diff(left: &str, right: &str) -> syson_exact_json_diff::Report {
    compare(left.as_bytes(), right.as_bytes()).unwrap()
}
#[test]
fn actual_native_capture_self_comparison_is_complete_observation_only() {
    let report = compare(ACTUAL, ACTUAL).unwrap();
    assert!(report.is_match() && report.is_complete() && report.differences().is_empty());
    let output = report.into_json();
    assert_eq!(output["status"], "MATCH");
    assert_eq!(output["left_bytes"], 24770);
    assert!(
        output["scope"]
            .as_str()
            .unwrap()
            .contains("no identity attestation")
    );
    // Mutate the actual captured class identity at its real full-document path.
    let left = std::str::from_utf8(ACTUAL).unwrap();
    let right = left.replacen(
        "GraphQLController#performRequest",
        "OtherController#performRequest",
        1,
    );
    assert_ne!(left, right);
    let report = diff(left, &right);
    assert!(!report.is_match() && report.is_complete());
    assert_eq!(report.differences().len(), 1);
    assert!(
        report.differences()[0]
            .left_pointer
            .starts_with("/contexts/application/mappings/dispatcherServlets/dispatcherServlet/")
    );
    assert!(report.differences()[0].left_pointer.ends_with("/handler"));
}
#[test]
fn object_order_array_order_and_identity_strings_are_never_sorted_or_mapped() {
    let report = diff(r#"{"a":1,"b":2}"#, r#"{"b":2,"a":1}"#);
    assert!(!report.is_match() && report.is_complete());
    assert_eq!(report.differences()[0].kind, "OBJECT_MEMBER_ORDER_OR_KEY");
    assert_eq!(report.differences()[0].left_pointer, "/a");
    assert_eq!(report.differences()[0].right_pointer, "/b");
    let report = diff(
        r#"[{"id":"node-A"},{"id":"node-B"}]"#,
        r#"[{"id":"node-B"},{"id":"node-A"}]"#,
    );
    assert_eq!(report.differences().len(), 2);
    assert_eq!(report.differences()[0].left_pointer, "/0/id");
    assert_eq!(report.differences()[1].right_pointer, "/1/id");
    assert!(!report.is_match());
}
#[test]
fn null_missing_type_and_jsonpointer_escapes_remain_distinct() {
    let report = diff(r#"{"x":null}"#, r#"{}"#);
    assert_eq!(report.differences()[0].kind, "MISSING_RIGHT");
    assert_eq!(report.differences()[0].left_pointer, "/x");
    assert_eq!(report.differences()[0].left_preview, "null");
    assert_eq!(report.differences()[0].right_preview, "<absent>");
    assert_eq!(diff("null", "\"null\"").differences()[0].kind, "TYPE");
    let report = diff(r#"{"a/b":{"~key":[true]}}"#, r#"{"a/b":{"~key":[false]}}"#);
    assert_eq!(report.differences()[0].left_pointer, "/a~1b/~0key/0");
    assert_eq!(report.differences()[0].kind, "BOOLEAN");
    assert_eq!(diff("[]", "[null]").differences()[0].kind, "MISSING_LEFT");
}
#[test]
fn exact_numeric_lexemes_cover_large_integers_decimals_exponents_and_negative_zero() {
    for (left, right) in [
        ("900719925474099312345", "900719925474099312346"),
        ("1.5184049843810499", "1.51840498438105"),
        ("1", "1.0"),
        ("1e2", "100"),
        ("1e+2", "1E2"),
        ("-0", "0"),
    ] {
        assert!(diff(left, left).is_match());
        let report = diff(left, right);
        assert!(!report.is_match() && report.is_complete());
        assert_eq!(report.differences().len(), 1);
        assert_eq!(report.differences()[0].kind, "NUMBER_LEXEME");
        assert_eq!(report.differences()[0].left_preview, left);
        assert_eq!(report.differences()[0].right_preview, right);
    }
}
#[test]
fn only_insignificant_whitespace_and_string_escape_spelling_are_excluded() {
    assert!(
        diff(
            " \n {\"a\": \"é\",\"b\":1} \t",
            r#"{"\u0061":"\u00e9","b":1}"#
        )
        .is_match()
    );
    assert!(!diff(r#""é""#, r#""e\u0301""#).is_match());
    assert!(!diff(r#"{"id":"A"}"#, r#"{"id":"a"}"#).is_match());
    assert!(!diff("[1,2]", "[2,1]").is_match());
}
#[test]
fn duplicate_keys_invalid_and_late_malformed_inputs_refuse_before_comparison() {
    for bad in [
        r#"{"a":1,"a":2}"#,
        r#"{"a":null,"\u0061":null}"#,
        r#"{"x":{"id":1,"id":1}}"#,
        "{",
        "true false",
        "[1,]",
        "01",
        r#""\ud800""#,
    ] {
        assert!(compare(bad.as_bytes(), b"null").is_err(), "{bad}");
        assert!(compare(b"null", bad.as_bytes()).is_err(), "{bad}");
    }
    assert!(compare(&[0xff], b"null").is_err());
    let many = vec!["true"; MAX_DIFFS + 1].join(",");
    let bad = format!("[{many},{{\"duplicate\":1,\"duplicate\":2}}]");
    assert!(compare(b"[]", bad.as_bytes()).is_err());
}
#[test]
fn diff_limit_never_hides_mismatch_and_previews_are_utf8_safe() {
    let list = |v: &str, n| format!("[{}]", vec![v; n].join(","));
    let report = diff(&list("0", MAX_DIFFS), &list("1", MAX_DIFFS));
    assert_eq!(report.differences().len(), MAX_DIFFS);
    assert!(report.is_complete() && !report.is_match());
    let report = diff(&list("0", MAX_DIFFS + 1), &list("1", MAX_DIFFS + 1));
    assert_eq!(report.differences().len(), MAX_DIFFS);
    assert!(!report.is_complete() && !report.is_match());
    assert_eq!(report.into_json()["status"], "MISMATCH");
    let long = format!("\"{}\"", "🚗".repeat(1000));
    let report = diff(&long, "\"other\"");
    assert!(report.differences()[0].left_preview.ends_with("…\""));
    assert!(report.differences()[0].left_preview.len() < 1024);
}
#[test]
fn input_depth_pointer_container_and_global_pending_value_limits_are_enforced() {
    assert!(compare(&vec![b' '; MAX_INPUT + 1], b"null").is_err());
    let nested = |n| format!("{}null{}", "[".repeat(n), "]".repeat(n));
    assert!(diff(&nested(MAX_DEPTH - 1), &nested(MAX_DEPTH - 1)).is_match());
    assert!(compare(nested(MAX_DEPTH).as_bytes(), b"null").is_err());
    let object = |n| format!("{{\"{}\":null}}", "x".repeat(n));
    assert!(diff(&object(MAX_POINTER - 1), &object(MAX_POINTER - 1)).is_match());
    assert!(compare(object(MAX_POINTER).as_bytes(), b"null").is_err());
    let list = |n| format!("[{}]", vec!["null"; n].join(","));
    assert!(diff(&list(MAX_CHILDREN), &list(MAX_CHILDREN)).is_match());
    assert!(compare(list(MAX_CHILDREN + 1).as_bytes(), b"null").is_err());
    let row = list(MAX_CHILDREN);
    let exact = format!("[{row},{row},{row},{row},{}]", list(19_994));
    assert!(diff(&exact, &exact).is_match()); // 100000 nodes including containers.
    let over = format!("[{row},{row},{row},{row},{}]", list(19_995));
    assert!(
        compare(over.as_bytes(), b"null")
            .unwrap_err()
            .contains("value limit")
    );
}
#[test]
fn worst_pointer_previews_are_bounded_without_relaxing_the_input_policy() {
    let key = "\\u0000".repeat(4000);
    let values = |n| format!("[{}]", vec![n; MAX_DIFFS + 1].join(","));
    let left = format!("{{\"{key}\":{}}}", values("0"));
    let right = format!("{{\"{key}\":{}}}", values("1"));
    let report = diff(&left, &right);
    assert!(!report.is_complete() && !report.is_match());
    let encoded = report.into_json().to_string();
    assert!(encoded.len() < 4 * 1024 * 1024);
}

#[test]
fn individually_admitted_paths_cannot_create_oversized_missing_counterpart_pointers() {
    let long = "x".repeat(4000);
    let left = format!("{{\"{long}\":{{}}}}");
    let right = format!("{{\"short\":{{\"{long}\":1}}}}");
    assert!(diff(&left, &left).is_match());
    assert!(diff(&right, &right).is_match());
    // Both actual paths fit, but /<long>/<long> on the missing side does not.
    for (a, b) in [(&left, &right), (&right, &left)] {
        assert!(
            compare(a.as_bytes(), b.as_bytes())
                .unwrap_err()
                .contains("comparison pointer limit")
        );
    }
    let edge = "x".repeat(MAX_POINTER - 1);
    let left = format!("{{\"{edge}\":[]}}");
    let right = "{\"short\":[0]}";
    assert!(diff(&left, &left).is_match());
    assert!(diff(right, right).is_match());
    // An absent array element also needs a bounded hypothetical /0 pointer.
    assert!(
        compare(left.as_bytes(), right.as_bytes())
            .unwrap_err()
            .contains("comparison pointer limit")
    );
}
