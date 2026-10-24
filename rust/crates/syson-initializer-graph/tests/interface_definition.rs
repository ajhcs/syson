#![cfg(feature="raw-factory-candidate")]
use serde_json::{json,Value};
use std::cell::Cell;
use std::rc::Rc;
use syson_initializer::{Effect,Target,ScalarFeature as SF,Scalar,Containment};
use syson_initializer_graph::{initialize_interface_definition,InterfaceHelperCause};
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
 node(2,"OwningMembership",json!({"ownedRelatedElement":[node(3,"ConnectionDefinition",json!({"declaredName":"connection-peer"}))]})),
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
fn interface_source_order_raw_ports_references_two_true_assignments_final_name_and_one_commit() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();
 let tx=attached(&editor,&mut f,C::InterfaceDefinition);
 let (tx,report)=initialize_interface_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap();
 assert!(editor.document().element(&id(5)).is_none());
 assert_eq!(report.applied_steps.len(),33);
 assert_eq!(report.allocations.iter().map(|r|r.classifier).collect::<Vec<_>>(),vec![C::PortUsage,C::FeatureMembership,C::PortUsage,C::FeatureMembership,C::ReferenceUsage,C::FeatureMembership,C::ReferenceUsage,C::FeatureMembership]);
 let root=tx.document().element(&id(5)).unwrap();assert_eq!(root["data"]["isSufficient"],true);assert_eq!(root["data"]["declaredName"],"InterfaceDefinition2");
 let ends=root["data"]["ownedRelationship"].as_array().unwrap();assert_eq!(ends.len(),4);
 for (i,(class,name)) in [("PortUsage","source"),("PortUsage","target"),("ReferenceUsage","source"),("ReferenceUsage","target")].into_iter().enumerate() {
  assert_eq!(ends[i]["eClass"],"sysml:FeatureMembership");
  let members=ends[i]["data"]["ownedRelatedElement"].as_array().unwrap();assert_eq!(members.len(),1);
  let end=&members[0];assert_eq!(end["eClass"],format!("sysml:{class}"));assert_eq!(end["data"]["declaredName"],name);assert_eq!(end["data"]["isEnd"],true);
  if class=="PortUsage" {assert!(end["data"].get("isComposite").is_none())} else {assert_eq!(end["data"]["isComposite"],false)}
 }
 let plan=report.declarative_plan.as_ref().unwrap();
 let flags=report.applied_steps.iter().enumerate().filter_map(|(i,s)|match &s.effect {
  Effect::SetScalar {target:Target::Root,feature:SF::IsSufficient,value:Scalar::Boolean(true)}=>Some((i,s.source_method)),_=>None
 }).collect::<Vec<_>>();
 assert_eq!(flags.len(),2);assert_eq!(flags[0],(0,"caseInterfaceDefinition"));assert_eq!(flags[1].1,"caseConnectionDefinition");
 assert_eq!(root_attachments(&report.applied_steps[..flags[1].0]),2);
 for (key,class) in plan.steps.iter().filter_map(|s|match &s.effect {Effect::AllocateRaw {object,classifier,..} if matches!(classifier,C::PortUsage|C::ReferenceUsage)=>Some((*object,*classifier)),_=>None}) {
  let setters=report.applied_steps.iter().filter_map(|s|match &s.effect {
   Effect::SetScalar {target:Target::Planned(k),feature,value} if *k==key=>Some((*feature,value.clone())),_=>None
  }).collect::<Vec<_>>();
  if class==C::PortUsage {
   assert_eq!(setters.len(),2);assert_eq!(setters[0].0,SF::DeclaredName);assert_eq!(setters[1],(SF::IsEnd,Scalar::Boolean(true)));
  } else {
   assert_eq!(setters.len(),5);assert_eq!(setters[0],(SF::DeclaredName,Scalar::Text("reference0".into())));
   assert_eq!(setters[1],(SF::IsComposite,Scalar::Boolean(true)));assert_eq!(setters[2],(SF::IsComposite,Scalar::Boolean(false)));
  }
 }
 assert_eq!(report.applied_steps.last().unwrap().source_method,"caseDefinition");
 assert_eq!(tx.document().element(&id(3)).unwrap()["data"]["declaredName"],"connection-peer");
 assert_eq!(tx.document().element(&id(51)).unwrap()["data"]["declaredName"],"interface-peer");
 editor.commit(tx).unwrap();let after=editor.document().to_json().unwrap();assert!(editor.undo());assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());assert!(editor.redo());assert_eq!(editor.document().to_json().unwrap(),after);
}
#[test]
fn eight_helper_identities_resources_and_unrepresented_constructor_state_are_explicit() {
 let editor=Editor::new(base());let mut f=normal();let tx=attached(&editor,&mut f,C::InterfaceDefinition);
 let (tx,report)=initialize_interface_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap();
 assert_eq!(report.allocations.len(),8);
 let mut semantic=std::collections::BTreeSet::new();let mut resource=std::collections::BTreeSet::new();
 for a in &report.allocations {
  assert!(semantic.insert(a.semantic_id.clone()));assert!(resource.insert(a.resource_id.clone()));assert_ne!(a.semantic_id,a.resource_id);
  assert_eq!(tx.document().element(&a.semantic_id).unwrap()["id"],a.resource_id);
  assert!(a.selected_scalars.is_empty());assert!(!a.unrepresented_runtime_fields.is_empty());
 }
}
#[test]
fn cancellation_after_three_attached_ends_drops_owned_creation_and_preserves_redo() {
 let mut editor=Editor::new(base());let mut prior=editor.begin();prior.rename(&id(1),"earlier").unwrap();editor.commit(prior).unwrap();editor.undo();
 let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::InterfaceDefinition);
 let count=Rc::new(Cell::new(0_u32));let alloc_count=count.clone();let cancel_count=count.clone();
 let mut r=move|_:C,_:&str|{alloc_count.set(alloc_count.get()+1);Ok(id(200+alloc_count.get()))};
 let e=initialize_interface_definition(tx,&id(5),&mut f,&mut r,move||cancel_count.get()>=7).unwrap_err();
 assert!(matches!(e.cause,InterfaceHelperCause::Cancelled));assert_eq!(root_attachments(&e.partial.applied_steps),3);
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(editor.redo());assert_eq!(f.allocate(C::ReferenceUsage).unwrap().semantic_id(),id(13));
}
#[test]
fn late_reference_semantic_collision_with_existing_connection_discards_both_ports() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();
 let mut f=factory([4,5,6,7,8,9,3,11].into_iter().map(|n|Ok(id(n))).collect());let tx=attached(&editor,&mut f,C::InterfaceDefinition);
 let e=initialize_interface_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap_err();
 assert!(matches!(e.cause,InterfaceHelperCause::Model(ModelError::DuplicateId(_))));assert_eq!(root_attachments(&e.partial.applied_steps),2);
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn fourth_endpoint_resource_collision_discards_three_attached_ends() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::InterfaceDefinition);
 let mut values=[201,202,203,204,205,206,201,208].into_iter();let mut r=move|_:C,_:&str|Ok(id(values.next().unwrap()));
 let e=initialize_interface_definition(tx,&id(5),&mut f,&mut r,||false).unwrap_err();
 assert!(matches!(e.cause,InterfaceHelperCause::Model(ModelError::DuplicateId(_))));assert_eq!(root_attachments(&e.partial.applied_steps),3);
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn late_factory_provider_failure_and_reuse_never_commit_partial_interface() {
 for duplicate in [false,true] {
  let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();
  let mut values=(4..13).map(|n|Ok(id(n))).collect::<Vec<_>>();values.push(if duplicate {Ok(id(6))} else {Err("late provider refusal".into())});
  let mut f=factory(values);let tx=attached(&editor,&mut f,C::InterfaceDefinition);
  let e=initialize_interface_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap_err();
  assert!(matches!(e.cause,InterfaceHelperCause::Factory(FactoryError::ReusedSemanticId)|InterfaceHelperCause::Factory(FactoryError::IdentityProvider(_))));
  assert_eq!(root_attachments(&e.partial.applied_steps),3);assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
 }
}
#[test]
fn late_resource_provider_refusal_discards_all_owned_interface_changes() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::InterfaceDefinition);
 let mut n=0;let mut r=move|_:C,_:&str|{n+=1;if n==7 {Err("late resource refusal".into())} else {Ok(id(200+n))}};
 let e=initialize_interface_definition(tx,&id(5),&mut f,&mut r,||false).unwrap_err();
 assert!(matches!(e.cause,InterfaceHelperCause::ResourceProvider(_)));assert_eq!(root_attachments(&e.partial.applied_steps),3);
 assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn connection_root_is_refused_before_helper_provider_consumption() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let mut f=normal();let tx=attached(&editor,&mut f,C::ConnectionDefinition);
 let mut n=0;let mut r=|_:C,_:&str|{n+=1;Ok(id(200+n))};
 let e=initialize_interface_definition(tx,&id(5),&mut f,&mut r,||false).unwrap_err();
 assert!(matches!(e.cause,InterfaceHelperCause::UnsupportedRoot(C::ConnectionDefinition)));assert_eq!(n,0);
 assert_eq!(f.allocate(C::PortUsage).unwrap().semantic_id(),id(6));assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn existing_relationships_are_preserved_and_four_source_ordered_ends_append() {
 let editor=Editor::new(base());let mut f=normal();let mut tx=attached(&editor,&mut f,C::InterfaceDefinition);
 let existing=node(40,"FeatureMembership",json!({"ownedRelatedElement":[node(41,"ReferenceUsage",json!({"declaredName":"existing-end"}))]}));
 tx.attach_preallocated_subtree(&id(5),F::OwnedRelationship,existing.clone()).unwrap();
 let (tx,_)=initialize_interface_definition(tx,&id(5),&mut f,&mut resources(),||false).unwrap();
 let rels=tx.document().element(&id(5)).unwrap()["data"]["ownedRelationship"].as_array().unwrap();assert_eq!(rels.len(),5);assert_eq!(rels[0],existing);
 for (i,name) in ["source","target","source","target"].into_iter().enumerate() {assert_eq!(rels[i+1]["data"]["ownedRelatedElement"][0]["data"]["declaredName"],name)}
}
#[test]
fn exact_interface_setter_preserves_generic_and_connection_refusals_and_transaction_poisoning() {
 let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();
 let mut generic=attached(&editor,&mut normal(),C::InterfaceDefinition);
 assert!(matches!(generic.set_attribute(&id(5),"isSufficient",json!(true)),Err(ModelError::Attribute(syson_model::AttributeError::UnresolvedAttributeSemantics {..}))));
 assert!(matches!(generic.initialize_interface_definition_sufficient(&id(5)),Err(ModelError::FailedTransaction)));assert!(matches!(editor.commit(generic),Err(ModelError::FailedTransaction)));
 let mut connection_only=attached(&editor,&mut normal(),C::InterfaceDefinition);
 assert!(matches!(connection_only.initialize_connection_definition_sufficient(&id(5)),Err(ModelError::WrongType {..})));assert!(matches!(editor.commit(connection_only),Err(ModelError::FailedTransaction)));
 let mut wrong=editor.begin();assert!(matches!(wrong.initialize_interface_definition_sufficient(&id(1)),Err(ModelError::WrongType {..})));assert!(matches!(editor.commit(wrong),Err(ModelError::FailedTransaction)));
 let mut missing=editor.begin();assert!(matches!(missing.initialize_interface_definition_sufficient(&id(999)),Err(ModelError::MissingElement(_))));assert!(matches!(editor.commit(missing),Err(ModelError::FailedTransaction)));
 let mut unknown=attached(&editor,&mut normal(),C::InterfaceDefinition);assert!(matches!(unknown.set_attribute(&id(5),"unsupportedAttribute",json!(true)),Err(ModelError::Attribute(syson_model::AttributeError::UnknownAttribute(_)))));assert!(matches!(editor.commit(unknown),Err(ModelError::FailedTransaction)));
 assert_eq!(editor.document().to_json().unwrap(),before);
 let mut exact=attached(&editor,&mut normal(),C::InterfaceDefinition);let mut expected=exact.document().element(&id(5)).unwrap().clone();
 exact.initialize_interface_definition_sufficient(&id(5)).unwrap();expected["data"]["isSufficient"]=json!(true);assert_eq!(exact.document().element(&id(5)).unwrap(),&expected);
 editor.commit(exact).unwrap();assert!(editor.undo());assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
