// Copyright (c) 2024, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
// Source: MetamodelMutationElementService.addChildInParent, pinned6ba2ac64.
use serde_json::json;
use syson_metamodel::ClassifierId;
use syson_model::{Document, Editor, NodeIds};
fn ids(n: u32) -> NodeIds {
    NodeIds::new(
        &format!("00000000-0000-4000-8000-{n:012x}"),
        &format!("10000000-0000-4000-8000-{n:012x}"),
    )
    .unwrap()
}
fn imported(class: &str) -> Document {
    Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{"id":"resource","eClass":class,"data":{"elementId":"parent","declaredName":"Parent"}}]}).to_string()).unwrap()
}
#[test]
fn add_child_uses_type_feature_rule_for_imported_action_and_namespace() {
    for (parent, expected) in [
        ("sysml:ActionUsage", "sysml:FeatureMembership"),
        ("sysml:Namespace", "sysml:OwningMembership"),
        ("sysml:Documentation", "sysml:OwningMembership"),
    ] {
        let mut editor = Editor::new(imported(parent));
        assert!(editor.document().can_create_in("parent"));
        let mut tx = editor.begin();
        tx.create_part_usage("parent", ids(1), ids(2), "child")
            .unwrap();
        editor.commit(tx).unwrap();
        assert_eq!(
            editor.document().value()["content"][0]["data"]["ownedRelationship"][0]["eClass"],
            expected
        );
    }
}
#[test]
fn nonfeature_package_child_always_uses_owning_membership() {
    let mut editor = Editor::new(imported("sysml:ActionUsage"));
    let mut tx = editor.begin();
    tx.create_package(Some("parent"), ids(1), Some(ids(2)), "Package")
        .unwrap();
    editor.commit(tx).unwrap();
    assert_eq!(
        editor.document().value()["content"][0]["data"]["ownedRelationship"][0]["eClass"],
        "sysml:OwningMembership"
    );
}
#[test]
fn unknown_prefix_enum_or_missing_parent_never_advertise_creation() {
    for kind in [
        "foreign:ActionUsage",
        "sysml:Unknown",
        "sysml:VisibilityKind",
    ] {
        let doc = imported(kind);
        assert!(!doc.can_create_in("parent"));
        let mut editor = Editor::new(doc);
        let mut tx = editor.begin();
        assert!(
            tx.create_part_usage("parent", ids(1), ids(2), "child")
                .is_err()
        );
        assert!(editor.commit(tx).is_err());
    }
    assert!(!imported("sysml:ActionUsage").can_create_in("missing"));
}
#[test]
fn source_subtypes_are_accepted_for_typed_part_endpoints() {
    let mut value = imported("sysml:ConnectionUsage").value().clone();
    value["content"].as_array_mut().unwrap().push(json!({"id":"definition-resource","eClass":"sysml:ConnectionDefinition","data":{"elementId":"definition","declaredName":"Connection"}}));
    let mut editor = Editor::new(Document::from_json(&value.to_string()).unwrap());
    assert!(
        editor
            .document()
            .is_instance_of("parent", ClassifierId::PartUsage)
    );
    assert!(
        editor
            .document()
            .is_instance_of("definition", ClassifierId::PartDefinition)
    );
    let mut tx = editor.begin();
    tx.type_part_usage("parent", "definition", ids(1)).unwrap();
    editor.commit(tx).unwrap();
    assert_eq!(
        editor.document().value()["content"][0]["data"]["ownedRelationship"][0]["data"]["type"],
        "definition"
    );
}
