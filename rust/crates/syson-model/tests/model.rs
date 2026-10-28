// Copyright (c) 2024, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
// Upstream fixture from SysMLv2-SimpleProject.sql, retained under root LICENSE/NOTICE.
use serde_json::{Value, json};
use syson_model::*;
const FIXTURE: &str = include_str!("fixtures/simple-project.emf.json");
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
fn setup() -> Editor {
    let mut editor = Editor::new(Document::empty());
    let mut tx = editor.begin();
    tx.create_package(None, ids(1), None, "Vehicle").unwrap();
    tx.create_part_definition(&eid(1), ids(2), ids(3), "Wheel")
        .unwrap();
    tx.create_part_usage(&eid(1), ids(4), ids(5), "wheel")
        .unwrap();
    tx.type_part_usage(&eid(4), &eid(2), ids(6)).unwrap();
    assert!(editor.commit(tx).unwrap());
    editor
}
#[test]
fn exact_upstream_emf_sql_value_roundtrip() {
    let document = Document::from_json(FIXTURE).unwrap();
    let original: Value = serde_json::from_str(FIXTURE).unwrap();
    assert_eq!(document.value(), &original);
    assert_eq!(
        Document::from_json(&document.to_json().unwrap()).unwrap(),
        document
    );
    let summaries = document.elements();
    assert_eq!(summaries.len(), 11);
    assert!(
        summaries
            .iter()
            .any(|e| e.element_id == "d99634bb-d4e0-4afa-bbe4-ee553005937d"
                && e.resource_id == "a4f51a38-bfeb-4e0d-a870-55f8fe90405e"
                && e.class == "PartUsage")
    );
}
#[test]
fn unknown_fields_and_large_numbers_survive_rename() {
    let mut v: Value = serde_json::from_str(FIXTURE).unwrap();
    let huge: Value =
        serde_json::from_str("12345678901234567890123456789012345678901234567890").unwrap();
    v["extension"] = json!({"unknown":[null,true,{"token":"é😀"}]});
    v["content"][0]["data"]["unrecognizedVendorField"] = huge.clone();
    let mut editor = Editor::new(Document::from_json(&v.to_string()).unwrap());
    let mut tx = editor.begin();
    tx.rename("d99634bb-d4e0-4afa-bbe4-ee553005937d", "renamed")
        .unwrap();
    editor.commit(tx).unwrap();
    let recovered = Document::from_json(&editor.document().to_json().unwrap()).unwrap();
    assert_eq!(recovered.value()["extension"], v["extension"]);
    assert_eq!(
        recovered.value()["content"][0]["data"]["unrecognizedVendorField"],
        huge
    );
    assert_eq!(
        recovered
            .element("d99634bb-d4e0-4afa-bbe4-ee553005937d")
            .unwrap()["id"],
        "a4f51a38-bfeb-4e0d-a870-55f8fe90405e"
    );
}
#[test]
fn fixture_projection_uses_semantic_owner_and_stable_ids() {
    let document = Document::from_json(FIXTURE).unwrap();
    let projection = document.projection().unwrap();
    let id = projection
        .model
        .find_by_name_and_type("Package1::p", |e| e.class == "PartUsage")
        .unwrap()
        .unwrap();
    assert_eq!(
        projection.stable_id(id),
        Some("d99634bb-d4e0-4afa-bbe4-ee553005937d")
    );
    assert_eq!(
        projection.model.qualified_name(id).unwrap(),
        Some("Package1::p".into())
    );
}
#[test]
fn create_real_package_part_definition_usage_and_typing() {
    let editor = setup();
    let doc = editor.document();
    assert_eq!(doc.elements().len(), 6);
    assert_eq!(doc.element(&eid(1)).unwrap()["eClass"], "sysml:Package");
    assert_eq!(
        doc.element(&eid(3)).unwrap()["eClass"],
        "sysml:OwningMembership"
    );
    assert_eq!(
        doc.element(&eid(5)).unwrap()["data"]["ownedRelatedElement"][0]["data"]["elementId"],
        eid(4)
    );
    assert_eq!(doc.element(&eid(4)).unwrap()["data"]["isComposite"], true);
    assert_eq!(
        doc.element(&eid(6)).unwrap()["eClass"],
        "sysml:FeatureTyping"
    );
    assert_eq!(doc.element(&eid(6)).unwrap()["data"]["type"], eid(2));
    assert_eq!(
        doc.element(&eid(6)).unwrap()["data"]["typedFeature"],
        eid(4)
    );
    assert_eq!(Document::from_json(&doc.to_json().unwrap()).unwrap(), *doc);
}
#[test]
fn feature_membership_only_for_feature_in_type() {
    let mut editor = setup();
    let mut tx = editor.begin();
    tx.create_part_usage(&eid(2), ids(7), ids(8), "nestedWheel")
        .unwrap();
    tx.create_part_definition(&eid(2), ids(9), ids(10), "NestedDefinition")
        .unwrap();
    editor.commit(tx).unwrap();
    assert_eq!(
        editor.document().element(&eid(8)).unwrap()["eClass"],
        "sysml:FeatureMembership"
    );
    assert_eq!(
        editor.document().element(&eid(10)).unwrap()["eClass"],
        "sysml:OwningMembership"
    );
}
#[test]
fn staged_edits_are_invisible_until_commit_and_cancel_is_complete() {
    let mut editor = setup();
    let before = editor.document().clone();
    let mut tx = editor.begin();
    tx.rename(&eid(4), "changed").unwrap();
    assert_ne!(tx.document(), editor.document());
    tx.cancel();
    assert_eq!(*editor.document(), before);
    assert!(!editor.commit(editor.begin()).unwrap());
}
#[test]
fn invalid_operation_poisons_whole_transaction_and_preserves_history() {
    let mut editor = setup();
    let before = editor.document().clone();
    let mut tx = editor.begin();
    tx.rename(&eid(4), "staged").unwrap();
    assert_eq!(
        tx.rename("missing", "x"),
        Err(ModelError::MissingElement("missing".into()))
    );
    assert_eq!(editor.commit(tx), Err(ModelError::FailedTransaction));
    assert_eq!(*editor.document(), before);
    assert!(editor.undo());
    assert!(editor.document().elements().is_empty());
    assert!(editor.redo());
    assert_eq!(*editor.document(), before);
}
#[test]
fn undo_redo_preserves_ids_fields_and_new_edit_clears_redo() {
    let mut editor = setup();
    let before = editor.document().clone();
    let mut tx = editor.begin();
    tx.rename(&eid(4), "renamed").unwrap();
    editor.commit(tx).unwrap();
    let after = editor.document().clone();
    assert!(editor.undo());
    assert_eq!(*editor.document(), before);
    assert!(editor.redo());
    assert_eq!(*editor.document(), after);
    assert!(editor.undo());
    let mut tx = editor.begin();
    tx.rename(&eid(4), "branch").unwrap();
    editor.commit(tx).unwrap();
    assert!(!editor.redo());
}
#[test]
fn stale_transactions_rejected_after_commit_and_undo() {
    let mut editor = setup();
    let mut stale = editor.begin();
    let mut next = editor.begin();
    stale.rename(&eid(4), "stale").unwrap();
    next.rename(&eid(4), "next").unwrap();
    editor.commit(next).unwrap();
    assert_eq!(editor.commit(stale), Err(ModelError::StaleTransaction));
    let mut stale = editor.begin();
    stale.rename(&eid(4), "again").unwrap();
    editor.undo();
    assert_eq!(editor.commit(stale), Err(ModelError::StaleTransaction));
}
#[test]
fn duplicate_ids_and_invalid_uuid_rejected_without_partial_state() {
    assert!(matches!(
        NodeIds::new("bad", &eid(1)),
        Err(ModelError::InvalidId(_))
    ));
    let mut editor = setup();
    let before = editor.document().clone();
    let mut tx = editor.begin();
    assert!(matches!(
        tx.create_package(None, ids(1), None, "duplicate"),
        Err(ModelError::DuplicateId(_))
    ));
    assert_eq!(editor.commit(tx), Err(ModelError::FailedTransaction));
    assert_eq!(*editor.document(), before);
}
#[test]
fn wrong_typing_endpoint_and_missing_parent_are_explicit() {
    let editor = setup();
    let mut tx = editor.begin();
    assert_eq!(
        tx.type_part_usage(&eid(4), &eid(1), ids(7)),
        Err(ModelError::WrongType {
            element: eid(1),
            expected: "PartDefinition"
        })
    );
    let mut tx = editor.begin();
    assert_eq!(
        tx.create_part_usage("missing", ids(7), ids(8), "x"),
        Err(ModelError::MissingElement("missing".into()))
    );
}
#[test]
fn malformed_json_flat_rest_and_invalid_containment_rejected() {
    assert!(matches!(Document::from_json("{"), Err(ModelError::Json(_))));
    assert!(matches!(
        Document::from_json("[{\"@id\":\"x\",\"@type\":\"Package\"}]"),
        Err(ModelError::Shape(_))
    ));
    let mut v: Value = serde_json::from_str(FIXTURE).unwrap();
    v["content"][0]["data"]["ownedRelationship"] = json!(false);
    assert!(matches!(
        Document::from_json(&v.to_string()),
        Err(ModelError::Shape(_))
    ));
}
#[test]
fn independent_resource_and_element_duplicate_checks() {
    let mut v: Value = serde_json::from_str(FIXTURE).unwrap();
    let mut other = v["content"][0].clone();
    other["data"] = json!({"elementId":eid(200)});
    v["content"].as_array_mut().unwrap().push(other);
    assert!(matches!(
        Document::from_json(&v.to_string()),
        Err(ModelError::DuplicateId(_))
    ));
}
#[test]
fn stable_ids_never_use_process_local_indices() {
    let editor = setup();
    let projection = editor.document().projection().unwrap();
    let handle = projection.handle(&eid(4)).unwrap();
    assert_eq!(projection.stable_id(handle), Some(eid(4).as_str()));
    let summary = editor
        .document()
        .elements()
        .into_iter()
        .find(|e| e.element_id == eid(4))
        .unwrap();
    assert_eq!(summary.owner, Some(eid(1)));
    assert_ne!(summary.resource_id, summary.element_id);
}

#[test]
fn transaction_cannot_commit_to_equivalent_other_editor() {
    let source = Editor::new(Document::empty());
    let mut destination = Editor::new(Document::empty());
    let mut tx = source.begin();
    tx.create_package(None, ids(1), None, "foreign").unwrap();
    assert_eq!(destination.commit(tx), Err(ModelError::ForeignTransaction));
    assert!(destination.document().elements().is_empty());
    assert!(!destination.undo());
}
