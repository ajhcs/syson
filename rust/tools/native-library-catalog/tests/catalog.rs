// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use syson_native_library_catalog::{
    Catalog, LibrarySource, MAX_DEPTH, MAX_FILE, MAX_RESOURCES, NodeId, SemanticId, manifest,
};
const ATTR: &str = include_str!("fixtures/Attributes.json");
const SCALAR: &str = include_str!("fixtures/ScalarValues.json");
const NATIVE: &str = include_str!("fixtures/representative-native.json");
const URI: &str = "sysmllibrary:///facd22dc-b5db-3e27-8679-83afde904862";
const KURI: &str = "kermllibrary:///b2c6dd37-2084-3ce4-9ce2-580fdf30629c";
const A: &str = "00000000-0000-4000-8000-000000000001";
const B: &str = "00000000-0000-4000-8000-000000000002";
fn source(raw: &str) -> LibrarySource<'_> {
    LibrarySource {
        classpath_path: "sysml.libraries/Attributes.json",
        resource_uri: URI,
        json: raw,
    }
}
fn node(id: &str, semantic: &str) -> Value {
    json!({"id":id,"eClass":"sysml:Namespace","data":{"elementId":semantic}})
}
fn doc(nodes: Vec<Value>) -> String {
    json!({"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":nodes}).to_string()
}
fn refuses(raw: &str, why: &str) {
    match Catalog::from_resources(&[source(raw)]) {
        Err(e) => assert!(e.contains(why), "{e}"),
        Ok(_) => panic!("accepted {why}"),
    }
}
#[test]
fn genuine_resources_match_actual_native_identity_observations_and_raw_pointers() {
    let catalog = Catalog::from_resources(&[
        source(ATTR),
        LibrarySource {
            classpath_path: "kerml.libraries/ScalarValues.json",
            resource_uri: KURI,
            json: SCALAR,
        },
    ])
    .unwrap();
    let observed: Value = serde_json::from_str(NATIVE).unwrap();
    for actual in observed["observations"].as_array().unwrap() {
        let uri = actual["resource_uri"].as_str().unwrap();
        let resource = catalog.resource(uri).unwrap();
        assert_eq!(
            resource.nodes().len() as u64,
            actual["object_count"].as_u64().unwrap()
        );
        let parsed: Value = serde_json::from_str(resource.json()).unwrap();
        for n in resource.nodes() {
            let physical = parsed.pointer(&n.pointer).unwrap();
            assert_eq!(physical["id"], n.node_id);
            assert_eq!(physical["data"]["elementId"], n.semantic_id);
            assert_eq!(physical["eClass"], n.e_class);
        }
        for group in ["roots", "packages"] {
            for n in actual[group].as_array().unwrap() {
                let by_node = catalog
                    .by_node_id(uri, NodeId(n["node_id"].as_str().unwrap()))
                    .unwrap();
                let by_semantic = catalog
                    .by_semantic_id(uri, SemanticId(n["semantic_id"].as_str().unwrap()))
                    .unwrap();
                assert!(std::ptr::eq(by_node, by_semantic));
                assert_eq!(
                    by_node.e_class,
                    format!("sysml:{}", n["e_class"].as_str().unwrap())
                );
                assert_eq!(
                    by_node.declared_name.as_deref(),
                    n["declared_name"].as_str()
                );
            }
        }
        assert_eq!(actual["roots"][0]["semantic_lookup_same"], false);
        assert_eq!(actual["packages"][0]["semantic_lookup_same"], true);
    }
    assert_eq!(catalog.resource(URI).unwrap().json(), ATTR);
    assert_eq!(catalog.resource(KURI).unwrap().json(), SCALAR);
}
#[test]
fn domains_and_resource_scopes_remain_distinct_with_same_class_collisions() {
    let raw = doc(vec![node(A, B), node(B, A)]);
    let another = "sysmllibrary:///facd22dc-b5db-3e27-8679-83afde904863";
    let catalog = Catalog::from_resources(&[
        source(&raw),
        LibrarySource {
            classpath_path: "sysml.libraries/Other.json",
            resource_uri: another,
            json: &raw,
        },
    ])
    .unwrap();
    assert_eq!(catalog.by_node_id(URI, NodeId(A)).unwrap().semantic_id, B);
    assert_eq!(
        catalog.by_semantic_id(URI, SemanticId(A)).unwrap().node_id,
        B
    );
    assert_eq!(
        catalog.by_node_id(another, NodeId(A)).unwrap().semantic_id,
        B
    );
    let single = doc(vec![node(A, B)]);
    let catalog = Catalog::from_resources(&[source(&single)]).unwrap();
    assert!(catalog.by_node_id(URI, NodeId(B)).is_none());
    assert!(catalog.by_semantic_id(URI, SemanticId(A)).is_none());
    assert!(catalog.by_node_id("missing", NodeId(A)).is_none());
}
#[test]
fn duplicate_domains_resources_and_late_invalid_input_refuse_atomically() {
    refuses(
        &doc(vec![node(A, B), node(A, A)]),
        "duplicate node/semantic",
    );
    refuses(
        &doc(vec![node(A, B), node(B, B)]),
        "duplicate node/semantic",
    );
    assert!(Catalog::from_resources(&[source(ATTR), source(ATTR)]).is_err());
    let bad = LibrarySource {
        classpath_path: "sysml.libraries/Other.json",
        resource_uri: URI,
        json: ATTR,
    };
    assert!(Catalog::from_resources(&[source(ATTR), bad]).is_err());
    let before = ATTR.to_owned();
    let bad = LibrarySource {
        classpath_path: "sysml.libraries/Other.json",
        resource_uri: "sysmllibrary:///facd22dc-b5db-3e27-8679-83afde904863",
        json: "{}",
    };
    assert!(Catalog::from_resources(&[source(ATTR), bad]).is_err());
    assert_eq!(ATTR, before);
}
#[test]
fn duplicate_decoded_keys_malformed_ids_and_trailing_documents_refuse() {
    for raw in [
        r#"{"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[],"content":[]}"#,
        r#"{"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[],"x":{"id":1,"\u0069d":2}}"#,
    ] {
        refuses(raw, "duplicate JSON key");
    }
    let raw = doc(vec![node(A, B)]).replace(
        &format!("\"elementId\":\"{B}\""),
        &format!("\"elementId\":\"{B}\",\"elementId\":\"{A}\""),
    );
    refuses(&raw, "duplicate JSON key");
    refuses(&doc(vec![node("bad", B)]), "identity/class");
    refuses(
        &doc(vec![json!({"id":A,"eClass":"sysml:Namespace","data":{}})]),
        "semantic ID required",
    );
    refuses(&(ATTR.to_owned() + "{}"), "JSON syntax");
    refuses("{", "JSON syntax");
}
#[test]
fn opaque_tokens_unicode_strings_and_escaped_pointer_keys_are_preserved() {
    let mut v: Value = serde_json::from_str(&doc(vec![])).unwrap();
    let mut n = node(A, B);
    n["data"]["declaredName"] = json!("🚗长度");
    v["opaque/~"] = json!([n]);
    let raw = format!(
        " \n{} \n",
        v.to_string().trim_end_matches('}').to_owned()
            + ",\"number\":900719925474099312345,\"decimal\":1.5184049843810499}"
    );
    let catalog = Catalog::from_resources(&[source(&raw)]).unwrap();
    let r = catalog.resource(URI).unwrap();
    assert_eq!(r.json(), raw);
    let n = catalog.by_node_id(URI, NodeId(A)).unwrap();
    assert_eq!(n.declared_name.as_deref(), Some("🚗长度"));
    assert_eq!(n.pointer, "/opaque~1~0/0");
}
fn opaque_depth(n: usize) -> String {
    format!(
        r#"{{"ns":{{"sysml":"http://www.eclipse.org/syson/sysml"}},"content":[],"opaque":{}null{}}}"#,
        "[".repeat(n),
        "]".repeat(n)
    )
}
#[test]
fn real_corpus_depth_range_and_exact_depth_boundary_have_explicit_admission() {
    let raw = opaque_depth(208);
    assert!(Catalog::from_resources(&[source(&raw)]).is_ok());
    let raw = opaque_depth(MAX_DEPTH - 2);
    assert!(Catalog::from_resources(&[source(&raw)]).is_ok());
    refuses(&opaque_depth(MAX_DEPTH - 1), "depth limit");
    refuses(&opaque_depth(MAX_DEPTH), "depth limit");
    let mut v: Value = serde_json::from_str(&doc(vec![])).unwrap();
    v["[quoted]"] = json!("[{\\\"ignored");
    assert!(Catalog::from_resources(&[source(&v.to_string())]).is_ok());
}
#[test]
fn byte_count_uri_and_pointer_bounds_refuse_without_publishing_partial_catalogues() {
    let mut raw = doc(vec![]);
    raw.push_str(&" ".repeat(MAX_FILE - raw.len()));
    assert!(Catalog::from_resources(&[source(&raw)]).is_ok());
    raw.push(' ');
    refuses(&raw, "resource byte limit");
    let inputs: Vec<_> = (0..=MAX_RESOURCES).map(|_| source(ATTR)).collect();
    assert!(Catalog::from_resources(&inputs).is_err());
    for (path, uri) in [
        ("sysml.libraries/../bad.json", URI),
        ("sysml.libraries/Attributes.json", KURI),
        ("other/Attributes.json", URI),
    ] {
        assert!(
            Catalog::from_resources(&[LibrarySource {
                classpath_path: path,
                resource_uri: uri,
                json: ATTR
            }])
            .is_err()
        );
    }
    let mut v: Value = serde_json::from_str(&doc(vec![])).unwrap();
    v["x".repeat(32769)] = node(A, B);
    refuses(&v.to_string(), "pointer limit");
}
#[test]
fn local_manifest_requires_exact_fields_unique_keys_and_string_types() {
    let good=json!([{"classpath_path":"sysml.libraries/Attributes.json","resource_uri":URI,"file":"Attributes.json"}]).to_string();
    assert_eq!(manifest(&good).unwrap().len(), 1);
    for raw in [
        "{}",
        r#"[{"classpath_path":"a","classpath_path":"b","resource_uri":"c","file":"d"}]"#,
        r#"[{"classpath_path":"a","resource_uri":"c","file":null}]"#,
        r#"[{"classpath_path":"a","resource_uri":"c","file":"d","extra":0}]"#,
    ] {
        assert!(manifest(raw).is_err());
    }
}
#[test]
fn actual_cli_exports_real_lookup_rows_and_refuses_filesystem_and_late_failure() {
    use std::{fs, path::PathBuf, process::Command};
    let base = PathBuf::from(
        std::env::var_os("SYSON_LIBRARY_CATALOG_TEST_ROOT").expect("owned root required"),
    );
    assert!(base.is_absolute() && base.is_dir());
    let dir = base.join(format!("catalog-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("Attributes.json"), ATTR).unwrap();
    let row = json!({"classpath_path":"sysml.libraries/Attributes.json","resource_uri":URI,"file":"Attributes.json"});
    let file = dir.join("manifest.json");
    fs::write(&file, json!([row.clone()]).to_string()).unwrap();
    let run = |name: &str| {
        Command::new(env!("CARGO_BIN_EXE_syson-native-library-catalog"))
            .arg(&dir)
            .arg(name)
            .output()
            .unwrap()
    };
    let output = run("manifest.json");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let v: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["resources"][0]["nodes"].as_array().unwrap().len(), 13);
    let mut bad = row.clone();
    bad["resource_uri"] = json!("bad");
    fs::write(&file, json!([row, bad]).to_string()).unwrap();
    let output = run("manifest.json");
    assert!(!output.status.success() && output.stdout.is_empty());
    let output = run("../manifest.json");
    assert!(!output.status.success() && output.stdout.is_empty());
    let output = run("Attributes.json/missing");
    assert!(!output.status.success() && output.stdout.is_empty());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(dir.join("Attributes.json"), dir.join("link.json")).unwrap();
        let output = run("link.json");
        assert!(!output.status.success() && output.stdout.is_empty());
    }
    assert_eq!(
        fs::read_to_string(dir.join("Attributes.json")).unwrap(),
        ATTR
    );
}
