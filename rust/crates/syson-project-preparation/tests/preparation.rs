// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::collections::HashMap;
use syson_migration_pipeline::{self as library, Resolver};
use syson_migrations::{b25a, b25b, b25c, b25d};
use syson_model::Document;
use syson_persistence::{DIAGRAM_VERSION, REFERENCES_VERSION, ResolvedElement};
use syson_project_preparation::{
    Error, MAX_ITEMS, ProjectInput, RepresentationInput, SemanticInput, SemanticVersions, prepare,
};

const FIXTURES: &str =
    include_str!("../../../../porting/rust-parity-evidence/C03b/project-fixtures.json");
fn optional(v: &Value) -> Option<String> {
    v.as_str().map(str::to_owned)
}
fn input(id: &str) -> ProjectInput {
    let packet: Value = serde_json::from_str(FIXTURES).unwrap();
    let v = &packet["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == id)
        .unwrap()["input"];
    ProjectInput {
        documents: v["documents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| SemanticInput {
                json: d["json"].as_str().unwrap().into(),
                references_version: optional(&d["references_version"]),
                versions: SemanticVersions {
                    connector: optional(&d["versions"]["connector"]),
                    annotation: optional(&d["versions"]["annotation"]),
                    view: optional(&d["versions"]["view"]),
                },
            })
            .collect(),
        representations: v["representations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| RepresentationInput {
                kind: r["kind"].as_str().unwrap().into(),
                json: r["json"].as_str().unwrap().into(),
                control_version: optional(&r["control_version"]),
                diagram_version: optional(&r["diagram_version"]),
            })
            .collect(),
    }
}
fn ids() -> HashMap<String, String> {
    [
        ("old".into(), "new".into()),
        ("new".into(), "wrong-replay".into()),
    ]
    .into()
}
#[derive(Default)]
struct Snapshot {
    calls: Vec<String>,
    fail: Option<String>,
}
impl Resolver for Snapshot {
    fn resolve(&mut self, key: &str) -> Result<Option<ResolvedElement>, String> {
        self.calls.push(key.into());
        if self.fail.as_deref() == Some(key) {
            Err("owned snapshot unavailable".into())
        } else {
            Ok(None)
        }
    }
}
fn node<'a>(v: &'a Value, id: &str) -> &'a Value {
    fn find<'a>(v: &'a Value, id: &str) -> Option<&'a Value> {
        if v["data"]["elementId"] == id {
            return Some(v);
        }
        match v {
            Value::Object(o) => o.values().find_map(|v| find(v, id)),
            Value::Array(a) => a.iter().find_map(|v| find(v, id)),
            _ => None,
        }
    }
    find(v, id).unwrap()
}

#[test]
fn actual_consuming_recipe_removes_legacy_fields_then_admits_and_migrates_both_project_lanes() {
    let input = input("combined");
    let before = input.clone();
    let mut resolver = Snapshot::default();
    let output = prepare(&input, &ids(), &mut resolver).unwrap();
    let doc = &output.project().documents[0].document;
    assert_eq!(
        doc.elements(),
        Document::from_json(&input.documents[0].json)
            .unwrap()
            .elements()
    );
    for (id, field) in [
        ("annotation-semantic", "annotatingElement"),
        ("comment-semantic", "annotation"),
        ("connection-semantic", "source"),
        ("connection-semantic", "target"),
        ("view-semantic", "exposedElement"),
    ] {
        assert!(doc.element(id).unwrap()["data"].get(field).is_none());
    }
    assert_eq!(
        doc.element("comment-semantic").unwrap()["data"]["body"],
        "Keep this note 🚗"
    );
    assert_eq!(
        doc.element("typing-semantic").unwrap()["data"]["type"],
        "sysml:Type sysmllibrary:/Types#new"
    );
    assert_eq!(
        doc.element("typing-semantic").unwrap()["data"]["typedFeature"],
        "usage-node"
    );
    let versions = &output.wire().documents[0].versions;
    assert_eq!(versions.connector.as_deref(), Some(b25b::VERSION));
    assert_eq!(versions.annotation.as_deref(), Some(b25a::VERSION));
    assert_eq!(versions.view.as_deref(), Some(b25c::VERSION));
    assert_eq!(
        output.wire().documents[0].references_version.as_deref(),
        Some(REFERENCES_VERSION)
    );
    let rep = &output.project().representations[0].content;
    for (n, (_, new)) in b25d::MAPPINGS.iter().enumerate() {
        assert_eq!(rep["nodes"][n]["descriptionId"], *new);
    }
    assert_eq!(rep["nodes"][0]["targetObjectId"], "new");
    assert_eq!(rep["readOnly"], true);
    assert_eq!(
        rep["opaque"]["integer"].to_string(),
        "900719925474099312345"
    );
    assert_eq!(
        output.wire().representations[0].control_version.as_deref(),
        Some(b25d::VERSION)
    );
    assert_eq!(
        output.wire().representations[0].diagram_version.as_deref(),
        Some(DIAGRAM_VERSION)
    );
    assert_eq!(resolver.calls, ["sysmllibrary:/Types#old", "old"]);
    assert_eq!(
        Document::from_json(&output.wire().documents[0].json).unwrap(),
        *doc
    );
    assert_eq!(
        serde_json::from_str::<Value>(&output.wire().representations[0].json).unwrap(),
        *rep
    );
    assert_eq!(input, before);
}

