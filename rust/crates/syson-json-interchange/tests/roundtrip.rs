// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use syson_json_interchange::{Bundle, Error, Input, MAX_NODES, MAX_RESOURCE_BYTES, MAX_RESOURCES};
use syson_metamodel::ClassifierId as C;
use syson_model::{Document, Editor, NodeIds};
use syson_references::{Error as R, FragmentIdentity as F, Resource};
use syson_view_usage::{Ids, Presence, stage};
const LIB: &str = "sysmllibrary:///aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
fn ids(n: usize) -> NodeIds {
    NodeIds::new(
        &format!("10000000-0000-4000-8000-{n:012x}"),
        &format!("00000000-0000-4000-8000-{n:012x}"),
    )
    .unwrap()
}
fn node(n: usize, class: &str, data: Value) -> Value {
    let mut data = data.as_object().unwrap().clone();
    data.insert("elementId".into(), json!(ids(n).element));
    json!({"id":ids(n).resource,"eClass":format!("sysml:{class}"),"data":data})
}
fn envelope(nodes: Vec<Value>) -> Value {
    json!({"json":{"version":"1.0","encoding":"utf-8"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":nodes})
}
fn input(key: &str, value: Value) -> Input {
    Input {
        key: key.into(),
        json: value.to_string(),
        fragments: F::SemanticElementId,
    }
}
fn local() -> Input {
    input(
        "model",
        envelope(vec![node(
            1,
            "Package",
            json!({"declaredName":"Vehicle","ownedRelationship":[
                node(2,"OwningMembership",json!({"ownedRelatedElement":[node(3,"PartDefinition",json!({"declaredName":"VehicleType"}))]})),
                node(4,"OwningMembership",json!({"ownedRelatedElement":[node(5,"PartUsage",json!({"declaredName":"vehicle","ownedRelationship":[node(6,"FeatureTyping",json!({"type":ids(3).resource,"typedFeature":ids(5).resource}))]}))]})),
                node(7,"OwningMembership",json!({"ownedRelatedElement":[node(8,"PortDefinition",json!({"declaredName":"Socket"}))]})),
                node(9,"OwningMembership",json!({"ownedRelatedElement":[node(10,"PortUsage",json!({"declaredName":"socket","ownedRelationship":[node(11,"FeatureTyping",json!({"type":ids(8).resource,"typedFeature":ids(10).resource}))]}))]}))
            ]}),
        )]),
    )
}
fn changed_type(wire: Value) -> Input {
    let mut value: Value = serde_json::from_str(&local().json).unwrap();
    value["content"][0]["data"]["ownedRelationship"][1]["data"]["ownedRelatedElement"][0]["data"]
        ["ownedRelationship"][0]["data"]["type"] = wire;
    input("model", value)
}
fn library() -> Input {
    input(
        LIB,
        envelope(vec![node(
            100,
            "Package",
            json!({"declaredName":"StandardViewDefinitions","ownedRelationship":[node(101,"OwningMembership",json!({"ownedRelatedElement":[node(102,"ViewDefinition",json!({"declaredName":"GeneralView"}))]}))]}),
        )]),
    )
}
#[test]
fn package_part_port_roundtrip_preserves_values_and_local_node_identity() {
    let original = local();
    let bundle = Bundle::load(vec![original.clone()]).unwrap();
    let target = bundle
        .resolve("model", &json!(ids(3).resource), C::PartDefinition)
        .unwrap()
        .unwrap();
    assert_eq!(target.element_id(), ids(3).element);
    assert_eq!(target.node_id(), ids(3).resource);
    let saved = bundle.save().unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&saved[0].json).unwrap(),
        serde_json::from_str::<Value>(&original.json).unwrap()
    );
    let reloaded = Bundle::load(saved).unwrap();
    assert_eq!(reloaded.document("model"), bundle.document("model"));
    assert!(matches!(reloaded.target(&target), Err(R::StaleSnapshot)));
}
#[test]
fn actual_f05_external_view_projection_roundtrips_all_resources() {
    let model = Document::from_json(&local().json).unwrap();
    let lib = library();
    let mut editor = Editor::new(model);
    let tx = stage(
        editor.begin(),
        &ids(1).element,
        Ids {
            membership: ids(20),
            view: ids(21),
            typing: ids(22),
        },
        "model",
        vec![Resource {
            key: LIB.into(),
            document: Document::from_json(&lib.json).unwrap(),
            fragments: F::SemanticElementId,
        }],
        Presence::ResourceSet,
        || false,
    )
    .unwrap();
    editor.commit(tx).unwrap();
    let before = editor.document().clone();
    let bundle = Bundle::load(vec![
        Input {
            json: before.to_json().unwrap(),
            ..local()
        },
        lib,
    ])
    .unwrap();
    let reloaded = Bundle::load(bundle.save().unwrap()).unwrap();
    let typing = reloaded
        .document("model")
        .unwrap()
        .element(&ids(22).element)
        .unwrap();
    let wire = &typing["data"]["type"];
    assert_eq!(
        wire,
        &json!(format!("sysml:ViewDefinition {LIB}#{}", ids(102).element))
    );
    let target = reloaded
        .resolve("model", wire, C::ViewDefinition)
        .unwrap()
        .unwrap();
    assert_eq!(target.resource(), LIB);
    assert_eq!(target.element_id(), ids(102).element);
    assert_eq!(editor.document(), &before);
}
#[test]
fn unknown_envelope_node_and_data_fields_survive_as_opaque_values() {
    let mut value: Value = serde_json::from_str(&local().json).unwrap();
    value["vendor"] = json!({"exact":900719925474099312345u128,"wire":"resource:///opaque#x"});
    value["content"][0]["annotation"] = json!([true, null, "quoted \" text"]);
    value["content"][0]["data"]["vendorReference"] = json!({"unknown":"unchanged"});
    let bundle = Bundle::load(vec![input("model", value.clone())]).unwrap();
    let saved = bundle.save().unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&saved[0].json).unwrap(),
        value
    );
}
#[test]
fn dangling_local_ref_has_position_and_never_uses_semantic_fallback() {
    for wire in [ids(999).resource, ids(3).element] {
        match Bundle::load(vec![changed_type(json!(wire))]) {
            Err(Error::Reference {
                resource,
                element,
                feature,
                cause: R::Unresolved { .. },
            }) => {
                assert_eq!(resource, "model");
                assert_eq!(element, ids(6).element);
                assert_eq!(feature, "type");
            }
            _ => panic!("expected scoped unresolved reference"),
        }
    }
}
#[test]
fn external_fragment_policy_is_explicit_and_same_ids_in_other_resource_do_not_alias() {
    let wire = json!(format!("sysml:PartDefinition {LIB}#{}", ids(3).element));
    let mut lib = local();
    lib.key = LIB.into();
    let bundle = Bundle::load(vec![changed_type(wire.clone()), lib.clone()]).unwrap();
    let target = bundle
        .resolve("model", &wire, C::PartDefinition)
        .unwrap()
        .unwrap();
    assert_eq!(target.resource(), LIB);
    lib.fragments = F::JsonNodeId;
    assert!(matches!(
        Bundle::load(vec![changed_type(wire), lib]),
        Err(Error::Reference {
            cause: R::Unresolved { .. },
            ..
        })
    ));
}
#[test]
fn missing_external_resources_are_not_discovered_or_loaded() {
    let wire = json!(format!("sysml:PartDefinition {LIB}#{}", ids(3).element));
    assert!(matches!(
        Bundle::load(vec![changed_type(wire)]),
        Err(Error::Reference {
            cause: R::Unresolved { .. },
            ..
        })
    ));
}
#[test]
fn reference_class_tags_and_target_classes_are_both_checked() {
    let wire = json!(format!("sysml:ViewDefinition {LIB}#{}", ids(3).element));
    let mut lib = local();
    lib.key = LIB.into();
    assert!(matches!(
        Bundle::load(vec![changed_type(wire), lib]),
        Err(Error::Reference {
            cause: R::TypeMismatch { .. },
            ..
        })
    ));
    assert!(matches!(
        Bundle::load(vec![changed_type(json!(ids(2).resource))]),
        Err(Error::Reference {
            cause: R::TypeMismatch { .. },
            ..
        })
    ));
}
#[test]
fn unsupported_reference_forms_and_known_reference_features_refuse() {
    for wire in [
        json!(format!("sysml:PartDefinition {}", ids(3).resource)),
        json!("resource:///file#fragment"),
        json!([ids(3).resource]),
    ] {
        assert!(matches!(
            Bundle::load(vec![changed_type(wire)]),
            Err(Error::Reference {
                cause: R::Unsupported(_),
                ..
            })
        ));
    }
    let value = envelope(vec![node(1, "Package", json!({"owner":ids(2).resource}))]);
    assert!(matches!(
        Bundle::load(vec![input("model", value)]),
        Err(Error::UnsupportedReference { .. })
    ));
}
#[test]
fn absent_and_explicit_null_type_remain_distinct_json_values() {
    let absent = input("model", envelope(vec![node(1, "FeatureTyping", json!({}))]));
    let null = input(
        "model",
        envelope(vec![node(1, "FeatureTyping", json!({"type":null}))]),
    );
    let a = Bundle::load(vec![absent]).unwrap();
    let b = Bundle::load(vec![null]).unwrap();
    assert!(
        a.document("model")
            .unwrap()
            .element(&ids(1).element)
            .unwrap()["data"]
            .get("type")
            .is_none()
    );
    assert!(
        b.document("model")
            .unwrap()
            .element(&ids(1).element)
            .unwrap()["data"]
            .get("type")
            .unwrap()
            .is_null()
    );
}
#[test]
fn later_failure_does_not_publish_partial_replacement_or_mutate_previous_bundle() {
    let mut published = Bundle::load(vec![local()]).unwrap();
    let before = published.save().unwrap()[0].json.clone();
    let mut bad = library();
    bad.json = "broken".into();
    let staged = Bundle::load(vec![local(), bad]);
    if let Ok(value) = staged {
        published = value;
    }
    assert_eq!(published.save().unwrap()[0].json, before);
}
#[test]
fn duplicate_resource_keys_ids_and_invalid_ids_refuse() {
    assert!(matches!(
        Bundle::load(vec![local(), local()]),
        Err(Error::Identity)
    ));
    for value in [
        envelope(vec![
            node(1, "Package", json!({})),
            node(1, "Package", json!({})),
        ]),
        json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{"id":"not-uuid","eClass":"sysml:Package","data":{"elementId":ids(1).element}}]}),
    ] {
        assert!(matches!(
            Bundle::load(vec![input("model", value)]),
            Err(Error::Model(_))
        ));
    }
}
#[test]
fn finite_class_and_envelope_contract_refuses_other_families() {
    assert!(matches!(
        Bundle::load(vec![input(
            "model",
            envelope(vec![node(1, "ActionUsage", json!({}))])
        )]),
        Err(Error::UnsupportedClass(_))
    ));
    assert!(matches!(
        Bundle::load(vec![input("model", json!({"content":[]}))]),
        Err(Error::Model(_))
    ));
    assert!(
        Bundle::load(vec![input(
            "model",
            envelope(vec![node(1, "FeatureMembership", json!({}))])
        )])
        .is_ok()
    );
}
#[test]
fn resource_byte_count_and_total_node_limits_are_enforced() {
    assert!(matches!(Bundle::load(vec![]), Err(Error::Limit)));
    let mut resources = vec![];
    for n in 0..=MAX_RESOURCES {
        resources.push(input(&format!("r{n}"), envelope(vec![])));
    }
    assert!(matches!(Bundle::load(resources), Err(Error::Limit)));
    let mut oversized = local();
    oversized.json = " ".repeat(MAX_RESOURCE_BYTES + 1);
    assert!(matches!(Bundle::load(vec![oversized]), Err(Error::Limit)));
    let nodes = (1..=MAX_NODES + 1)
        .map(|n| node(n, "Package", json!({})))
        .collect();
    assert!(matches!(
        Bundle::load(vec![input("model", envelope(nodes))]),
        Err(Error::Limit)
    ));
}

#[test]
fn accepted_uploads_also_fit_canonical_save_and_reload_budget() {
    let dense = input(
        "model",
        envelope(vec![node(
            1,
            "Package",
            json!({"opaque":vec![0u8;100_000]}),
        )]),
    );
    assert!(dense.json.len() < MAX_RESOURCE_BYTES);
    assert!(
        Document::from_json(&dense.json)
            .unwrap()
            .to_json()
            .unwrap()
            .len()
            > MAX_RESOURCE_BYTES
    );
    assert!(matches!(Bundle::load(vec![dense]), Err(Error::Limit)));
    let near = input(
        "model",
        envelope(vec![node(
            1,
            "Package",
            json!({"opaque":"x".repeat(MAX_RESOURCE_BYTES-1024)}),
        )]),
    );
    let bundle = Bundle::load(vec![near]).unwrap();
    let saved = bundle.save().unwrap();
    assert!(saved[0].json.len() <= MAX_RESOURCE_BYTES);
    let reloaded = Bundle::load(saved).unwrap();
    assert_eq!(bundle.document("model"), reloaded.document("model"));
}
