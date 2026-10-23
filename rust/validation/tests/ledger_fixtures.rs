// SPDX-License-Identifier: EPL-2.0
//! These nine stable ledger cases consume the checked-in JSON fixture bytes.
//! Expected results are source-derived; no upstream runtime has run them.
use serde_json::Value;
use std::collections::HashMap;
use syson_domain::names::name_matches;
use syson_editor::{Diagram, Node, Point, finish_rotation};
use syson_persistence::migrate_reference_uri;

fn read_fixture(raw: &str, expected_id: &str) -> Value {
    let fixture: Value = serde_json::from_str(raw).expect("valid case JSON");
    assert_eq!(fixture["case_id"], expected_id);
    fixture["input_and_expected"].clone()
}
macro_rules! fixture {
    ($number:literal) => {
        read_fixture(
            include_str!(concat!(
                "../../../porting/ledger-schema/fixtures/SYSON-CASE-",
                $number,
                ".json"
            )),
            concat!("SYSON-CASE-", $number),
        )
    };
}
fn name_case(value: Value) {
    assert_eq!(
        name_matches(value["candidate"].as_str(), value["query"].as_str()),
        value["expected"].as_bool().unwrap()
    );
}
fn layout_node(id: &str, x: f64) -> Node<()> {
    Node {
        id: id.into(),
        position: Point { x, y: 0.0 },
        width: Some(100.0),
        height: Some(50.0),
        resized_by_user: false,
        connection_handles: vec![],
        payload: (),
    }
}
fn rotation_case(value: Value) {
    let id = value["node_id"].as_str().unwrap();
    let prepared = Diagram::<(), ()> {
        nodes: vec![layout_node(id, value["prepared_x"].as_f64().unwrap())],
        edges: vec![],
    };
    let laid_out = Diagram::<(), ()> {
        nodes: if value.get("layout_nodes").is_some() {
            vec![]
        } else {
            vec![layout_node(id, value["layout_x"].as_f64().unwrap())]
        },
        edges: vec![],
    };
    let selected = value["selected"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let actual = finish_rotation(&prepared, &laid_out, &selected);
    assert_eq!(
        actual.nodes[0].position.x,
        value["expected_x"].as_f64().unwrap()
    );
}
fn migration_case(value: Value) {
    let ids = value
        .get("map")
        .and_then(Value::as_object)
        .map(|map| {
            map.iter()
                .map(|(key, value)| (key.clone(), value.as_str().unwrap().to_owned()))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    let mut called = false;
    let actual = migrate_reference_uri(
        value["uri"].as_str().unwrap(),
        value["containment"].as_bool().unwrap(),
        &ids,
        |_| {
            called = true;
            value["resolver"].as_str().map(str::to_owned)
        },
    );
    assert_eq!(actual, value["expected"].as_str().unwrap());
    if let Some(expected) = value["resolver_called"].as_bool() {
        assert_eq!(called, expected);
    }
}
#[test]
fn case_000001() {
    name_case(fixture!("000001"));
}
#[test]
fn case_000002() {
    name_case(fixture!("000002"));
}
#[test]
fn case_000003() {
    name_case(fixture!("000003"));
}
#[test]
fn case_000004() {
    rotation_case(fixture!("000004"));
}
#[test]
fn case_000005() {
    rotation_case(fixture!("000005"));
}
#[test]
fn case_000006() {
    rotation_case(fixture!("000006"));
}
#[test]
fn case_000007() {
    migration_case(fixture!("000007"));
}
#[test]
fn case_000008() {
    migration_case(fixture!("000008"));
}
#[test]
fn case_000009() {
    migration_case(fixture!("000009"));
}
