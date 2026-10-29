// Copyright (c) 2026 SysON Rust port contributors.
// SPDX-License-Identifier: EPL-2.0
// Bounded raw append regressions; no inverse ownership or native parity claim.
use serde_json::{Value, json};
use syson_model::{ContainmentFeature as F, Document, Editor, ModelError};

fn id(n: u32) -> String {
    format!("a0000000-0000-4000-8000-{n:012x}")
}
fn node(n: u32, class: &str, extra: Value) -> Value {
    let mut data = extra.as_object().unwrap().clone();
    data.insert("elementId".into(), json!(id(n)));
    json!({"id":id(n + 100),"eClass":format!("sysml:{class}"),"data":data})
}
fn document(root: Value) -> Document {
    Document::from_json(
        &json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[root]}).to_string(),
    )
    .unwrap()
}
fn base() -> Document {
    document(node(1, "Package", json!({"declaredName":"base"})))
}
fn with_redo(document: Document) -> Editor {
    let mut editor = Editor::new(document);
    let mut tx = editor.begin();
    tx.rename(&id(1), "retained-redo").unwrap();
    editor.commit(tx).unwrap();
    assert!(editor.undo());
    editor
}
fn assert_retained_redo(editor: &mut Editor) {
    assert!(editor.redo());
    assert_eq!(
        editor.document().element(&id(1)).unwrap()["data"]["declaredName"],
        "retained-redo"
    );
    assert!(!editor.redo());
}

#[test]
fn missing_parent_poison_preserves_prior_staged_value_and_live_redo() {
    let mut editor = with_redo(base());
    let before = editor.document().clone();
    let mut tx = editor.begin();
    tx.rename(&id(1), "staged").unwrap();
    let staged = tx.document().clone();
    assert_eq!(
        tx.attach_preallocated_subtree(
            &id(999),
            F::OwnedRelationship,
            node(2, "OwningMembership", json!({})),
        ),
        Err(ModelError::MissingElement(id(999)))
    );
    assert_eq!(tx.document(), &staged);
    assert_eq!(
        tx.rename(&id(1), "after-failure"),
        Err(ModelError::FailedTransaction)
    );
    assert_eq!(editor.commit(tx), Err(ModelError::FailedTransaction));
    assert_eq!(editor.document(), &before);
    assert_retained_redo(&mut editor);
}

#[test]
fn nested_resource_and_semantic_collisions_against_existing_descendants_are_atomic() {
    for semantic in [true, false] {
        let existing = node(
            2,
            "OwningMembership",
            json!({
                "ownedRelatedElement":[node(3, "PartUsage", json!({}))]
            }),
        );
        let mut editor = with_redo(document(node(
            1,
            "Package",
            json!({
                "declaredName":"base", "ownedRelationship":[existing]
            }),
        )));
        let before = editor.document().clone();
        let mut leaf = node(5, "PartUsage", json!({}));
        if semantic {
            leaf["data"]["elementId"] = json!(id(3).to_uppercase());
        } else {
            leaf["id"] = json!(id(103).to_uppercase());
        }
        let mut tx = editor.begin();
        tx.rename(&id(1), "staged").unwrap();
        let staged = tx.document().clone();
        assert!(matches!(
            tx.attach_preallocated_subtree(
                &id(1),
                F::OwnedRelationship,
                node(4, "OwningMembership", json!({"ownedRelatedElement":[leaf]})),
            ),
            Err(ModelError::DuplicateId(_))
        ));
        assert_eq!(tx.document(), &staged);
        assert_eq!(editor.commit(tx), Err(ModelError::FailedTransaction));
        assert_eq!(editor.document(), &before);
        assert_retained_redo(&mut editor);
    }
}

