// SPDX-License-Identifier: EPL-2.0
use std::collections::HashMap;
use syson_persistence::*;
const TABLE: &str = include_str!(
    "../../../../backend/application/syson-application-configuration/src/main/resources/migration/stdlib_oldId_to_elementId.txt"
);
const SQL: &str = include_str!(
    "../../../../backend/application/syson-application/src/test/resources/scripts/database-content/StandardLibrariesElementsIdsMigrationParticipant-Test.sql"
);
const OLD_STRING: &str = "b352ded6-38ba-4e7e-ae55-9299cb0cc6d0";
const STRING: &str = "76028d3d-69a4-5e12-9002-ce403e0244bd";
const OLD_START: &str = "03262155-734a-473f-9535-262925cd5b8b";
const START: &str = "9a0d2905-0f9c-5bb4-af74-9780d6db1817";
#[test]
fn supplied_table_and_upstream_fixture_preserve_expected_string_and_action_ids() {
    let map = parse_id_map(TABLE);
    assert_eq!(map.get(OLD_STRING).unwrap(), STRING);
    assert_eq!(map.get(OLD_START).unwrap(), START);
    assert!(SQL.contains(OLD_STRING));
    assert!(SQL.contains(OLD_START));
    assert_eq!(map.len(), 123_773);
}
#[test]
fn parser_matches_java_split_trim_duplicate_and_line_end_rules() {
    let map = parse_id_map(
        " a = b \r\nx=one\rx=two\nignored\na=b=c\nempty=\n=valid\ntrailing=value==\n\u{a0}key\u{a0}=v\n",
    );
    assert_eq!(map.get("a").unwrap(), "b");
    assert_eq!(map.get("x").unwrap(), "two");
    assert_eq!(map.get("").unwrap(), "valid");
    assert_eq!(map.get("trailing").unwrap(), "value");
    assert_eq!(map.get("\u{a0}key\u{a0}").unwrap(), "v");
    assert!(!map.contains_key("empty"));
}
#[test]
fn semantic_uri_from_upstream_sql_maps_to_upstream_expected_element_id() {
    let uri =
        format!("sysml:DataType kermllibrary:///b2c6dd37-2084-3ce4-9ce2-580fdf30629c#{OLD_STRING}");
    assert!(SQL.contains(&uri));
    let out = migrate_reference_uri(&uri, false, &parse_id_map(TABLE), |lookup| {
        assert_eq!(lookup, uri.strip_prefix("sysml:DataType ").unwrap());
        None
    });
    assert_eq!(out, uri.replace(OLD_STRING, STRING));
}
#[test]
fn live_element_resolution_takes_precedence_over_fallback() {
    let map = HashMap::from([("old".into(), "fallback".into())]);
    assert_eq!(
        migrate_reference_uri("sysmllibrary:///a#old", false, &map, |_| Some(
            "live".into()
        )),
        "sysmllibrary:///a#live"
    );
}
#[test]
fn ordinary_and_containment_references_never_call_resolver() {
    for (uri, containment) in [("sysmllibrary:///a#old", true), ("project:///a#old", false)] {
        assert_eq!(
            migrate_reference_uri(uri, containment, &HashMap::new(), |_| panic!(
                "must not resolve"
            )),
            uri
        );
    }
}
#[test]
fn unknown_reference_unchanged_and_missing_fragment_follows_upstream_slice_rules() {
    assert_eq!(
        migrate_reference_uri("kermllibrary:///a#unknown", false, &HashMap::new(), |_| {
            None
        }),
        "kermllibrary:///a#unknown"
    );
    assert_eq!(
        migrate_reference_uri("kermllibrary:///a", false, &HashMap::new(), |_| Some(
            "resolved".into()
        )),
        "resolved"
    );
}
#[test]
fn diagram_start_id_matches_upstream_assertion_and_is_idempotent_without_resolution() {
    let map = parse_id_map(TABLE);
    assert_eq!(
        migrate_diagram_target("targetObjectId", Some(OLD_START), &map, |_| None),
        Some(START.into())
    );
    assert_eq!(
        migrate_diagram_target("targetObjectId", Some(START), &map, |_| None),
        None
    );
}
#[test]
fn resolved_nonlibrary_element_blocks_map_fallback() {
    let map = HashMap::from([("old".into(), "fallback".into())]);
    assert_eq!(
        migrate_diagram_target("targetObjectId", Some("old"), &map, |_| Some(
            ResolvedElement {
                element_id: "current".into(),
                resource_uri: Some("project:///a".into())
            }
        )),
        None
    );
    assert_eq!(
        migrate_diagram_target("targetObjectId", Some("old"), &map, |_| Some(
            ResolvedElement {
                element_id: "current".into(),
                resource_uri: Some("sysmllibrary:///a".into())
            }
        )),
        Some("current".into())
    );
}
#[test]
fn non_target_and_non_string_json_attributes_never_resolve() {
    assert_eq!(
        migrate_diagram_target("other", Some("old"), &HashMap::new(), |_| panic!()),
        None
    );
    assert_eq!(
        migrate_diagram_target("targetObjectId", None, &HashMap::new(), |_| panic!()),
        None
    );
}
#[test]
fn project_download_filter_preserves_source_prefix_and_null_semantics() {
    for uri in [
        Some("sysmllibrary:///a"),
        Some("kermllibrary:///a"),
        Some("sysmllibraryExtra"),
    ] {
        assert!(!should_persist_in_project_download(uri));
    }
    for uri in [
        None,
        Some("project:///a"),
        Some("SYSmllibrary:///a"),
        Some("other/sysmllibrary"),
    ] {
        assert!(should_persist_in_project_download(uri));
    }
}
