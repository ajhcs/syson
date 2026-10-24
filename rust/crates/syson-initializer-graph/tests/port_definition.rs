#![cfg(feature="raw-factory-candidate")]
use serde_json::{json,Value};
use std::cell::Cell;
use std::rc::Rc;
use syson_initializer::{Effect,Target,ScalarFeature as SF,Scalar,Containment,ReferenceFeature as RF,CallKind};
use syson_initializer_graph::{initialize_port_definition,PortDefinitionHelperCause};
use syson_metamodel::ClassifierId as C;
use syson_model::{Document,Editor,Transaction,ContainmentFeature as F,attach_raw_selected,ModelError,PortDefinitionReference as PR};
use syson_model::factory::{RawFactory,FactoryError};
type Provider=Box<dyn FnMut()->Result<String,String>>;
fn id(n:u32)->String {format!("00000000-0000-4000-8000-{n:012x}")}
fn node(n:u32,class:&str,data:Value)->Value {
 let mut d=data.as_object().unwrap().clone();d.insert("elementId".into(),json!(id(n)));
 json!({"id":id(n+100),"eClass":format!("sysml:{class}"),"data":d})
}
fn base()->Document {
 Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[node(1,"Package",json!({"declaredName":"package","ownedRelationship":[
 node(2,"OwningMembership",json!({"ownedRelatedElement":[node(3,"PortDefinition",json!({"declaredName":"port-peer"}))]})),
 node(50,"OwningMembership",json!({"ownedRelatedElement":[node(51,"ConjugatedPortDefinition",json!({"declaredName":"conjugated-peer"}))]}))
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
fn reference_count(steps:&[syson_initializer::Step])->usize {
 steps.iter().filter(|s|matches!(&s.effect,Effect::SetReference {..})).count()
}
#[test]
fn source_order_nine_effects_three_raw_helpers_two_refs_and_nonnull_stop() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::PortDefinition);
 let (tx,r)=initialize_port_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap();
 assert!(editor.document().element(&id(5)).is_none());assert_eq!(r.applied_steps.len(),9);
 assert_eq!(r.allocations.iter().map(|x|x.classifier).collect::<Vec<_>>(),vec![C::OwningMembership,C::ConjugatedPortDefinition,C::PortConjugation]);
 assert!(matches!(&r.applied_steps[0].effect,Effect::SetScalar {target:Target::Root,feature:SF::DeclaredName,value:Scalar::Text(v)} if v=="PortDefinition2"));
 let keys=r.applied_steps.iter().filter_map(|s|match &s.effect {Effect::AllocateRaw {object,..}=>Some(*object),_=>None}).collect::<Vec<_>>();
 assert!(matches!(&r.applied_steps[1].effect,Effect::AllocateRaw {classifier:C::OwningMembership,..}));
 assert!(matches!(&r.applied_steps[2].effect,Effect::Attach {owner:Target::Root,feature:Containment::OwnedRelationship,child:Target::Planned(k)} if *k==keys[0]));
 assert!(matches!(&r.applied_steps[3].effect,Effect::AllocateRaw {classifier:C::ConjugatedPortDefinition,..}));
 assert!(matches!(&r.applied_steps[4].effect,Effect::Attach {owner:Target::Planned(p),feature:Containment::OwnedRelatedElement,child:Target::Planned(c)} if *p==keys[0]&&*c==keys[1]));
 assert!(matches!(&r.applied_steps[5].effect,Effect::AllocateRaw {classifier:C::PortConjugation,..}));
 assert!(matches!(&r.applied_steps[6].effect,Effect::Attach {owner:Target::Planned(p),feature:Containment::OwnedRelationship,child:Target::Planned(c)} if *p==keys[1]&&*c==keys[2]));
 assert!(matches!(&r.applied_steps[7].effect,Effect::SetReference {target:Target::Planned(p),feature:RF::ConjugatedType,value:Some(Target::Planned(c))} if *p==keys[2]&&*c==keys[1]));
 assert!(matches!(&r.applied_steps[8].effect,Effect::SetReference {target:Target::Planned(p),feature:RF::OriginalPortDefinition,value:Some(Target::Root)} if *p==keys[2]));
 let root=tx.document().element(&id(5)).unwrap();assert_eq!(root["data"]["declaredName"],"PortDefinition2");assert!(root["data"].get("isSufficient").is_none());
 let conjugated=tx.document().element(&id(7)).unwrap();assert!(conjugated["data"].get("declaredName").is_none());assert!(conjugated["data"].get("isComposite").is_none());
 let data=&tx.document().element(&id(8)).unwrap()["data"];assert_eq!(data["conjugatedType"],id(7));assert_eq!(data["originalPortDefinition"],id(5));
 for key in ["originalType","conjugatedPortDefinition","owningType"] {assert!(data.get(key).is_none());}
 let plan=r.declarative_plan.as_ref().unwrap();assert!(!plan.dispatch_trace.iter().any(|i|i.case_name=="Definition"&&i.kind==CallKind::GeneratedDispatch));
 assert!(!plan.dispatch_trace.iter().any(|i|i.case_name=="ConjugatedPortDefinition"));assert_eq!(r.applied_steps.last().unwrap().source_method,"casePortDefinition");
 assert_eq!(tx.document().element(&id(3)).unwrap()["data"]["declaredName"],"port-peer");
 assert!(editor.commit(tx).unwrap());let after=editor.document().to_json().unwrap();assert!(editor.undo());assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());assert!(editor.redo());assert_eq!(editor.document().to_json().unwrap(),after);
}
#[test]
fn fresh_helpers_keep_semantic_resource_and_unrepresented_native_states_distinct() {
 let editor=Editor::new(base());let mut f=normal();let tx=attached(&editor,&mut f,C::PortDefinition);
 let (tx,r)=initialize_port_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap();
 let mut semantics=std::collections::BTreeSet::new();let mut resource_ids=std::collections::BTreeSet::new();
 for a in &r.allocations {assert!(semantics.insert(a.semantic_id.clone()));assert!(resource_ids.insert(a.resource_id.clone()));assert_ne!(a.semantic_id,a.resource_id);assert_eq!(tx.document().element(&a.semantic_id).unwrap()["id"],a.resource_id);assert!(a.selected_scalars.is_empty());assert!(!a.unrepresented_runtime_fields.is_empty());}
 assert_eq!(r.allocations.len(),3);assert_eq!(reference_count(&r.applied_steps),2);
}
#[test]
fn cancellation_after_first_reference_discards_owned_transaction_and_preserves_redo() {
 let mut editor=Editor::new(base());let mut prior=editor.begin();prior.rename(&id(1),"earlier").unwrap();editor.commit(prior).unwrap();assert!(editor.undo());let before=editor.document().to_json().unwrap();
 let mut f=normal();let tx=attached(&editor,&mut f,C::PortDefinition);
 let phase=Rc::new(Cell::new(None::<u32>));let arm=phase.clone();let check=phase.clone();let mut n=0;
 let mut resources=move|_:C,_:&str|{n+=1;if n==3 {arm.set(Some(0));}Ok(id(200+n))};
 let e=initialize_port_definition(tx,&id(5),&mut f,&mut resources,move||match check.get() {Some(n)=>{check.set(Some(n+1));n+1==3},None=>false}).unwrap_err();
 assert!(matches!(e.cause,PortDefinitionHelperCause::Cancelled));assert_eq!(e.partial.applied_steps.len(),8);assert_eq!(reference_count(&e.partial.applied_steps),1);
 assert!(matches!(&e.partial.applied_steps[7].effect,Effect::SetReference {feature:RF::ConjugatedType,..}));
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(editor.redo());assert_eq!(f.allocate(C::PortConjugation).unwrap().semantic_id(),id(9));
}
#[test]
fn third_factory_error_or_identity_reuse_discards_all_staged_effects() {
 for duplicate in [false,true] {
  let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();
  let mut values=(4..8).map(|n|Ok(id(n))).collect::<Vec<_>>();values.push(if duplicate {Ok(id(6))} else {Err("third helper identity refused".into())});
  let mut f=factory(values);let tx=attached(&editor,&mut f,C::PortDefinition);
  let e=initialize_port_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap_err();
  assert!(matches!(e.cause,PortDefinitionHelperCause::Factory(FactoryError::ReusedSemanticId)|PortDefinitionHelperCause::Factory(FactoryError::IdentityProvider(_))));
  assert_eq!(e.partial.applied_steps.len(),5);assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
 }
}
#[test]
fn late_semantic_and_resource_collision_poison_without_live_history_changes() {
 for resource_collision in [false,true] {
  let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();
  let mut f=if resource_collision {normal()} else {factory([4,5,6,7,51].into_iter().map(|n|Ok(id(n))).collect())};
  let tx=attached(&editor,&mut f,C::PortDefinition);let mut n=0;let mut resources=move|_:C,_:&str|{n+=1;Ok(id(if resource_collision&&n==3 {201} else {200+n}))};
  let e=initialize_port_definition(tx,&id(5),&mut f,&mut resources,||false).unwrap_err();
  assert!(matches!(e.cause,PortDefinitionHelperCause::Model(ModelError::DuplicateId(_))));assert_eq!(reference_count(&e.partial.applied_steps),0);
  assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
 }
}
#[test]
fn third_resource_provider_refusal_keeps_history_and_consumes_factory_identity() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::PortDefinition);
 let mut n=0;let mut resources=move|_:C,_:&str|{n+=1;if n==3 {Err("third helper resource refused".into())} else {Ok(id(200+n))}};
 let e=initialize_port_definition(tx,&id(5),&mut f,&mut resources,||false).unwrap_err();
 assert!(matches!(e.cause,PortDefinitionHelperCause::ResourceProvider(_)));assert_eq!(e.partial.allocations.len(),2);assert_eq!(e.partial.applied_steps.len(),5);
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());assert_eq!(f.allocate(C::PortConjugation).unwrap().semantic_id(),id(9));
}
#[test]
fn conjugated_port_subclass_root_is_refused_before_helper_consumption() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::ConjugatedPortDefinition);
 let mut n=0;let mut resources=|_:C,_:&str|{n+=1;Ok(id(200+n))};
 let e=initialize_port_definition(tx,&id(5),&mut f,&mut resources,||false).unwrap_err();
 assert!(matches!(e.cause,PortDefinitionHelperCause::UnsupportedRoot(C::ConjugatedPortDefinition)));assert_eq!(n,0);assert_eq!(f.allocate(C::OwningMembership).unwrap().semantic_id(),id(6));assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn existing_conjugated_children_are_preserved_and_not_selected_for_new_references() {
 let editor=Editor::new(base());let mut f=normal();let mut tx=attached(&editor,&mut f,C::PortDefinition);
 let existing=node(40,"OwningMembership",json!({"ownedRelatedElement":[node(41,"ConjugatedPortDefinition",json!({"ownedRelationship":[node(42,"PortConjugation",json!({}))]}))]}));
 tx.attach_preallocated_subtree(&id(5),F::OwnedRelationship,existing.clone()).unwrap();
 let (tx,_)=initialize_port_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap();
 let children=tx.document().element(&id(5)).unwrap()["data"]["ownedRelationship"].as_array().unwrap();assert_eq!(children.len(),2);assert_eq!(children[0],existing);
 assert_eq!(tx.document().element(&id(8)).unwrap()["data"]["conjugatedType"],id(7));assert!(tx.document().element(&id(42)).unwrap()["data"].get("conjugatedType").is_none());
}
#[test]
fn initialized_constructor_then_wrong_reference_role_cannot_commit_any_staged_changes() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::PortDefinition);
 let (mut tx,_)=initialize_port_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap();
 assert!(tx.initialize_port_definition_reference(&id(5),&id(8),PR::OriginalPortDefinition,&id(3)).is_err());
 assert!(matches!(editor.commit(tx),Err(ModelError::FailedTransaction)));assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn early_cancellation_never_allocates_helpers_or_changes_live_document() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::PortDefinition);
 let mut resources=|_:C,_:&str|->Result<String,String>{panic!("cancelled constructor must not request resources")};
 let e=initialize_port_definition(tx,&id(5),&mut f,&mut resources,||true).unwrap_err();
 assert!(matches!(e.cause,PortDefinitionHelperCause::Cancelled));assert!(e.partial.allocations.is_empty());assert!(e.partial.applied_steps.is_empty());assert_eq!(f.allocate(C::OwningMembership).unwrap().semantic_id(),id(6));assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