#[test]
fn recipe_is_a_real_consumer_not_a_metadata_only_preflight() {
    let input = input("combined");
    let unprepared = library::Project {
        documents: vec![library::SemanticDocument {
            document: Document::from_json(&input.documents[0].json).unwrap(),
            references_version: None,
        }],
        representations: vec![],
    };
    // B01 cannot consume the old object-valued reference shapes on its own.
    assert!(library::prepare(&unprepared, &ids(), &mut Snapshot::default()).is_err());
    assert!(prepare(&input, &ids(), &mut Snapshot::default()).is_ok());
}

#[test]
fn replay_does_not_double_map_ids_and_control_stamp_is_independent_from_library_stamps() {
    let first = prepare(&input("combined"), &ids(), &mut Snapshot::default()).unwrap();
    let mut resolver = Snapshot::default();
    let replay = prepare(first.wire(), &ids(), &mut resolver).unwrap();
    assert_eq!(first, replay);
    assert!(resolver.calls.is_empty());
    let mut only_control = first.wire().clone();
    only_control.representations[0].control_version = None;
    let same = prepare(&only_control, &ids(), &mut resolver).unwrap();
    assert_eq!(same, first);
    assert!(resolver.calls.is_empty());
    let (wire, typed) = same.into_parts();
    assert_eq!(wire, first.wire().clone());
    assert_eq!(typed, *first.project());
}

#[test]
fn foreign_first_namespace_survives_b01_and_replay_without_activating_pending_removals() {
    let input = input("foreign-first");
    let first = prepare(&input, &ids(), &mut Snapshot::default()).unwrap();
    let wire = &first.wire().documents[0];
    assert!(wire.json.find("\"z\":").unwrap() < wire.json.find("\"a\":").unwrap());
    assert_eq!(wire.versions, SemanticVersions::default());
    let doc = &first.project().documents[0].document;
    assert!(
        doc.element("comment-semantic").unwrap()["data"]
            .get("annotation")
            .is_some()
    );
    assert_eq!(
        doc.element("typing-semantic").unwrap()["data"]["type"],
        "sysml:Type sysmllibrary:/Types#new"
    );
    let mut resolver = Snapshot::default();
    assert_eq!(prepare(first.wire(), &ids(), &mut resolver).unwrap(), first);
    assert!(resolver.calls.is_empty());
}

#[test]
fn foreign_representations_keep_both_unknown_versions_and_opaque_json_values() {
    let input = input("foreign-kind");
    let mut resolver = Snapshot::default();
    let output = prepare(&input, &ids(), &mut resolver).unwrap();
    assert_eq!(output.wire().representations[0], input.representations[0]);
    assert!(resolver.calls.is_empty());
}

#[test]
fn every_selected_future_watermark_refuses_before_any_earlier_resolvable_library_work() {
    for which in 0..6 {
        let mut input = input("combined");
        input.documents.push(input.documents[0].clone());
        input.representations.push(input.representations[0].clone());
        let future = Some("future".into());
        match which {
            0 => input.documents[1].versions.connector = future,
            1 => input.documents[1].versions.annotation = future,
            2 => input.documents[1].versions.view = future,
            3 => input.documents[1].references_version = future,
            4 => input.representations[1].control_version = future,
            _ => input.representations[1].diagram_version = future,
        }
        let before = input.clone();
        let mut resolver = Snapshot::default();
        assert!(matches!(
            prepare(&input, &ids(), &mut resolver),
            Err(Error::Version { .. })
        ));
        assert!(resolver.calls.is_empty());
        assert_eq!(input, before);
    }
}

