// SPDX-License-Identifier: EPL-2.0
//! Local workflow coverage, not upstream-runtime or complete EMF acceptance.
#![cfg(feature = "raw-factory-candidate")]

use std::{
    cell::Cell,
    path::PathBuf,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};
use syson_metamodel::ClassifierId as C;
use syson_model::{Document, Editor, NodeIds};
use syson_workflow::{SnapshotStore, stage_initialized_creation};

fn ids(n: u32) -> NodeIds {
    NodeIds::new(
        &format!("00000000-0000-4000-8000-{n:012x}"),
        &format!("10000000-0000-4000-8000-{n:012x}"),
    )
    .unwrap()
}
fn eid(n: u32) -> String {
    ids(n).element
}
fn pool(first: u32) -> Vec<NodeIds> {
    (first..first + 20).map(ids).collect()
}
fn base() -> Document {
    let mut editor = Editor::new(Document::empty());
    let mut tx = editor.begin();
    tx.create_package(None, ids(1), None, "Vehicle").unwrap();
    editor.commit(tx).unwrap();
    editor.document().clone()
}
fn classes() -> [(C, &'static str, usize); 4] {
    [
        (C::ConnectionDefinition, "ConnectionDefinition1", 6),
        (C::InterfaceDefinition, "InterfaceDefinition1", 10),
        (C::AcceptActionUsage, "acceptAction1", 6),
        (C::PortDefinition, "PortDefinition1", 5),
    ]
}
fn assert_shape(document: &Document, root_id: &str, class: C) {
    let root = document.element(root_id).unwrap();
    let members = root["data"]["ownedRelationship"].as_array().unwrap();
    match class {
        C::ConnectionDefinition | C::InterfaceDefinition => {
            assert_eq!(root["data"]["isSufficient"], true);
            let expected = if class == C::InterfaceDefinition {
                vec![
                    ("PortUsage", "source"),
                    ("PortUsage", "target"),
                    ("ReferenceUsage", "source"),
                    ("ReferenceUsage", "target"),
                ]
            } else {
                vec![("ReferenceUsage", "source"), ("ReferenceUsage", "target")]
            };
            assert_eq!(members.len(), expected.len());
            for (member, (class, name)) in members.iter().zip(expected) {
                assert_eq!(member["eClass"], "sysml:FeatureMembership");
                let children = member["data"]["ownedRelatedElement"].as_array().unwrap();
                assert_eq!(children.len(), 1);
                let child = &children[0];
                assert_eq!(child["eClass"], format!("sysml:{class}"));
                assert_eq!(child["data"]["declaredName"], name);
                assert_eq!(child["data"]["isEnd"], true);
                if class == "PortUsage" {
                    assert!(child["data"].get("isComposite").is_none());
                } else {
                    assert_eq!(child["data"]["isComposite"], false);
                }
            }
        }
        C::AcceptActionUsage => {
            assert_eq!(root["data"]["isComposite"], true);
            assert_eq!(members.len(), 2);
            for (member, (name, direction)) in members
                .iter()
                .zip([("payload", "inout"), ("receiver", "in")])
            {
                assert_eq!(member["eClass"], "sysml:ParameterMembership");
                let children = member["data"]["ownedRelatedElement"].as_array().unwrap();
                assert_eq!(children.len(), 1);
                let child = &children[0];
                assert_eq!(child["eClass"], "sysml:ReferenceUsage");
                assert_eq!(child["data"]["declaredName"], name);
                assert_eq!(child["data"]["direction"], direction);
                assert!(child["data"].get("isComposite").is_none());
                assert!(child["data"].get("isEnd").is_none());
            }
        }
        C::PortDefinition => {
            assert_eq!(members.len(), 1);
            assert_eq!(members[0]["eClass"], "sysml:OwningMembership");
            let children = members[0]["data"]["ownedRelatedElement"]
                .as_array()
                .unwrap();
            assert_eq!(children.len(), 1);
            let conjugated = &children[0];
            assert_eq!(conjugated["eClass"], "sysml:ConjugatedPortDefinition");
            assert!(conjugated["data"].get("declaredName").is_none());
            let relationships = conjugated["data"]["ownedRelationship"].as_array().unwrap();
            assert_eq!(relationships.len(), 1);
            let reference = &relationships[0];
            assert_eq!(reference["eClass"], "sysml:PortConjugation");
            assert_eq!(
                reference["data"]["conjugatedType"],
                conjugated["data"]["elementId"]
            );
            assert_eq!(reference["data"]["originalPortDefinition"], root_id);
            for unsupported in ["originalType", "conjugatedPortDefinition", "owningType"] {
                assert!(reference["data"].get(unsupported).is_none());
            }
        }
        _ => unreachable!(),
    }
}
fn editor_with_redo() -> (Editor, Document, Document) {
    let mut editor = Editor::new(base());
    let original = editor.document().clone();
    let mut prior = editor.begin();
    prior.rename(&eid(1), "EarlierVehicle").unwrap();
    editor.commit(prior).unwrap();
    let redo = editor.document().clone();
    assert!(editor.undo());
    (editor, original, redo)
}
fn assert_failure_preserves_history(mut editor: Editor, original: &Document, redo: &Document) {
    assert_eq!(editor.document(), original);
    assert!(
        editor.redo(),
        "failed creation must preserve the previous redo"
    );
    assert_eq!(editor.document(), redo);
    assert!(!editor.redo());
    assert!(editor.undo());
    assert_eq!(editor.document(), original);
    assert!(!editor.undo(), "failed creation must add no undo entry");
}

#[test]
fn four_initialized_classes_compose_with_package_creation_in_one_transaction() {
    let mut editor = Editor::new(Document::empty());
    let before = editor.document().clone();
    let mut tx = editor.begin();
    tx.create_package(None, ids(1), None, "Vehicle").unwrap();
    for (i, (class, default_name, consumed)) in classes().into_iter().enumerate() {
        let first = 10 + i as u32 * 30;
        let (next, report) =
            stage_initialized_creation(tx, class, &eid(1), None, pool(first), || false).unwrap();
        tx = next;
        assert_eq!(report.classifier, class);
        assert_eq!(report.element_id, eid(first + 1));
        assert_eq!(report.identities_consumed, consumed);
        assert_eq!(report.membership_class, C::OwningMembership);
        assert_ne!(report.root.resource_id, report.root.semantic_id);
        assert_eq!(
            tx.document().element(&report.element_id).unwrap()["data"]["declaredName"],
            default_name
        );
        assert_shape(tx.document(), &report.element_id, class);
        assert_eq!(
            editor.document(),
            &before,
            "staging must not publish any root or helper"
        );
    }
    assert!(editor.commit(tx).unwrap());
    let after = editor.document().clone();
    assert!(editor.undo());
    assert_eq!(editor.document(), &before);
    assert!(
        !editor.undo(),
        "the entire workflow must have one history entry"
    );
    assert!(editor.redo());
    assert_eq!(editor.document(), &after);
}

#[test]
fn default_names_count_exact_siblings_and_explicit_names_keep_initialized_structure() {
    for (class, first_default, _) in classes() {
        let mut editor = Editor::new(base());
        let (tx, first) =
            stage_initialized_creation(editor.begin(), class, &eid(1), None, pool(10), || false)
                .unwrap();
        let (tx, second) =
            stage_initialized_creation(tx, class, &eid(1), None, pool(40), || false).unwrap();
        let (tx, explicit) =
            stage_initialized_creation(tx, class, &eid(1), Some("chosen name"), pool(70), || false)
                .unwrap();
        assert_eq!(
            tx.document().element(&first.element_id).unwrap()["data"]["declaredName"],
            first_default
        );
        let second_default = format!("{}2", first_default.strip_suffix('1').unwrap());
        assert_eq!(
            tx.document().element(&second.element_id).unwrap()["data"]["declaredName"],
            second_default
        );
        assert_eq!(
            tx.document().element(&explicit.element_id).unwrap()["data"]["declaredName"],
            "chosen name"
        );
        assert_shape(tx.document(), &explicit.element_id, class);
        editor.commit(tx).unwrap();
    }
}

#[test]
fn early_and_mid_constructor_cancellation_keep_document_and_redo() {
    for (class, _, required) in classes() {
        for after in [0, required - 1] {
            let (editor, original, redo) = editor_with_redo();
            let count = Rc::new(Cell::new(0));
            let consumed = count.clone();
            let checked = count.clone();
            let identities = pool(10)
                .into_iter()
                .inspect(move |_| consumed.set(consumed.get() + 1));
            let result = stage_initialized_creation(
                editor.begin(),
                class,
                &eid(1),
                None,
                identities,
                move || checked.get() >= after,
            );
            assert!(result.is_err());
            assert_eq!(
                count.get(),
                after,
                "cancel must not exhaust the remaining identity provider"
            );
            assert_failure_preserves_history(editor, &original, &redo);
        }
    }
}

#[test]
fn exhausted_identity_provider_never_publishes_partial_construction() {
    for (class, _, required) in classes() {
        for available in [0, 1, required - 1] {
            let (editor, original, redo) = editor_with_redo();
            let result = stage_initialized_creation(
                editor.begin(),
                class,
                &eid(1),
                None,
                pool(10).into_iter().take(available),
                || false,
            );
            assert!(result.is_err());
            assert_failure_preserves_history(editor, &original, &redo);
        }
    }
}

#[test]
fn existing_and_new_duplicate_semantic_or_resource_ids_preserve_document_and_redo() {
    for (class, _, required) in classes() {
        for duplicate_semantic in [true, false] {
            for existing in [true, false] {
                let (editor, original, redo) = editor_with_redo();
                let mut identities = pool(10);
                let duplicate = ids(if existing { 1 } else { 10 });
                if duplicate_semantic {
                    identities[required - 1].element = duplicate.element;
                } else {
                    identities[required - 1].resource = duplicate.resource;
                }
                let result = stage_initialized_creation(
                    editor.begin(),
                    class,
                    &eid(1),
                    None,
                    identities,
                    || false,
                );
                assert!(
                    result.is_err(),
                    "class={class:?}, semantic={duplicate_semantic}, existing={existing}"
                );
                assert_failure_preserves_history(editor, &original, &redo);
            }
        }
    }
}

#[test]
fn invalid_requests_do_not_consume_identity_provider_or_change_history() {
    for (class, parent, name) in [
        (C::PartDefinition, eid(1), None),
        (C::PortDefinition, eid(999), None),
        (C::PortDefinition, eid(1), Some("")),
    ] {
        let (editor, original, redo) = editor_with_redo();
        let identities = std::iter::from_fn(|| -> Option<NodeIds> {
            panic!("invalid request consumed an identity")
        });
        assert!(
            stage_initialized_creation(editor.begin(), class, &parent, name, identities, || false)
                .is_err()
        );
        assert_failure_preserves_history(editor, &original, &redo);
    }
}

#[test]
fn feature_children_use_feature_membership_inside_type_parents() {
    let mut editor = Editor::new(base());
    let mut tx = editor.begin();
    tx.create_part_definition(&eid(1), ids(2), ids(3), "Controller")
        .unwrap();
    let (tx, action) = stage_initialized_creation(
        tx,
        C::AcceptActionUsage,
        &eid(2),
        Some("receive"),
        pool(10),
        || false,
    )
    .unwrap();
    let (tx, port) = stage_initialized_creation(
        tx,
        C::PortDefinition,
        &eid(2),
        Some("Power"),
        pool(40),
        || false,
    )
    .unwrap();
    assert_eq!(action.membership_class, C::FeatureMembership);
    assert_eq!(port.membership_class, C::OwningMembership);
    let parent = tx.document().element(&eid(2)).unwrap();
    let memberships = parent["data"]["ownedRelationship"].as_array().unwrap();
    assert_eq!(memberships[0]["eClass"], "sysml:FeatureMembership");
    assert_eq!(memberships[1]["eClass"], "sysml:OwningMembership");
    editor.commit(tx).unwrap();
    let projection = editor.document().projection().unwrap();
    let handle = projection
        .model
        .find_by_name_and_type("Vehicle::Controller::receive", |e| {
            e.class == "AcceptActionUsage"
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        projection.stable_id(handle),
        Some(action.element_id.as_str())
    );
}

#[test]
fn persisted_restart_retains_all_initialized_graphs_stable_ids_and_local_port_references() {
    let mut editor = Editor::new(base());
    let mut created = Vec::new();
    let mut tx = editor.begin();
    for (i, (class, _, _)) in classes().into_iter().enumerate() {
        let name = ["Connection", "Interface", "Accept", "Power"][i];
        let (next, report) = stage_initialized_creation(
            tx,
            class,
            &eid(1),
            Some(name),
            pool(10 + i as u32 * 30),
            || false,
        )
        .unwrap();
        tx = next;
        created.push((class, name, report.element_id));
    }
    editor.commit(tx).unwrap();
    let root = PathBuf::from(
        std::env::var_os("SYSON_TEST_DIR")
            .expect("Set SYSON_TEST_DIR on the approved build volume"),
    )
    .join(format!(
        "initialized-workflow-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    SnapshotStore::open(&root)
        .unwrap()
        .save_new("all-initialized", editor.document())
        .unwrap();
    let recovered = SnapshotStore::open(&root)
        .unwrap()
        .load("all-initialized")
        .unwrap();
    assert_eq!(&recovered, editor.document());
    let reserialized = Document::from_json(&recovered.to_json().unwrap()).unwrap();
    assert_eq!(reserialized, recovered);
    let projection = recovered.projection().unwrap();
    for (class, name, semantic) in created {
        assert_shape(&recovered, &semantic, class);
        let found = projection
            .model
            .find_by_name_and_type(&format!("Vehicle::{name}"), |e| {
                e.class == class.metadata().name()
            })
            .unwrap()
            .unwrap();
        assert_eq!(projection.stable_id(found), Some(semantic.as_str()));
    }
    let mut restarted = Editor::new(recovered);
    assert!(
        !restarted.undo(),
        "snapshots persist documents, not history"
    );
    let (tx, next) = stage_initialized_creation(
        restarted.begin(),
        C::PortDefinition,
        &eid(1),
        None,
        pool(150),
        || false,
    )
    .unwrap();
    assert_eq!(
        tx.document().element(&next.element_id).unwrap()["data"]["declaredName"],
        "PortDefinition2"
    );
    restarted.commit(tx).unwrap();
    assert!(restarted.undo());
    assert_eq!(restarted.document(), &reserialized);
}
