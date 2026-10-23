#![cfg(feature="raw-factory-candidate")]
use serde_json::{json,Value};
use syson_metamodel::ClassifierId as C;
use syson_model::{Document,Editor,ContainmentFeature as F,attach_raw_selected,RawAdmissionError};
use syson_model::factory::{RawFactory,RawObject,StorageObservation};
use syson_initializer_graph::{initialize_staged,ReferenceEncoding};
fn id(n:u32)->String {format!("00000000-0000-4000-8000-{n:012x}")}
fn node(n:u32,class:&str,data:Value)->Value {
 let mut d=data.as_object().unwrap().clone();d.insert("elementId".into(),json!(id(n)));
 json!({"id":id(n+100),"eClass":format!("sysml:{class}"),"data":d})
}
fn base()->Document {
 Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[node(1,"Package",json!({"declaredName":"root","ownedRelationship":[node(2,"OwningMembership",json!({"ownedRelatedElement":[node(3,"PartUsage",json!({"declaredName":"existing"}))]}))]}))]}).to_string()).unwrap()
}
fn raw(class:C,n:u32)->RawObject {
 let mut factory=RawFactory::new(move||Ok(id(n)));factory.allocate(class).unwrap()
}
#[test]
fn explicit_resource_assignment_preserves_raw_identity_and_runtime_state() {
 let editor=Editor::new(base());let raw=raw(C::OwningMembership,4);
 let before:Vec<_>=raw.cached_fields().iter().map(|f|(f.spec.id,f.value.clone(),f.origin.clone())).collect();
 let (tx,report)=attach_raw_selected(editor.begin(),&id(1),F::OwnedRelationship,&raw,&id(104),&[]).unwrap();
 assert_eq!(raw.semantic_id(),id(4));assert_eq!(raw.resource_id(),None);
 assert_eq!(tx.document().element(&id(4)).unwrap()["id"],id(104));
 assert_eq!(report.semantic_id,id(4));assert_eq!(report.resource_id,id(104));
 let after:Vec<_>=raw.cached_fields().iter().map(|f|(f.spec.id,f.value.clone(),f.origin.clone())).collect();
 assert_eq!(before,after);assert_eq!(report.unrepresented_runtime_fields.len(),raw.cached_fields().len());
 assert!(editor.document().element(&id(4)).is_none());
}
#[test]
fn selected_null_is_deliberately_materialized_without_claiming_source_storage() {
 let editor=Editor::new(base());let wrapper=raw(C::OwningMembership,4);let child=raw(C::PartUsage,5);
 let (tx,_)=attach_raw_selected(editor.begin(),&id(1),F::OwnedRelationship,&wrapper,&id(104),&[]).unwrap();
 let (tx,report)=attach_raw_selected(tx,&id(4),F::OwnedRelatedElement,&child,&id(105),&["declaredName"]).unwrap();
 assert!(tx.document().element(&id(5)).unwrap()["data"].as_object().unwrap().contains_key("declaredName"));
 assert_eq!(tx.document().element(&id(5)).unwrap()["data"]["declaredName"],Value::Null);
 assert_eq!(report.selected_scalars[0].1.storage_observation,StorageObservation::Unobserved);
 assert_eq!(child.resource_id(),None);
}
#[test]
fn unselected_scalars_are_omitted_by_explicit_policy_not_inferred_eset() {
 let editor=Editor::new(base());let wrapper=raw(C::OwningMembership,4);let child=raw(C::PartUsage,5);
 let (tx,_)=attach_raw_selected(editor.begin(),&id(1),F::OwnedRelationship,&wrapper,&id(104),&[]).unwrap();
 let (tx,report)=attach_raw_selected(tx,&id(4),F::OwnedRelatedElement,&child,&id(105),&[]).unwrap();
 assert!(!tx.document().element(&id(5)).unwrap()["data"].as_object().unwrap().contains_key("declaredName"));
 assert!(report.selected_scalars.is_empty());assert_eq!(report.unrepresented_runtime_fields.len(),child.cached_fields().len());
}
#[test]
fn invalid_resource_id_discards_owned_pending_creation() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let wrapper=raw(C::OwningMembership,4);let child=raw(C::PartUsage,5);
 let (tx,_)=attach_raw_selected(editor.begin(),&id(1),F::OwnedRelationship,&wrapper,&id(104),&[]).unwrap();
 assert!(matches!(attach_raw_selected(tx,&id(4),F::OwnedRelatedElement,&child,"not-a-resource-id",&[]),Err(RawAdmissionError::Model(_))));
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn identity_duplicate_and_reference_selections_refuse_without_partial_creation() {
 for attributes in [vec!["elementId"],vec!["declaredName","declaredName"],vec!["ownedRelationship"]] {
  let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let wrapper=raw(C::OwningMembership,4);let child=raw(C::PartUsage,5);
  let (tx,_)=attach_raw_selected(editor.begin(),&id(1),F::OwnedRelationship,&wrapper,&id(104),&[]).unwrap();
  assert!(attach_raw_selected(tx,&id(4),F::OwnedRelatedElement,&child,&id(105),&attributes).is_err());
  assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
 }
}
#[test]
fn duplicate_from_independently_faulty_factories_is_caught_by_document_admission() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();
 let left=raw(C::OwningMembership,4);let right=raw(C::OwningMembership,4);
 let (tx,_)=attach_raw_selected(editor.begin(),&id(1),F::OwnedRelationship,&left,&id(104),&[]).unwrap();
 assert!(matches!(attach_raw_selected(tx,&id(1),F::OwnedRelationship,&right,&id(106),&[]),Err(RawAdmissionError::Model(syson_model::ModelError::DuplicateId(_)))));
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn actual_factory_admission_query_defaults_single_commit_and_undo_redo() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();
 let wrapper=raw(C::OwningMembership,4);let child=raw(C::PartUsage,5);
 let (tx,_)=attach_raw_selected(editor.begin(),&id(1),F::OwnedRelationship,&wrapper,&id(104),&[]).unwrap();
 let (tx,_)=attach_raw_selected(tx,&id(4),F::OwnedRelatedElement,&child,&id(105),&["declaredName"]).unwrap();
 let tx=initialize_staged(tx,&id(5),ReferenceEncoding::Disabled,false).unwrap();
 assert_eq!(tx.document().element(&id(5)).unwrap()["data"]["declaredName"],"part2");
 assert_eq!(tx.document().element(&id(5)).unwrap()["data"]["isComposite"],true);
 assert_eq!(child.resource_id(),None);
 editor.commit(tx).unwrap();let after=editor.document().to_json().unwrap();
 assert!(editor.undo());assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
 assert!(editor.redo());assert_eq!(editor.document().to_json().unwrap(),after);
}
#[test]
fn cancellation_after_real_allocation_and_admission_preserves_redo() {
 let mut editor=Editor::new(base());let mut first=editor.begin();first.rename(&id(1),"earlier").unwrap();editor.commit(first).unwrap();editor.undo();
 let before=editor.document().to_json().unwrap();let wrapper=raw(C::OwningMembership,4);let child=raw(C::PartUsage,5);
 let (tx,_)=attach_raw_selected(editor.begin(),&id(1),F::OwnedRelationship,&wrapper,&id(104),&[]).unwrap();
 let (tx,_)=attach_raw_selected(tx,&id(4),F::OwnedRelatedElement,&child,&id(105),&["declaredName"]).unwrap();
 assert!(initialize_staged(tx,&id(5),ReferenceEncoding::Disabled,true).is_err());
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(editor.redo());
}
