// SPDX-License-Identifier: EPL-2.0
//! Independent source-derived cases; no Java/TypeScript oracle was executed.
use std::collections::HashMap;
use syson_domain::names::name_matches;
use syson_editor::*;
use syson_persistence::*;
#[test]
fn java_strip_unicode_boundary() {
    for c in ['\u{a0}', '\u{2007}', '\u{202f}', '\u{85}'] {
        assert!(!name_matches(Some(&format!("{c}x")), Some("x")));
    }
    for c in [
        '\u{1c}', '\u{1d}', '\u{1e}', '\u{1f}', '\u{2006}', '\u{2008}',
    ] {
        assert!(name_matches(Some(&format!("{c}x{c}")), Some("x")));
    }
}
fn node(id: &str, x: f64) -> Node<()> {
    Node {
        id: id.into(),
        position: Point { x, y: 0.0 },
        width: Some(20.0),
        height: Some(10.0),
        resized_by_user: false,
        connection_handles: vec![],
        payload: (),
    }
}
#[test]
fn empty_id_find_result_is_false_even_when_matched() {
    let prepared = Diagram::<(), ()> {
        nodes: vec![node("", 0.0)],
        edges: vec![],
    };
    let laid_out = Diagram::<(), ()> {
        nodes: vec![node("", 100.0)],
        edges: vec![],
    };
    assert_eq!(
        finish_rotation(&prepared, &laid_out, &["".into()]).nodes[0]
            .position
            .x,
        0.0
    );
}
#[test]
fn first_empty_edge_match_prevents_later_truthy_match() {
    let edge = Edge {
        id: "e".into(),
        source: "".into(),
        target: "x".into(),
        data: Some(EdgeData {
            bending_points: Some(vec![]),
            payload: (),
        }),
    };
    let prepared = Diagram::<(), ()> {
        nodes: vec![],
        edges: vec![edge.clone()],
    };
    let mut updated = edge.clone();
    updated.data = None;
    let laid_out = Diagram::<(), ()> {
        nodes: vec![],
        edges: vec![updated],
    };
    assert_eq!(
        finish_rotation(&prepared, &laid_out, &["".into(), "x".into()]).edges[0],
        edge
    );
}
#[test]
fn nan_computed_minimum_retains_previous_height() {
    let actual = layout_package_leaf(LeafLayout {
        measured_label_height: Some(f64::NAN),
        resized_by_user: true,
        previous_size: Size {
            width: 90.0,
            height: 60.0,
        },
        ..Default::default()
    });
    assert_eq!(actual.height, 60.0);
}
#[test]
fn reference_priority_is_sysml_substring_even_when_kerml_comes_first() {
    let input = "sysml:Type kermllibrary:///a/sysmllibrary:///b#old";
    assert_eq!(
        migrate_reference_uri(input, false, &HashMap::new(), |lookup| {
            assert_eq!(lookup, "sysmllibrary:///b#old");
            Some("new".into())
        }),
        "sysml:Type kermllibrary:///a/sysmllibrary:///b#new"
    );
}
#[test]
fn diagram_found_nonlibrary_blocks_old_id_fallback() {
    let ids = HashMap::from([("old".into(), "new".into())]);
    assert_eq!(
        migrate_diagram_target("targetObjectId", Some("old"), &ids, |_| Some(
            ResolvedElement {
                element_id: "current".into(),
                resource_uri: None
            }
        )),
        None
    );
}
#[test]
fn parser_preserves_java_trim_nonbreaking_space_and_trailing_split_rules() {
    let ids = parse_id_map("a=b==\r\nx=\u{a0}v\u{a0}\nx=final\r=zero\nempty=");
    assert_eq!(ids.get("a").map(String::as_str), Some("b"));
    assert_eq!(ids.get("x").map(String::as_str), Some("final"));
    assert_eq!(ids.get("").map(String::as_str), Some("zero"));
    assert!(!ids.contains_key("empty"));
    assert_eq!(parse_id_map("x=\u{a0}v\u{a0}")["x"], "\u{a0}v\u{a0}");
}
