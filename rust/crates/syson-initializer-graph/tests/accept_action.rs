#![cfg(feature="raw-factory-candidate")]
use serde_json::{json,Value};
use std::cell::Cell;
use std::rc::Rc;
use syson_initializer::{Effect,Target,ScalarFeature as SF,Scalar,Containment,Direction as D,CallKind};
use syson_initializer_graph::{initialize_accept_action,AcceptActionHelperCause};
use syson_metamodel::ClassifierId as C;
use syson_model::{Document,Editor,Transaction,ContainmentFeature as F,attach_raw_selected,ModelError};
use syson_model::factory::{RawFactory,FactoryError};
type Provider=Box<dyn FnMut()->Result<String,String>>;
fn id(n:u32)->String {format!("00000000-0000-4000-8000-{n:012x}")}
fn node(n:u32,class:&str,data:Value)->Value {
 let mut d=data.as_object().unwrap().clone();d.insert("elementId".into(),json!(id(n)));
 json!({"id":id(n+100),"eClass":format!("sysml:{class}"),"data":d})
}
fn base()->Document {
 Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[node(1,"Package",json!({"declaredName":"package","ownedRelationship":[
 node(2,"OwningMembership",json!({"ownedRelatedElement":[node(3,"AcceptActionUsage",json!({"declaredName":"accept-peer"}))]})),
 node(50,"OwningMembership",json!({"ownedRelatedElement":[node(51,"InterfaceDefinition",json!({"declaredName":"interface-peer"}))]}))
 ]}))]}).to_string()).unwrap()
}
fn factory(values:Vec<Result<String,String>>)->RawFactory<Provider> {
 let mut values=values.into_iter();
 RawFactory::new(Box::new(move||values.next().unwrap_or_else(||Err("fixture provider exhausted".into()))))
}
fn normal()->RawFactory<Provider> {factory((4..50).map(|n|Ok(id(n))).collect())}
fn attached(editor:&Editor,f:&mut RawFactory<Provider>,class:C)->Transaction {
 let membership=f.allocate(C::OwningMembership).unwrap();let root=f.allocate(class).unwrap();
 let (tx,_)=attach_raw_selected(editor.begin(),&id(1),F::OwnedRelationship,&membership,&id(104),&[]).unwrap();
 attach_raw_selected(tx,&id(4),F::OwnedRelatedElement,&root,&id(105),&[]).unwrap().0
}
fn resources()->impl FnMut(C,&str)->Result<String,String> {let mut n=200;move|_,_|{n+=1;Ok(id(n))}}
fn root_attachments(steps:&[syson_initializer::Step])->usize {
 steps.iter().filter(|s|matches!(&s.effect,Effect::Attach {owner:Target::Root,feature:Containment::OwnedRelationship,..})).count()
}
#[test]
fn accept_usage_first_raw_parameters_inout_then_in_nonnull_stop_and_one_commit() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::AcceptActionUsage);
 let (tx,r)=initialize_accept_action(tx,&id(5),&mut f,&mut resources(),||false).unwrap();
 assert!(editor.document().element(&id(5)).is_none());assert_eq!(r.applied_steps.len(),14);
 assert_eq!(r.allocations.iter().map(|a|a.classifier).collect::<Vec<_>>(),vec![C::ReferenceUsage,C::ParameterMembership,C::ReferenceUsage,C::ParameterMembership]);
 let root=tx.document().element(&id(5)).unwrap();assert_eq!(root["data"]["declaredName"],"acceptAction2");assert_eq!(root["data"]["isComposite"],true);
 assert!(matches!(&r.applied_steps[0].effect,Effect::SetScalar {target:Target::Root,feature:SF::DeclaredName,value:Scalar::Text(v)} if v=="acceptAction2"));
 assert!(matches!(&r.applied_steps[1].effect,Effect::SetScalar {target:Target::Root,feature:SF::IsComposite,value:Scalar::Boolean(true)}));
 let members=root["data"]["ownedRelationship"].as_array().unwrap();assert_eq!(members.len(),2);
 for (i,(name,direction)) in [("payload","inout"),("receiver","in")].into_iter().enumerate() {
  assert_eq!(members[i]["eClass"],"sysml:ParameterMembership");let refs=members[i]["data"]["ownedRelatedElement"].as_array().unwrap();assert_eq!(refs.len(),1);
  let child=&refs[0];assert_eq!(child["eClass"],"sysml:ReferenceUsage");assert_eq!(child["data"]["declaredName"],name);assert_eq!(child["data"]["direction"],direction);
  assert!(child["data"].get("isComposite").is_none());assert!(child["data"].get("isEnd").is_none());
 }
 let plan=r.declarative_plan.as_ref().unwrap();
 let direct=plan.dispatch_trace.iter().filter(|c|c.kind==CallKind::DirectCase).map(|c|c.case_name).collect::<Vec<_>>();assert_eq!(direct,vec!["Usage"]);
 let directions=r.applied_steps.iter().filter_map(|s|match &s.effect {Effect::SetScalar {target:Target::Planned(_),feature:SF::Direction,value:Scalar::Direction(d)}=>Some(*d),_=>None}).collect::<Vec<_>>();assert_eq!(directions,vec![D::InOut,D::In]);
 assert_eq!(root_attachments(&r.applied_steps),2);assert_eq!(r.applied_steps.last().unwrap().source_method,"caseAcceptActionUsage");
 assert_eq!(tx.document().element(&id(3)).unwrap()["data"]["declaredName"],"accept-peer");
 editor.commit(tx).unwrap();let after=editor.document().to_json().unwrap();assert!(editor.undo());assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());assert!(editor.redo());assert_eq!(editor.document().to_json().unwrap(),after);
}
#[test]
fn four_fresh_helpers_keep_semantic_resource_and_unrepresented_states_separate() {
 let editor=Editor::new(base());let mut f=normal();let tx=attached(&editor,&mut f,C::AcceptActionUsage);
 let (tx,r)=initialize_accept_action(tx,&id(5),&mut f,&mut resources(),||false).unwrap();assert_eq!(r.allocations.len(),4);
 let mut ids=std::collections::BTreeSet::new();let mut resources=std::collections::BTreeSet::new();
 for a in &r.allocations {assert!(ids.insert(a.semantic_id.clone()));assert!(resources.insert(a.resource_id.clone()));assert_ne!(a.semantic_id,a.resource_id);assert_eq!(tx.document().element(&a.semantic_id).unwrap()["id"],a.resource_id);assert!(a.selected_scalars.is_empty());assert!(!a.unrepresented_runtime_fields.is_empty());}
}
#[test]
fn cancellation_after_payload_attachment_preserves_redo_and_does_not_refund_ids() {
 let mut editor=Editor::new(base());let mut prior=editor.begin();prior.rename(&id(1),"earlier").unwrap();editor.commit(prior).unwrap();editor.undo();
 let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::AcceptActionUsage);
 let count=Rc::new(Cell::new(0_u32));let alloc=count.clone();let cancel=count.clone();let mut resources=move|_:C,_:&str|{alloc.set(alloc.get()+1);Ok(id(200+alloc.get()))};
 let e=initialize_accept_action(tx,&id(5),&mut f,&mut resources,move||cancel.get()>=3).unwrap_err();assert!(matches!(e.cause,AcceptActionHelperCause::Cancelled));assert_eq!(root_attachments(&e.partial.applied_steps),1);
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(editor.redo());assert_eq!(f.allocate(C::ReferenceUsage).unwrap().semantic_id(),id(9));
}
#[test]
fn receiver_semantic_collision_discards_payload_and_usage_defaults() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=factory([4,5,6,7,51,9].into_iter().map(|n|Ok(id(n))).collect());let tx=attached(&editor,&mut f,C::AcceptActionUsage);
 let e=initialize_accept_action(tx,&id(5),&mut f,&mut resources(),||false).unwrap_err();assert!(matches!(e.cause,AcceptActionHelperCause::Model(ModelError::DuplicateId(_))));assert_eq!(root_attachments(&e.partial.applied_steps),1);assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn receiver_resource_collision_discards_already_attached_payload() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::AcceptActionUsage);
 let mut values=[201,202,201,204].into_iter();let mut resources=move|_:C,_:&str|Ok(id(values.next().unwrap()));
 let e=initialize_accept_action(tx,&id(5),&mut f,&mut resources,||false).unwrap_err();assert!(matches!(e.cause,AcceptActionHelperCause::Model(ModelError::DuplicateId(_))));assert_eq!(root_attachments(&e.partial.applied_steps),1);assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn late_factory_failure_and_reuse_roll_back_all_constructor_effects() {
 for duplicate in [false,true] {
  let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut values=(4..9).map(|n|Ok(id(n))).collect::<Vec<_>>();values.push(if duplicate {Ok(id(6))} else {Err("late provider refusal".into())});
  let mut f=factory(values);let tx=attached(&editor,&mut f,C::AcceptActionUsage);let e=initialize_accept_action(tx,&id(5),&mut f,&mut resources(),||false).unwrap_err();
  assert!(matches!(e.cause,AcceptActionHelperCause::Factory(FactoryError::ReusedSemanticId)|AcceptActionHelperCause::Factory(FactoryError::IdentityProvider(_))));assert_eq!(root_attachments(&e.partial.applied_steps),1);assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
 }
}
#[test]
fn receiver_resource_provider_refusal_drops_the_owned_transaction() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::AcceptActionUsage);
 let mut n=0;let mut resources=move|_:C,_:&str|{n+=1;if n==3 {Err("receiver resource refusal".into())} else {Ok(id(200+n))}};
 let e=initialize_accept_action(tx,&id(5),&mut f,&mut resources,||false).unwrap_err();assert!(matches!(e.cause,AcceptActionHelperCause::ResourceProvider(_)));assert_eq!(root_attachments(&e.partial.applied_steps),1);assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn interface_root_is_refused_without_consuming_helper_identity() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::InterfaceDefinition);
 let mut count=0;let mut resources=|_:C,_:&str|{count+=1;Ok(id(200+count))};let e=initialize_accept_action(tx,&id(5),&mut f,&mut resources,||false).unwrap_err();
 assert!(matches!(e.cause,AcceptActionHelperCause::UnsupportedRoot(C::InterfaceDefinition)));assert_eq!(count,0);assert_eq!(f.allocate(C::ReferenceUsage).unwrap().semantic_id(),id(6));assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn existing_memberships_are_preserved_and_payload_receiver_append() {
 let editor=Editor::new(base());let mut f=normal();let mut tx=attached(&editor,&mut f,C::AcceptActionUsage);
 let existing=node(40,"ParameterMembership",json!({"ownedRelatedElement":[node(41,"ReferenceUsage",json!({"declaredName":"existing-parameter"}))]}));tx.attach_preallocated_subtree(&id(5),F::OwnedRelationship,existing.clone()).unwrap();
 let (tx,_)=initialize_accept_action(tx,&id(5),&mut f,&mut resources(),||false).unwrap();let members=tx.document().element(&id(5)).unwrap()["data"]["ownedRelationship"].as_array().unwrap();assert_eq!(members.len(),3);assert_eq!(members[0],existing);assert_eq!(members[1]["data"]["ownedRelatedElement"][0]["data"]["declaredName"],"payload");assert_eq!(members[2]["data"]["ownedRelatedElement"][0]["data"]["declaredName"],"receiver");
}
#[test]
fn generic_enum_type_identity_and_unknown_refusals_stay_poisoning() {
 for (name,value) in [("direction",json!("invalid-direction")),("direction",json!(true)),("elementId",json!(id(99))),("unsupportedAttribute",json!(true))] {
  let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::AcceptActionUsage);
  let (mut tx,r)=initialize_accept_action(tx,&id(5),&mut f,&mut resources(),||false).unwrap();let reference=&r.allocations[0].semantic_id;
  let e=tx.set_attribute(reference,name,value).unwrap_err();match name {
   "direction"=>assert!(matches!(e,ModelError::Attribute(syson_model::AttributeError::InvalidEnumLiteral {..})|ModelError::Attribute(syson_model::AttributeError::WrongValueType {..}))),
   "elementId"=>assert!(matches!(e,ModelError::Attribute(syson_model::AttributeError::Blocked {reason:syson_model::AttributeBlock::IdentityManaged,..}))),
   _=>assert!(matches!(e,ModelError::Attribute(syson_model::AttributeError::UnknownAttribute(_))))
  }
  assert!(matches!(editor.commit(tx),Err(ModelError::FailedTransaction)));assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
 }
}