#[test]
fn malformed_fragment_shapes_do_not_publish_an_earlier_valid_attachment() {
    let mut missing_class = node(4, "OwningMembership", json!({}));
    missing_class.as_object_mut().unwrap().remove("eClass");
    let mut nonstring_class = node(4, "OwningMembership", json!({}));
    nonstring_class["eClass"] = json!(42);
    let mut nonobject_data = node(4, "OwningMembership", json!({}));
    nonobject_data["data"] = json!([]);
    let mut missing_resource = node(4, "OwningMembership", json!({}));
    missing_resource.as_object_mut().unwrap().remove("id");
    let mut nonstring_element = node(4, "OwningMembership", json!({}));
    nonstring_element["data"]["elementId"] = json!(false);
    for malformed in [
        Value::Null,
        missing_class,
        nonstring_class,
        nonobject_data,
        missing_resource,
        nonstring_element,
        node(4, "OwningMembership", json!({"ownedRelationship":{}})),
        node(4, "OwningMembership", json!({"ownedRelatedElement":null})),
    ] {
        let mut editor = with_redo(base());
        let before = editor.document().clone();
        let mut tx = editor.begin();
        tx.attach_preallocated_subtree(
            &id(1),
            F::OwnedRelationship,
            node(2, "OwningMembership", json!({})),
        )
        .unwrap();
        let staged = tx.document().clone();
        assert!(matches!(
            tx.attach_preallocated_subtree(&id(1), F::OwnedRelationship, malformed),
            Err(ModelError::Shape(_))
        ));
        assert_eq!(tx.document(), &staged);
        assert_eq!(editor.commit(tx), Err(ModelError::FailedTransaction));
        assert_eq!(editor.document(), &before);
        assert_retained_redo(&mut editor);
    }
}

#[test]
fn duplicate_resource_within_fragment_and_across_staged_attachments_is_refused() {
    for internal in [true, false] {
        let mut editor = with_redo(base());
        let before = editor.document().clone();
        let mut tx = editor.begin();
        let mut leaf = node(4, "PartUsage", json!({}));
        let fragment = if internal {
            leaf["id"] = json!(id(103).to_uppercase());
            node(
                2,
                "OwningMembership",
                json!({"ownedRelatedElement":[
                    node(3, "PartUsage", json!({})), leaf
                ]}),
            )
        } else {
            tx.attach_preallocated_subtree(
                &id(1),
                F::OwnedRelationship,
                node(2, "OwningMembership", json!({})),
            )
            .unwrap();
            leaf["id"] = json!(id(102).to_uppercase());
            node(3, "OwningMembership", json!({"ownedRelatedElement":[leaf]}))
        };
        let staged = tx.document().clone();
        assert!(matches!(
            tx.attach_preallocated_subtree(&id(1), F::OwnedRelationship, fragment),
            Err(ModelError::DuplicateId(_))
        ));
        assert_eq!(tx.document(), &staged);
        assert_eq!(editor.commit(tx), Err(ModelError::FailedTransaction));
        assert_eq!(editor.document(), &before);
        assert_retained_redo(&mut editor);
    }
}

#[test]
fn foreign_editor_refuses_attached_transaction_even_when_base_values_match() {
    let origin = Editor::new(base());
    let mut destination = with_redo(base());
    assert_eq!(origin.document(), destination.document());
    let before = destination.document().clone();
    let mut tx = origin.begin();
    tx.attach_preallocated_subtree(
        &id(1),
        F::OwnedRelationship,
        node(2, "OwningMembership", json!({})),
    )
    .unwrap();
    assert_eq!(destination.commit(tx), Err(ModelError::ForeignTransaction));
    assert_eq!(destination.document(), &before);
    assert_eq!(origin.document(), &before);
    assert_retained_redo(&mut destination);
}

#[test]
fn unknown_parent_classifier_refuses_attachment_and_malformed_parent_arrays_refuse_load() {
    for class in ["foreign:Package", "sysml:UnknownClass"] {
        let mut parent = node(2, "OwningMembership", json!({}));
        parent["eClass"] = json!(class);
        let mut editor = with_redo(document(node(
            1,
            "Package",
            json!({
                "declaredName":"base", "ownedRelationship":[parent]
            }),
        )));
        let before = editor.document().clone();
        let mut tx = editor.begin();
        assert_eq!(
            tx.attach_preallocated_subtree(
                &id(2),
                F::OwnedRelationship,
                node(3, "OwningMembership", json!({})),
            ),
            Err(ModelError::UnsupportedClass(class.into()))
        );
        assert_eq!(tx.document(), &before);
        assert_eq!(editor.commit(tx), Err(ModelError::FailedTransaction));
        assert_eq!(editor.document(), &before);
        assert_retained_redo(&mut editor);
    }
    // Document is immutable through its public value view. Its loader rejects
    // malformed parent arrays, so that attachment error branch is unreachable
    // from an admitted Document; do not introduce a semantic bypass to test it.
    for feature in ["ownedRelationship", "ownedRelatedElement"] {
        for malformed in [Value::Null, json!({}), json!("children")] {
            let mut root = node(1, "Package", json!({}));
            root["data"][feature] = malformed;
            let wire = json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[root]}).to_string();
            assert!(matches!(
                Document::from_json(&wire),
                Err(ModelError::Shape(_))
            ));
        }
    }
}
