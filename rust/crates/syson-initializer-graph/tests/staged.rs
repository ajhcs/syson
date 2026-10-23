use serde_json::{json,Value};
use syson_initializer::*;
use syson_initializer_graph::*;
use syson_metamodel::ClassifierId as C;
use syson_model::{ContainmentFeature as F,Document,Editor,Transaction};
fn id(n:u32)->String {format!("b0000000-0000-4000-8000-{n:012x}")}
fn node(n:u32,class:&str,extra:Value)->Value {
    let mut data=extra.as_object().unwrap().clone();data.insert("elementId".into(),json!(id(n)));
    json!({"id":id(n+100),"eClass":format!("sysml:{class}"),"data":data})
}
fn base()->Document {
    Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[node(1,"Package",json!({"declaredName":"package","ownedRelationship":[node(2,"OwningMembership",json!({"ownedRelatedElement":[node(3,"PartUsage",json!({"declaredName":"existing"}))]}))]}))]}).to_string()).unwrap()
}
fn attached(editor:&Editor,class:&str)->Transaction {
    let mut tx=editor.begin();
    tx.attach_preallocated_subtree(&id(1),F::OwnedRelationship,node(4,"OwningMembership",json!({"ownedRelatedElement":[node(5,class,json!({}))]}))).unwrap();
    tx
}
fn staged_plan(tx:&Transaction)->Plan {
    let mut context=NativeContext::new(tx.document(),&id(5)).unwrap();
    match plan(C::PartUsage,&mut context) {Outcome::Planned(p)=>p,other=>panic!("{other:?}")}
}
#[test]
fn actual_attach_query_defaults_single_commit_and_undo_redo() {
    let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();
    let tx=attached(&editor,"PartUsage");
    let mut context=NativeContext::new(tx.document(),&id(5)).unwrap();
    assert_eq!(context.exact_class_owned_member_count(&ExternalObject(id(1)),C::PartUsage,&[]),Resolution::Known(2));
    let tx=initialize_staged(tx,&id(5),ReferenceEncoding::Disabled,false).unwrap();
    assert!(editor.document().element(&id(5)).is_none());
    assert_eq!(tx.document().element(&id(5)).unwrap()["data"]["declaredName"],"part2");
    assert_eq!(tx.document().element(&id(5)).unwrap()["data"]["isComposite"],true);
    assert_eq!(tx.document().element(&id(3)).unwrap()["data"]["declaredName"],"existing");
    assert!(editor.commit(tx).unwrap());let after=editor.document().to_json().unwrap();
    assert!(editor.undo());assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
    assert!(editor.redo());assert_eq!(editor.document().to_json().unwrap(),after);
}
#[test]
fn cancellation_discards_owned_creation_and_defaults() {
    let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let tx=attached(&editor,"PartUsage");
    assert!(matches!(initialize_staged(tx,&id(5),ReferenceEncoding::Disabled,true),Err(Error::Cancelled)));
    assert_eq!(editor.document().to_json().unwrap(),before);assert!(!editor.undo());
}
#[test]
fn late_scalar_failure_discards_owned_creation_and_preserves_redo() {
    let mut editor=Editor::new(base());let mut first=editor.begin();first.rename(&id(1),"earlier").unwrap();editor.commit(first).unwrap();editor.undo();
    let before=editor.document().to_json().unwrap();let tx=attached(&editor,"PartUsage");let mut p=staged_plan(&tx);
    p.steps.push(Step {source_method:"negative",effect:Effect::SetScalar {target:Target::Root,feature:ScalarFeature::IsComposite,value:Scalar::Text("invalid".into())}});
    assert!(matches!(apply_staged_plan(tx,&id(5),&p,false),Err(Error::Model(_))));
    assert_eq!(editor.document().to_json().unwrap(),before);assert!(editor.document().element(&id(5)).is_none());assert!(editor.redo());
}
#[test]
fn late_reference_refusal_discards_creation_after_supported_defaults() {
    let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let tx=attached(&editor,"PartUsage");let mut p=staged_plan(&tx);
    p.steps.push(Step {source_method:"negative",effect:Effect::SetReference {target:Target::Root,feature:ReferenceFeature::Type,value:None}});
    assert!(matches!(apply_staged_plan(tx,&id(5),&p,false),Err(Error::Gap(_))));
    assert_eq!(editor.document().to_json().unwrap(),before);assert!(editor.document().element(&id(5)).is_none());assert!(!editor.undo());
}
#[test]
fn raw_allocation_requirement_refuses_instead_of_committing_partial_creation() {
    let mut editor=Editor::new(base());let before=editor.document().to_json().unwrap();let tx=attached(&editor,"PortDefinition");
    assert!(initialize_staged(tx,&id(5),ReferenceEncoding::Disabled,false).is_err());
    assert_eq!(editor.document().to_json().unwrap(),before);assert!(editor.document().element(&id(5)).is_none());assert!(!editor.undo());
}
