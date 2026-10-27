// SPDX-License-Identifier: EPL-2.0
use syson_native_library_catalog::{Catalog, LibrarySource, NodeId, SemanticId};

const URI: &str = "sysmllibrary:///00000000-0000-4000-8000-000000000001";
const NODE: &str = "00000000-0000-4000-8000-000000000002";
const SEMANTIC: &str = "00000000-0000-4000-8000-000000000003";
fn source(raw: &str) -> LibrarySource<'_> {
    LibrarySource {
        classpath_path: "sysml.libraries/Literal.json",
        resource_uri: URI,
        json: raw,
    }
}
#[test]
fn encoded_node_in_literal_rawvalue_string_never_becomes_a_physical_node() {
    let text = format!(
        r#"{{"id":"{NODE}","eClass":"sysml:Namespace","data":{{"elementId":"{SEMANTIC}"}}}}"#
    );
    // Independent oracle: content is empty and the only node-shaped text is a
    // String. String serialization cannot invoke generic Value materialization.
    let encoded = serde_json::to_string(&text).unwrap();
    let raw = format!(
        r#"{{"ns":{{"sysml":"http://www.eclipse.org/syson/sysml"}},"content":[],"vendor":{{"$serde_json::private::RawValue":{encoded}}}}}"#
    );
    let catalog = Catalog::from_resources(&[source(&raw)]).unwrap();
    assert_eq!(catalog.resource(URI).unwrap().json(), raw);
    assert_eq!(catalog.resource(URI).unwrap().nodes().len(), 0);
    assert!(catalog.by_node_id(URI, NodeId(NODE)).is_none());
    assert!(catalog.by_semantic_id(URI, SemanticId(SEMANTIC)).is_none());
}

