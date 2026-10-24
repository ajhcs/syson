use serde_json::{Value,json};
use syson_model::{ContainmentFeature as F,Document,Editor,ModelError};
fn id(n:u32)->String {format!("a0000000-0000-4000-8000-{n:012x}")}
fn node(n:u32,class:&str,extra:Value)->Value {
    let mut data=extra.as_object().unwrap().clone();data.insert("elementId".into(),json!(id(n)));
    json!({"id":id(n+100),"eClass":format!("sysml:{class}"),"data":data})
}
fn base()->Document {
    Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[node(1,"Package",json!({"declaredName":"base"}))]}).to_string()).unwrap()
}
fn subtree()->Value {node(2,"OwningMembership",json!({"ownedRelatedElement":[node(3,"PartUsage",json!({"unknownScalar":{"retained":true}}))]}))}
#[test]
fn appends_preallocated_subtree_in_one_undoable_transaction_without_rewriting_ids_or_unknown_fields() {
    let doc=base();let before=doc.to_json().unwrap();let mut editor=Editor::new(doc);let fragment=subtree();
    let mut tx=editor.begin();tx.attach_preallocated_subtree(&id(1),F::OwnedRelationship,fragment.clone()).unwrap();
    assert_eq!(editor.document().to_json().unwrap(),before);
    assert!(editor.commit(tx).unwrap());
    assert_eq!(editor.document().element(&id(1)).unwrap()["data"]["ownedRelationship"][0],fragment);
    let after=editor.document().to_json().unwrap();
    assert!(editor.undo());assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
    assert!(editor.redo());assert_eq!(editor.document().to_json().unwrap(),after);
}
#[test]
fn supports_owned_related_element_on_relationship_and_preserves_append_order() {
    let mut editor=Editor::new(base());let mut tx=editor.begin();
    tx.attach_preallocated_subtree(&id(1),F::OwnedRelationship,node(2,"OwningMembership",json!({}))).unwrap();
    tx.attach_preallocated_subtree(&id(2),F::OwnedRelatedElement,node(3,"PartUsage",json!({}))).unwrap();
    tx.attach_preallocated_subtree(&id(2),F::OwnedRelatedElement,node(4,"PartUsage",json!({}))).unwrap();
    editor.commit(tx).unwrap();
    let children=editor.document().element(&id(2)).unwrap()["data"]["ownedRelatedElement"].as_array().unwrap();
    assert_eq!(children[0]["data"]["elementId"],id(3));assert_eq!(children[1]["data"]["elementId"],id(4));
}
#[test]
fn duplicate_semantic_or_case_folded_resource_id_poison_transaction_without_commit() {
    for semantic in [true,false] {
        let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();
        let mut child=node(2,"OwningMembership",json!({}));
        if semantic {child["data"]["elementId"]=json!(id(1));} else {child["id"]=json!(id(101).to_uppercase());}
        let mut tx=editor.begin();assert!(matches!(tx.attach_preallocated_subtree(&id(1),F::OwnedRelationship,child),Err(ModelError::DuplicateId(_))));
        assert!(matches!(editor.commit(tx),Err(ModelError::FailedTransaction)));
        assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
    }
}
#[test]
fn rejects_invalid_parent_child_and_nested_containment() {
    for (feature,child) in [
        (F::OwnedRelatedElement,node(2,"PartUsage",json!({}))),
        (F::OwnedRelationship,node(2,"PartUsage",json!({}))),
        (F::OwnedRelationship,node(2,"OwningMembership",json!({"ownedRelatedElement":[node(3,"PartUsage",json!({"ownedRelationship":[node(4,"PartUsage",json!({}))]}))]}))),
    ] {
        let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut tx=editor.begin();
        assert!(matches!(tx.attach_preallocated_subtree(&id(1),feature,child),Err(ModelError::WrongType {..})));
        assert!(matches!(editor.commit(tx),Err(ModelError::FailedTransaction)));assert_eq!(editor.document().to_json().unwrap(),before);
    }
}
#[test]
fn malformed_identity_unknown_class_abstract_class_and_duplicate_nested_ids_are_rejected() {
    let mut invalid_id=node(2,"OwningMembership",json!({}));invalid_id["id"]=json!("not-an-id");
    let unknown=node(2,"UnknownClass",json!({}));let abstract_class=node(2,"Element",json!({}));
    let duplicate=node(2,"OwningMembership",json!({"ownedRelatedElement":[node(3,"PartUsage",json!({})),node(3,"PartUsage",json!({}))]}));
    for child in [invalid_id,unknown,abstract_class,duplicate] {
        let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut tx=editor.begin();
        assert!(tx.attach_preallocated_subtree(&id(1),F::OwnedRelationship,child).is_err());
        assert!(matches!(editor.commit(tx),Err(ModelError::FailedTransaction)));assert_eq!(editor.document().to_json().unwrap(),before);
    }
}
#[test]
fn cancellation_discards_staged_containment_and_preserves_history() {
    let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut tx=editor.begin();
    tx.attach_preallocated_subtree(&id(1),F::OwnedRelationship,subtree()).unwrap();tx.cancel();
    assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());assert!(!editor.redo());
}
#[test]
fn late_failure_discards_prior_scalar_change_and_preserves_redo() {
    let mut editor=Editor::new(base());let mut first=editor.begin();first.rename(&id(1),"first").unwrap();editor.commit(first).unwrap();assert!(editor.undo());
    let before=editor.document().to_json().unwrap();let mut tx=editor.begin();tx.rename(&id(1),"uncommitted").unwrap();
    let mut child=subtree();child["id"]=json!(id(101));
    assert!(tx.attach_preallocated_subtree(&id(1),F::OwnedRelationship,child).is_err());
    assert!(matches!(editor.commit(tx),Err(ModelError::FailedTransaction)));
    assert_eq!(editor.document().to_json().unwrap(),before);assert!(editor.redo());
    assert_eq!(editor.document().element(&id(1)).unwrap()["data"]["declaredName"],"first");
}
#[test]
fn stale_attachment_cannot_replace_intervening_editor_document() {
    let mut editor=Editor::new(base());let mut stale=editor.begin();
    stale.attach_preallocated_subtree(&id(1),F::OwnedRelationship,subtree()).unwrap();
    let mut current=editor.begin();current.rename(&id(1),"current").unwrap();editor.commit(current).unwrap();
    let before=editor.document().to_json().unwrap();
    assert!(matches!(editor.commit(stale),Err(ModelError::StaleTransaction)));
    assert_eq!(editor.document().to_json().unwrap(),before);assert!(editor.document().element(&id(2)).is_none());
}
