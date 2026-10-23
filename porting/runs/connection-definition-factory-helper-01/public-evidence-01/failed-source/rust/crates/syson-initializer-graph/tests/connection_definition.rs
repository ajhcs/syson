#![cfg(feature="raw-factory-candidate")]
use serde_json::{json,Value};
use std::cell::Cell;
use std::rc::Rc;
use syson_initializer::{Effect,Target,ScalarFeature as SF,Scalar,Containment};
use syson_initializer_graph::{initialize_connection_definition,ConnectionHelperCause};
use syson_metamodel::ClassifierId as C;
use syson_model::{Document,Editor,Transaction,ContainmentFeature as F,attach_raw_selected,RawAdmissionReport,RawAdmissionError,ModelError};
use syson_model::factory::{RawFactory,FactoryError};
type Provider=Box<dyn FnMut()->Result<String,String>>;
fn id(n:u32)->String {format!("00000000-0000-4000-8000-{n:012x}")}
fn node(n:u32,class:&str,data:Value)->Value {
 let mut d=data.as_object().unwrap().clone();d.insert("elementId".into(),json!(id(n)));
 json!({"id":id(n+100),"eClass":format!("sysml:{class}"),"data":d})
}
fn base()->Document {
 Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[node(1,"Package",json!({"declaredName":"package","ownedRelationship":[
 node(2,"OwningMembership",json!({"ownedRelatedElement":[node(3,"ConnectionDefinition",json!({"declaredName":"existing"}))]})),
 node(50,"OwningMembership",json!({"ownedRelatedElement":[node(51,"InterfaceDefinition",json!({"declaredName":"subclass-peer"}))]}))
 ]}))]}).to_string()).unwrap()
}
fn factory(values:Vec<Result<String,String>>)->RawFactory<Provider> {
 let mut values=values.into_iter();
 RawFactory::new(Box::new(move||values.next().unwrap_or_else(||Err("fixture provider exhausted".into()))))
}
fn normal()->RawFactory<Provider> {factory((4..30).map(|n|Ok(id(n))).collect())}
fn attached(editor:&Editor,f:&mut RawFactory<Provider>,class:C)->Transaction {
 let parent=f.allocate(C::OwningMembership).unwrap();let root=f.allocate(class).unwrap();
 let (tx,_)=attach_raw_selected(editor.begin(),&id(1),F::OwnedRelationship,&parent,&id(104),&[]).unwrap();
 attach_raw_selected(tx,&id(4),F::OwnedRelatedElement,&root,&id(105),&[]).unwrap().0
}
fn resources()->impl FnMut(C,&str)->Result<String,String> {
 let mut n=200;move|_,_|{n+=1;Ok(id(n))}
}
#[test]
fn native_constructor_exact_end_shape_source_order_and_one_history_entry() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::ConnectionDefinition);
 let (tx,report)=initialize_connection_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap();
 assert!(editor.document().element(&id(5)).is_none());
 let root=tx.document().element(&id(5)).unwrap();
 assert_eq!(root["data"]["isSufficient"],true);
 assert_eq!(root["data"]["declaredName"],"ConnectionDefinition2");
 assert_eq!(report.allocations.iter().map(|a|a.classifier).collect::<Vec<_>>(),vec![C::ReferenceUsage,C::FeatureMembership,C::ReferenceUsage,C::FeatureMembership]);
 let ends=root["data"]["ownedRelationship"].as_array().unwrap();assert_eq!(ends.len(),2);
 for (at,name) in ["source","target"].into_iter().enumerate() {
  let membership=&ends[at];assert_eq!(membership["eClass"],"sysml:FeatureMembership");
  let children=membership["data"]["ownedRelatedElement"].as_array().unwrap();assert_eq!(children.len(),1);
  let reference=&children[0];assert_eq!(reference["eClass"],"sysml:ReferenceUsage");
  assert_eq!(reference["data"]["declaredName"],name);assert_eq!(reference["data"]["isEnd"],true);assert_eq!(reference["data"]["isComposite"],false);
 }
 let plan=report.declarative_plan.as_ref().unwrap();
 let refs=plan.steps.iter().filter_map(|s|match &s.effect {Effect::AllocateRaw {object,classifier:C::ReferenceUsage,..}=>Some(*object),_=>None}).collect::<Vec<_>>();
 for (key,name) in refs.into_iter().zip(["source","target"]) {
  let setters=report.applied_steps.iter().filter_map(|s|match &s.effect {Effect::SetScalar {target:Target::Planned(k),feature,value} if *k==key=>Some((*feature,value.clone())),_=>None}).collect::<Vec<_>>();
  assert_eq!(setters,vec![(SF::DeclaredName,Scalar::Text("reference0".into())),(SF::IsComposite,Scalar::Boolean(true)),(SF::IsComposite,Scalar::Boolean(false)),(SF::DeclaredName,Scalar::Text(name.into())),(SF::IsEnd,Scalar::Boolean(true))]);
 }
 assert_eq!(report.applied_steps.last().unwrap().source_method,"caseDefinition");
 assert_eq!(tx.document().element(&id(3)).unwrap()["data"]["declaredName"],"existing");
 editor.commit(tx).unwrap();let after=editor.document().to_json().unwrap();
 assert!(editor.undo());assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
 assert!(editor.redo());assert_eq!(editor.document().to_json().unwrap(),after);
}
#[test]
fn helper_resource_and_semantic_identities_stay_distinct() {
 let editor=Editor::new(base());let mut f=normal();let tx=attached(&editor,&mut f,C::ConnectionDefinition);
 let (tx,r)=initialize_connection_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap();
 assert_eq!(r.allocations.len(),4);
 for a in &r.allocations {
  assert_ne!(a.resource_id,a.semantic_id);
  assert_eq!(tx.document().element(&a.semantic_id).unwrap()["id"],a.resource_id);
  assert!(a.selected_scalars.is_empty());assert!(!a.unrepresented_runtime_fields.is_empty());
 }
}
#[test]
fn late_cancellation_discards_first_attached_endpoint_and_preserves_redo() {
 let mut editor=Editor::new(base());let mut prior=editor.begin();prior.rename(&id(1),"earlier").unwrap();editor.commit(prior).unwrap();editor.undo();
 let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::ConnectionDefinition);
 let calls=Rc::new(Cell::new(0_u32));let resource_calls=Rc::clone(&calls);let cancel_calls=Rc::clone(&calls);
 let mut r=move|_:C,_:&str|{resource_calls.set(resource_calls.get()+1);Ok(id(200+resource_calls.get()))};
 let error=initialize_connection_definition(tx,&id(5),&mut f,&mut r,move||cancel_calls.get()>=3).unwrap_err();
 assert!(matches!(error.cause,ConnectionHelperCause::Cancelled));
 assert!(error.partial.applied_steps.iter().any(|s|matches!(&s.effect,Effect::Attach {owner:Target::Root,feature:Containment::OwnedRelationship,..})));
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(editor.document().element(&id(5)).is_none());assert!(editor.redo());
}
#[test]
fn semantic_collision_with_existing_document_rolls_back_creation() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();
 let mut f=factory([4,5,3,7,8,9].into_iter().map(|n|Ok(id(n))).collect());let tx=attached(&editor,&mut f,C::ConnectionDefinition);
 let error=initialize_connection_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap_err();
 assert!(matches!(error.cause,ConnectionHelperCause::Model(ModelError::DuplicateId(_))));
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn late_resource_collision_discards_already_attached_source_endpoint() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::ConnectionDefinition);
 let mut values=[201,202,201,204].into_iter();let mut r=move|_:C,_:&str|Ok(id(values.next().unwrap()));
 let error=initialize_connection_definition(tx,&id(5),&mut f,&mut r,||false).unwrap_err();
 assert!(matches!(error.cause,ConnectionHelperCause::Model(ModelError::DuplicateId(_))));
 assert!(error.partial.applied_steps.iter().any(|s|matches!(&s.effect,Effect::Attach {owner:Target::Root,..})));
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn provider_failure_and_same_factory_uuid_reuse_never_commit_partial_creation() {
 for duplicate in [false,true] {
  let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();
  let values=if duplicate {vec![Ok(id(4)),Ok(id(5)),Ok(id(6)),Ok(id(7)),Ok(id(6))]} else {vec![Ok(id(4)),Ok(id(5)),Ok(id(6)),Ok(id(7)),Err("fixture unavailable".into())]};
  let mut f=factory(values);let tx=attached(&editor,&mut f,C::ConnectionDefinition);
  let error=initialize_connection_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap_err();
  assert!(matches!(error.cause,ConnectionHelperCause::Factory(FactoryError::ReusedSemanticId)|ConnectionHelperCause::Factory(FactoryError::IdentityProvider(_))));
  assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
 }
}
#[test]
fn explicit_resource_provider_refusal_discards_owned_transaction() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::ConnectionDefinition);
 let mut calls=0;let mut r=move|_:C,_:&str|{calls+=1;if calls==3 {Err("fixture resource refusal".into())} else {Ok(id(200+calls))}};
 let error=initialize_connection_definition(tx,&id(5),&mut f,&mut r,||false).unwrap_err();
 assert!(matches!(error.cause,ConnectionHelperCause::ResourceProvider(_)));
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn interface_definition_is_refused_without_consuming_helper_identity() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::InterfaceDefinition);
 let mut calls=0;let mut r=|_:C,_:&str|{calls+=1;Ok(id(200+calls))};
 let error=initialize_connection_definition(tx,&id(5),&mut f,&mut r,||false).unwrap_err();
 assert!(matches!(error.cause,ConnectionHelperCause::UnsupportedRoot(C::InterfaceDefinition)));assert_eq!(calls,0);
 assert_eq!(f.allocate(C::ReferenceUsage).unwrap().semantic_id(),id(6));
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn existing_relationships_are_preserved_and_new_endpoints_append() {
 let editor=Editor::new(base());let mut f=normal();let mut tx=attached(&editor,&mut f,C::ConnectionDefinition);
 let existing=node(40,"FeatureMembership",json!({"ownedRelatedElement":[node(41,"ReferenceUsage",json!({"declaredName":"existing-end"}))]}));
 tx.attach_preallocated_subtree(&id(5),F::OwnedRelationship,existing.clone()).unwrap();
 let (tx,_)=initialize_connection_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap();
 let rels=tx.document().element(&id(5)).unwrap()["data"]["ownedRelationship"].as_array().unwrap();
 assert_eq!(rels.len(),3);assert_eq!(rels[0],existing);
 assert_eq!(rels[1]["data"]["ownedRelatedElement"][0]["data"]["declaredName"],"source");
 assert_eq!(rels[2]["data"]["ownedRelatedElement"][0]["data"]["declaredName"],"target");
}
#[test]
fn detached_partial_projection_keeps_selected_null_and_refuses_reference_selection() {
 let mut f=normal();let raw=f.allocate(C::ReferenceUsage).unwrap();
 let (node,report)=RawAdmissionReport::project_selected(&raw,&id(204),&["declaredName"]).unwrap();
 assert!(node["data"].as_object().unwrap().contains_key("declaredName"));assert_eq!(node["data"]["declaredName"],Value::Null);
 assert_eq!(raw.resource_id(),None);assert_eq!(report.selected_scalars.len(),1);
 assert!(matches!(RawAdmissionReport::project_selected(&raw,&id(204),&["ownedRelationship"]),Err(RawAdmissionError::Factory(_))));
}
