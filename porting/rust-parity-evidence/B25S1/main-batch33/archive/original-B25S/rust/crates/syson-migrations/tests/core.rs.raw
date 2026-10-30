// SPDX-License-Identifier: EPL-2.0
use serde_json::json;
use syson_migrations::{Error, MAX_BYTES, MAX_DEPTH, MAX_VALUES, Removal, SYSML_URI, prepare};
const VIEW: &[Removal] = &[Removal {
    class: "ViewUsage",
    features: &["exposedElement"],
}];
#[test]
fn original_namespace_order_and_duplicate_first_key_value_are_not_sorted() {
    let data = r#", "content":[{"eClass":"sysml:ViewUsage","data":{"exposedElement":["old"]}}]}"#;
    let yes = format!(r#"{{"ns":{{"z":"{SYSML_URI}","a":"wrong"}}{data}"#);
    let no = format!(r#"{{"ns":{{"z":"wrong","a":"{SYSML_URI}"}}{data}"#);
    let good = prepare(&yes, "test", VIEW).unwrap();
    assert!(good.matched_resource);
    assert!(
        good.json["content"][0]["data"]
            .get("exposedElement")
            .is_none()
    );
    assert!(!prepare(&no, "test", VIEW).unwrap().matched_resource);
    let repeated = format!(r#"{{"ns":{{"z":"wrong","a":"wrong","z":"{SYSML_URI}"}}{data}"#);
    assert!(prepare(&repeated, "test", VIEW).unwrap().matched_resource);
}
#[test]
fn collects_object_children_not_parent_fields_or_untyped_array_items() {
    let input = json!({"ns":{"sysml":SYSML_URI},"content":[{"eClass":"sysml:ViewUsage",
      "exposedElement":"parent", "data":{"exposedElement":["old"],"keep":1},
      "vendor":{"exposedElement":"child"},"array":[{"exposedElement":"untyped"},
      {"eClass":"sysml:ViewUsage","data":{"exposedElement":"nested"}}]}]});
    let p = prepare(&input.to_string(), "v", VIEW).unwrap();
    assert_eq!(p.json["content"][0]["exposedElement"], "parent");
    assert_eq!(p.json["content"][0]["data"], json!({"keep":1}));
    assert_eq!(p.json["content"][0]["vendor"], json!({}));
    assert_eq!(
        p.json["content"][0]["array"][0]["exposedElement"],
        "untyped"
    );
    assert_eq!(p.json["content"][0]["array"][1]["data"], json!({}));
    assert_eq!(
        prepare(&p.json.to_string(), "v", VIEW).unwrap().json,
        p.json
    );
}
#[test]
fn pass_collection_is_a_snapshot_and_later_pass_sees_current_output() {
    let input = json!({"ns":{"s":SYSML_URI},"eClass":"sysml:ViewUsage", "data":{
      "eClass":"sysml:ViewUsage", "exposedElement":{"eClass":"sysml:ViewUsage","data":{"exposedElement":"gone"}},"keep":4}});
    let p = prepare(&input.to_string(), "v", VIEW).unwrap();
    assert!(p.json["data"].get("exposedElement").is_none());
    assert_eq!(p.json["data"]["keep"], 4);
}
#[test]
fn subtype_predicate_and_opaque_values_preserve_exact_selected_behavior() {
    let rules = &[Removal {
        class: "PartUsage",
        features: &["x"],
    }];
    let input = r#"{"ns":{"s":"http://www.eclipse.org/syson/sysml"},"content":[{"eClass":"sysml:ViewUsage","data":{"x":null,"n":123456789012345678901234567890}},{"eClass":"other:ViewUsage","data":{"x":1}},{"eClass":"sysml:VisibilityKind","data":{"x":2}}]}"#;
    let p = prepare(input, "v", rules).unwrap();
    assert!(p.json["content"][0]["data"].get("x").is_none());
    assert_eq!(
        p.json["content"][0]["data"]["n"].to_string(),
        "123456789012345678901234567890"
    );
    assert_eq!(p.json["content"][1]["data"]["x"], 1);
    assert_eq!(p.json["content"][2]["data"]["x"], 2);
}
#[test]
fn malformed_namespace_root_and_budget_failures_have_no_partial_result() {
    for input in ["{", "[]"] {
        assert!(matches!(prepare(input, "v", VIEW), Err(Error::Input(_))));
    }
    for ns in ["null", "[]", "5", "true", "\"x\"", "{}"] {
        assert!(
            !prepare(&format!("{{\"ns\":{ns}}}"), "v", VIEW)
                .unwrap()
                .matched_resource
        );
    }
    for ns in ["null", "{}", "[]"] {
        assert_eq!(
            prepare(&format!("{{\"ns\":{{\"s\":{ns}}}}}"), "v", VIEW),
            Err(Error::NamespaceShape)
        );
    }
    assert_eq!(
        prepare(&" ".repeat(MAX_BYTES + 1), "v", VIEW),
        Err(Error::Limit)
    );
    let deep = format!(
        "{{\"v\":{}0{}}}",
        "[".repeat(MAX_DEPTH),
        "]".repeat(MAX_DEPTH)
    );
    assert_eq!(prepare(&deep, "v", VIEW), Err(Error::Limit));
    let many = json!({"v":vec![0;MAX_VALUES]});
    assert_eq!(prepare(&many.to_string(), "v", VIEW), Err(Error::Limit));
}

#[test]
fn long_ancestor_key_many_targets_keep_values_without_path_key_copies() {
    let mut typed = serde_json::Map::new();
    typed.insert("eClass".into(), json!("sysml:Annotation"));
    for n in 0..256 {
        typed.insert(
            format!("slot{n}"),
            json!({"annotatingElement":"old","keep":n}),
        );
    }
    let mut wrapper = serde_json::Map::new();
    let key = "k".repeat(128 * 1024);
    wrapper.insert(key.clone(), serde_json::Value::Object(typed));
    let input = json!({"ns":{"s":SYSML_URI},"content":wrapper}).to_string();
    let p = prepare(
        &input,
        "v",
        &[Removal {
            class: "Annotation",
            features: &["annotatingElement"],
        }],
    )
    .unwrap();
    for n in 0..256 {
        assert!(
            p.json["content"][&key][format!("slot{n}")]
                .get("annotatingElement")
                .is_none()
        );
        assert_eq!(p.json["content"][&key][format!("slot{n}")]["keep"], n);
    }
}

#[test]
fn serialization_preserves_predicate_across_participants_in_both_orders() {
    for (first, second, matched) in [(SYSML_URI, "foreign", true), ("foreign", SYSML_URI, false)] {
        let input = format!(
            r#"{{"ns":{{"z":"{first}","a":"{second}"}},"content":[{{"eClass":"sysml:ViewUsage","data":{{"x":1,"exposedElement":2}}}}]}}"#
        );
        let first = prepare(
            &input,
            "one",
            &[Removal {
                class: "ViewUsage",
                features: &["x"],
            }],
        )
        .unwrap();
        let bytes = first.to_json().unwrap();
        let second = prepare(&bytes, "two", VIEW).unwrap();
        assert_eq!(second.matched_resource, matched);
        assert_eq!(
            second.json["content"][0]["data"]
                .get("exposedElement")
                .is_none(),
            matched
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&bytes).unwrap(),
            first.json
        );
    }
}
#[test]
fn namespace_serialization_uses_current_values_and_remaining_insertion_order() {
    let input = format!(
        r#"{{"ns":{{"exposedElement":"{SYSML_URI}","z":"{SYSML_URI}","a":"foreign"}},"eClass":"sysml:ViewUsage"}}"#
    );
    let first = prepare(&input, "one", VIEW).unwrap();
    assert!(first.json["ns"].get("exposedElement").is_none());
    let bytes = first.to_json().unwrap();
    assert!(bytes.contains(&format!("\"ns\":{{\"z\":\"{SYSML_URI}\"")));
    assert!(prepare(&bytes, "two", VIEW).unwrap().matched_resource);
    let input = format!(r#"{{"ns":{{"z":"wrong","a":"foreign","z":"{SYSML_URI}"}}}}"#);
    let first = prepare(&input, "one", VIEW).unwrap();
    let bytes = first.to_json().unwrap();
    assert!(prepare(&bytes, "two", VIEW).unwrap().matched_resource);
    let mut edited = first;
    edited.json["added"] = json!("x".repeat(MAX_BYTES));
    assert_eq!(edited.to_json(), Err(Error::Limit));
}

#[test]
fn edited_output_rechecks_depth_and_value_budget_before_serializing() {
    let mut p = prepare("{}", "v", VIEW).unwrap();
    let mut deep = json!(0);
    for _ in 0..MAX_DEPTH + 1 {
        deep = json!([deep]);
    }
    p.json["deep"] = deep;
    assert_eq!(p.to_json(), Err(Error::Limit));
    let mut p = prepare("{}", "v", VIEW).unwrap();
    p.json["many"] = json!(vec![0; MAX_VALUES]);
    assert_eq!(p.to_json(), Err(Error::Limit));
    let input = format!("{{\"ns\":{{\"s\":\"{SYSML_URI}\"}}}}");
    assert!(matches!(
        prepare(
            &input,
            "v",
            &[Removal {
                class: "VisibilityKind",
                features: &["x"]
            }]
        ),
        Err(Error::UnknownClass(_))
    ));
}
