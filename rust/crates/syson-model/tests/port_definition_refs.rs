// SPDX-License-Identifier: EPL-2.0
#![cfg(feature="raw-factory-candidate")]
use serde_json::{json,Value};
use syson_model::{Document,Editor,ModelError,PortDefinitionReference as R};
fn id(n:u32)->String {format!("00000000-0000-4000-8000-{n:012x}")}
fn node(n:u32,class:&str,data:Value)->Value {
    let mut data=data.as_object().unwrap().clone();data.insert("elementId".into(),json!(id(n)));
    json!({"id":id(n+100),"eClass":format!("sysml:{class}"),"data":data})
}
fn base()->Document {
    let relation=|member,port,conjugation|node(member,"OwningMembership",json!({"ownedRelatedElement":[
        node(port,"ConjugatedPortDefinition",json!({"ownedRelationship":[node(conjugation,"PortConjugation",json!({}))]}))
    ]}));
    Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[
        node(1,"Package",json!({"declaredName":"base","ownedRelationship":[
            node(2,"OwningMembership",json!({"ownedRelatedElement":[node(3,"PortDefinition",json!({"declaredName":"other"}))]})),
            node(4,"OwningMembership",json!({"ownedRelatedElement":[node(5,"PortDefinition",json!({"declaredName":"root","ownedRelationship":[relation(6,7,8),relation(12,13,14)]}))]}))
        ]}))
    ]}).to_string()).unwrap()
}
#[test]
fn local_reference_bindings_succeed_with_semantic_ids_and_one_history_commit() {
    let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut tx=editor.begin();
    tx.initialize_port_definition_reference(&id(5),&id(8),R::ConjugatedType,&id(7)).unwrap();
    tx.initialize_port_definition_reference(&id(5),&id(8),R::OriginalPortDefinition,&id(5)).unwrap();
    let data=&tx.document().element(&id(8)).unwrap()["data"];
    assert_eq!(data["conjugatedType"],id(7));assert_eq!(data["originalPortDefinition"],id(5));
    assert_ne!(data["conjugatedType"],id(107));
    for alias in ["originalType","conjugatedPortDefinition","owningType"] {assert!(data.get(alias).is_none());}
    assert!(editor.document().element(&id(8)).unwrap()["data"].get("conjugatedType").is_none());
    assert!(editor.commit(tx).unwrap());let after=editor.document().to_json().unwrap();
    assert!(editor.undo());assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
    assert!(editor.redo());assert_eq!(editor.document().to_json().unwrap(),after);
}
#[test]
fn first_reference_success_then_second_refusal_poison_cannot_commit_and_preserves_redo() {
    let mut editor=Editor::new(base());let mut prior=editor.begin();prior.rename(&id(1),"prior-change").unwrap();editor.commit(prior).unwrap();assert!(editor.undo());
    let before=editor.document().to_json().unwrap();let mut tx=editor.begin();
    tx.initialize_port_definition_reference(&id(5),&id(8),R::ConjugatedType,&id(7)).unwrap();
    assert_eq!(tx.document().element(&id(8)).unwrap()["data"]["conjugatedType"],id(7));
    // A different PortDefinition has the right class but the wrong source role.
    assert!(tx.initialize_port_definition_reference(&id(5),&id(8),R::OriginalPortDefinition,&id(3)).is_err());
    assert!(matches!(editor.commit(tx),Err(ModelError::FailedTransaction)));
    assert_eq!(editor.document().to_json().unwrap(),before);assert!(editor.redo());
    assert_eq!(editor.document().element(&id(1)).unwrap()["data"]["declaredName"],"prior-change");
}
#[test]
fn missing_resource_wrong_type_foreign_role_and_cross_root_targets_are_refused() {
    for (root,source,kind,target) in [
        (5,8,R::ConjugatedType,99),(5,8,R::ConjugatedType,107),
        (5,8,R::ConjugatedType,5),(5,8,R::ConjugatedType,13),
        (5,8,R::OriginalPortDefinition,7),(5,8,R::OriginalPortDefinition,3),
        (3,8,R::ConjugatedType,7),(5,14,R::ConjugatedType,7),
    ] {
        let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut tx=editor.begin();
        assert!(tx.initialize_port_definition_reference(&id(root),&id(source),kind,&id(target)).is_err());
        assert!(matches!(editor.commit(tx),Err(ModelError::FailedTransaction)));
        assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
    }
}
#[test]
fn exact_reference_source_and_root_classes_are_required() {
    for (root,source) in [(1,8),(7,8),(5,7),(5,99)] {
        let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut tx=editor.begin();
        assert!(tx.initialize_port_definition_reference(&id(root),&id(source),R::ConjugatedType,&id(7)).is_err());
        assert!(matches!(editor.commit(tx),Err(ModelError::FailedTransaction)));
        assert_eq!(editor.document().to_json().unwrap(),before);
    }
}
#[test]
fn generic_reference_identity_and_unknown_edits_remain_refused_and_poisoning() {
    for (name,value) in [("conjugatedType",json!(id(7))),("originalPortDefinition",json!(id(5))),("elementId",json!(id(99))),("unknownAttribute",json!(true))] {
        let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut tx=editor.begin();
        assert!(tx.set_attribute(&id(8),name,value).is_err());
        assert!(matches!(editor.commit(tx),Err(ModelError::FailedTransaction)));
        assert_eq!(editor.document().to_json().unwrap(),before);
    }
}
#[test]
fn stale_owned_transaction_cannot_replace_a_newer_live_document() {
    let mut editor=Editor::new(base());let mut tx=editor.begin();
    tx.initialize_port_definition_reference(&id(5),&id(8),R::ConjugatedType,&id(7)).unwrap();
    tx.initialize_port_definition_reference(&id(5),&id(8),R::OriginalPortDefinition,&id(5)).unwrap();
    let mut newer=editor.begin();newer.rename(&id(1),"newer-live-change").unwrap();editor.commit(newer).unwrap();
    let newer=editor.document().to_json().unwrap();assert!(editor.commit(tx).is_err());
    assert_eq!(editor.document().to_json().unwrap(),newer);assert!(editor.undo());
}