#[test]
fn late_document_admission_failure_discards_earlier_private_migrations() {
    let published = prepare(&input("combined"), &ids(), &mut Snapshot::default()).unwrap();
    let before = published.wire().clone();
    let mut input = input("combined");
    let mut invalid: Value = serde_json::from_str(&input.documents[0].json).unwrap();
    let duplicate = invalid["content"][0].clone();
    invalid["content"].as_array_mut().unwrap().push(duplicate);
    let mut bad = input.documents[0].clone();
    bad.json = invalid.to_string();
    input.documents.push(bad);
    let original = input.clone();
    let mut resolver = Snapshot::default();
    assert!(matches!(
        prepare(&input, &ids(), &mut resolver),
        Err(Error::Admission { index: 1, .. })
    ));
    assert!(resolver.calls.is_empty());
    assert_eq!(input, original);
    assert_eq!(published.wire(), &before);
}

#[test]
fn late_control_and_resolver_failures_preserve_input_and_previous_output_not_external_effects() {
    let published = prepare(&input("combined"), &ids(), &mut Snapshot::default()).unwrap();
    let before = published.wire().clone();
    let mut bad = input("combined");
    bad.representations[0].json =
        json!({"a":{"descriptionId":b25d::MAPPINGS[0].0},"z":[[]]}).to_string();
    let original = bad.clone();
    let mut resolver = Snapshot::default();
    assert!(matches!(
        prepare(&bad, &ids(), &mut resolver),
        Err(Error::Representation {
            cause: b25d::Error::NestedArray,
            ..
        })
    ));
    assert_eq!(resolver.calls, ["sysmllibrary:/Types#old"]);
    assert_eq!(bad, original);
    let good = input("combined");
    let original = good.clone();
    let mut resolver = Snapshot {
        fail: Some("old".into()),
        ..Snapshot::default()
    };
    assert!(matches!(
        prepare(&good, &ids(), &mut resolver),
        Err(Error::Library(library::Error::Resolver(_)))
    ));
    assert_eq!(resolver.calls, ["sysmllibrary:/Types#old", "old"]);
    assert_eq!(good, original);
    assert_eq!(published.wire(), &before);
}

#[test]
fn item_input_aggregate_value_depth_and_mapping_budgets_refuse_before_resolver() {
    let mut cases = Vec::new();
    let foreign = input("foreign-kind");
    let mut too_many = foreign.clone();
    too_many.representations = vec![foreign.representations[0].clone(); MAX_ITEMS + 1];
    cases.push(too_many);
    let mut huge = foreign.clone();
    huge.representations[0].json = " ".repeat(syson_migrations::MAX_BYTES + 1);
    cases.push(huge);
    let mut aggregate = input("combined");
    let mut padded: Value = serde_json::from_str(&aggregate.documents[0].json).unwrap();
    padded["padding"] = json!("x".repeat(syson_migrations::MAX_BYTES - 4096));
    aggregate.documents[0].json = padded.to_string();
    aggregate.documents = vec![aggregate.documents[0].clone(); 9];
    cases.push(aggregate);
    let mut many = foreign.clone();
    many.representations[0].json = json!(vec![0; syson_migrations::MAX_VALUES]).to_string();
    cases.push(many);
    let mut deep = json!(0);
    for _ in 0..66 {
        deep = json!({"child":deep});
    }
    let mut nested = foreign.clone();
    nested.representations[0].json = deep.to_string();
    cases.push(nested);
    for input in cases {
        let mut resolver = Snapshot::default();
        assert!(matches!(
            prepare(&input, &ids(), &mut resolver),
            Err(Error::Limit)
        ));
        assert!(resolver.calls.is_empty());
    }
    let mut resolver = Snapshot::default();
    let map = HashMap::from([("old".into(), "x".repeat(syson_migrations::MAX_BYTES + 1))]);
    assert!(matches!(
        prepare(&input("combined"), &map, &mut resolver),
        Err(Error::Limit)
    ));
    assert!(resolver.calls.is_empty());
}

#[test]
fn b01_expansion_limit_is_enforced_after_real_semantic_cleanup_without_publishing_stamps() {
    let mut input = input("combined");
    let mut value: Value = serde_json::from_str(&input.documents[0].json).unwrap();
    let mut extra = node(&value, "typing-semantic").clone();
    extra["id"] = json!("extra-typing-node");
    extra["data"]["elementId"] = json!("extra-typing-semantic");
    value["content"][0]["data"]["ownedRelationship"]
        .as_array_mut()
        .unwrap()
        .push(extra);
    input.documents[0].json = value.to_string();
    let before = input.clone();
    let mut resolver = Snapshot::default();
    let map = HashMap::from([("old".into(), "x".repeat(600_000))]);
    assert!(matches!(
        prepare(&input, &map, &mut resolver),
        Err(Error::Library(library::Error::Limit))
    ));
    assert_eq!(resolver.calls.len(), 2);
    assert_eq!(input, before);
}