fn node_text() -> String {
    format!(
        r#"{{"id":"{NODE}","eClass":"sysml:Namespace","data":{{"elementId":"{SEMANTIC}","declaredName":"轮🚗"}}}}"#
    )
}
fn document(content: &str, vendor: &str) -> String {
    format!(
        r#" {{"ns":{{"sysml":"http://www.eclipse.org/syson/sysml"}},"content":[{content}],"vendor":{vendor}}} 
"#
    )
}
#[test]
fn both_reserved_keys_remain_literal_objects_in_every_vendor_member_position() {
    for marker in [
        "$serde_json::private::RawValue",
        "$serde_json::private::Number",
    ] {
        let node = node_text();
        for vendor in [
            format!(r#"{{"{marker}":{node},"other":false}}"#),
            format!(r#"{{"other":false,"{marker}":{node}}}"#),
        ] {
            // Exactly one real object has eClass. Its parent key has no special
            // meaning; this expectation never uses a generic Value parser.
            let raw = document("", &vendor);
            let catalog = Catalog::from_resources(&[source(&raw)]).unwrap();
            let resource = catalog.resource(URI).unwrap();
            assert_eq!(resource.json().as_bytes(), raw.as_bytes());
            assert_eq!(resource.nodes().len(), 1);
            let by_node = catalog.by_node_id(URI, NodeId(NODE)).unwrap();
            let by_semantic = catalog.by_semantic_id(URI, SemanticId(SEMANTIC)).unwrap();
            assert!(std::ptr::eq(by_node, by_semantic));
            assert_eq!(by_node.pointer, format!("/vendor/{marker}"));
            assert_eq!(by_node.declared_name.as_deref(), Some("轮🚗"));
            assert!(catalog.by_node_id(URI, NodeId(SEMANTIC)).is_none());
            assert!(catalog.by_semantic_id(URI, SemanticId(NODE)).is_none());
        }
    }
}
#[test]
fn marker_strings_cannot_duplicate_or_shadow_a_real_physical_identity() {
    let node = node_text();
    let encoded = serde_json::to_string(&node).unwrap();
    let vendor = format!(
        r#"{{"$serde_json::private::RawValue":{encoded},"$serde_json::private::Number":"-0"}}"#
    );
    let raw = document(&node, &vendor);
    let catalog = Catalog::from_resources(&[source(&raw)]).unwrap();
    assert_eq!(catalog.resource(URI).unwrap().json(), raw);
    assert_eq!(catalog.resource(URI).unwrap().nodes().len(), 1);
    assert_eq!(
        catalog.by_node_id(URI, NodeId(NODE)).unwrap().pointer,
        "/content/0"
    );
}
#[test]
fn opaque_number_lexemes_and_escapes_keep_the_original_wire_carrier() {
    let vendor = r#"{"$serde_json::private::Number":"not a number","$serde_json::private::RawValue":"{\"eClass\":\"fake\"}","values":[-0,1e+00,1E-999999,900719925474099312345,"\u00e9🚗"]}"#;
    let raw = document("", vendor);
    let catalog = Catalog::from_resources(&[source(&raw)]).unwrap();
    let resource = catalog.resource(URI).unwrap();
    assert_eq!(resource.json().as_bytes(), raw.as_bytes());
    assert!(resource.nodes().is_empty());
}
#[test]
fn escaped_duplicate_marker_keys_and_malformed_strings_refuse_atomically() {
    let good = document("", "{}");
    let previous = Catalog::from_resources(&[source(&good)]).unwrap();
    for vendor in [
        r#"{"$serde_json::private::Number":"1","\u0024serde_json::private::Number":"2"}"#,
        r#"{"$serde_json::private::RawValue":{},"$serde_json::private::RawValue":false}"#,
        r#"{"x":"\uD800"}"#,
        r#"{"x":01}"#,
    ] {
        let raw = document("", vendor);
        let result = Catalog::from_resources(&[source(&raw)]);
        assert!(result.is_err());
        assert_eq!(previous.resource(URI).unwrap().json(), good);
        assert!(previous.resource(URI).unwrap().nodes().is_empty());
    }
}
#[test]
fn cancellation_during_scan_and_after_staging_returns_no_partial_catalogue() {
    let raw = document(&node_text(), &format!("\"{}\"", "x".repeat(50_000)));
    let inputs = [source(&raw)];
    let previous = Catalog::from_resources(&inputs).unwrap();
    let before = raw.clone();
    let mut calls = 0;
    let result = Catalog::from_resources_with_cancel(&inputs, || {
        calls += 1;
        calls == 5
    });
    assert!(matches!(result, Err(e) if e == "catalogue cancelled"));
    let mut total = 0;
    Catalog::from_resources_with_cancel(&inputs, || {
        total += 1;
        false
    })
    .unwrap();
    let mut current = 0;
    let result = Catalog::from_resources_with_cancel(&inputs, || {
        current += 1;
        current == total
    });
    assert!(matches!(result, Err(e) if e == "catalogue cancelled"));
    assert_eq!(current, total);
    assert_eq!(raw, before);
    assert_eq!(previous.resource(URI).unwrap().nodes().len(), 1);
    assert_eq!(previous.resource(URI).unwrap().json(), raw);
}
fn many_values(values: usize) -> String {
    format!(
        r#"{{"ns":{{"sysml":"http://www.eclipse.org/syson/sysml"}},"content":[],"opaque":[{}null]}}"#,
        "null,".repeat(values - 6)
    )
}
#[test]
fn aggregate_literal_value_budget_is_exact_across_resources_before_late_materialization() {
    use syson_native_library_catalog::MAX_VALUES;
    // Each document has five structural/string values and the requested count
    // includes those five. Each per-file payload remains below 4MiB.
    let first = many_values(700_000);
    let second = many_values(700_000);
    let remaining = MAX_VALUES - 1_400_000;
    let third = many_values(remaining);
    let paths = [
        "sysml.libraries/First.json",
        "sysml.libraries/Second.json",
        "sysml.libraries/Third.json",
    ];
    let uris = [
        "sysmllibrary:///00000000-0000-4000-8000-000000000011",
        "sysmllibrary:///00000000-0000-4000-8000-000000000012",
        "sysmllibrary:///00000000-0000-4000-8000-000000000013",
    ];
    let build = |last: &str| {
        let inputs: Vec<_> = [first.as_str(), second.as_str(), last]
            .into_iter()
            .enumerate()
            .map(|(i, json)| LibrarySource {
                classpath_path: paths[i],
                resource_uri: uris[i],
                json,
            })
            .collect();
        Catalog::from_resources(&inputs)
    };
    let exact = build(&third).unwrap();
    assert_eq!(exact.resources().len(), 3);
    assert!(exact.resources().iter().all(|r| r.nodes().is_empty()));
    let oversized = many_values(remaining + 1);
    assert!(matches!(build(&oversized), Err(e) if e == "JSON value limit"));
    assert_eq!(exact.resources()[2].json(), third);
}
#[test]
fn catalogue_depth208_and_exact_value_depth256_keep_literal_markers_opaque() {
    for depth in [208, 256] {
        // Root object depth1; vendor arrays contribute depth-2; String leaf
        // adds the final level. Node-shaped text remains a string at any depth.
        let encoded = serde_json::to_string(&node_text()).unwrap();
        let vendor = format!(
            "{}{}{}",
            "[".repeat(depth - 2),
            encoded,
            "]".repeat(depth - 2)
        );
        let raw = document("", &vendor);
        let catalog = Catalog::from_resources(&[source(&raw)]).unwrap();
        assert!(catalog.resource(URI).unwrap().nodes().is_empty());
        assert_eq!(catalog.resource(URI).unwrap().json(), raw);
    }
    let raw = document("", &("[".repeat(255) + "null" + &"]".repeat(255)));
    assert!(matches!(Catalog::from_resources(&[source(&raw)]), Err(e) if e == "JSON depth limit"));
}
