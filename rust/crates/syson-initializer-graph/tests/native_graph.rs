use serde_json::{Value,json};
use syson_initializer::*;
use syson_initializer_graph::*;
use syson_metamodel::ClassifierId as C;
use syson_model::{Document,Editor};

fn node(id:&str,class:&str,data:Value)->Value {
    let mut data=data.as_object().unwrap().clone();
    data.insert("elementId".into(),json!(id));
    json!({"id":format!("resource-{id}"),"eClass":format!("sysml:{class}"),"data":data})
}
fn document(root:Value)->Document {
    Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[root]}).to_string()).unwrap()
}
fn fixture()->Document {
    document(node("namespace","Package",json!({"ownedRelationship":[
        node("owning","OwningMembership",json!({"ownedRelatedElement":[node("root","PartUsage",json!({}))]})),
        node("broad","Membership",json!({"ownedRelatedElement":[node("sibling","PartUsage",json!({}))]})),
        node("other","OwningMembership",json!({"ownedRelatedElement":[node("subtype","ReferenceUsage",json!({}))]}))
    ]})))
}
fn planned(doc:&Document)->Plan {
    let mut context=NativeContext::new(doc,"root").unwrap();
    match plan(C::PartUsage,&mut context) { Outcome::Planned(p)=>p,other=>panic!("{other:?}") }
}
#[test]
fn raw_membership_namespace_owner_and_exact_count_are_distinct() {
    let doc=fixture(); let i=Index::new(&doc).unwrap();
    assert_eq!(i.owner("root").unwrap().unwrap().element_id,"namespace");
    assert_eq!(i.owning_membership("root").unwrap().unwrap().element_id,"owning");
    assert_eq!(i.owning_namespace("root").unwrap().unwrap().element_id,"namespace");
    assert!(i.owning_membership("sibling").unwrap().is_none());
    assert!(i.owning_namespace("sibling").unwrap().is_none());
    assert!(i.owner("owning").unwrap().is_none());
    assert_eq!(i.exact_class_owned_member_count("namespace",C::PartUsage).unwrap(),2);
    assert_eq!(i.nodes["root"].resource_id,"resource-root");
    assert!(i.owner("resource-root").is_err());
}
#[test]
fn native_plan_applies_one_atomic_history_entry_and_roundtrips_undo_redo() {
    let doc=fixture(); let before=doc.to_json().unwrap(); let p=planned(&doc);
    let doc_copy=doc.clone(); let mut editor=Editor::new(doc);
    assert!(apply(&mut editor,&PreparedPlan::bind(&doc_copy,"root",p.clone()).unwrap(),false).unwrap());
    assert_eq!(editor.document().element("root").unwrap()["data"]["declaredName"],"part2");
    assert_eq!(editor.document().element("root").unwrap()["data"]["isComposite"],true);
    let after=editor.document().to_json().unwrap();
    assert!(editor.undo()); assert_eq!(editor.document().to_json().unwrap(),before);
    assert!(!editor.undo()); assert!(editor.redo()); assert_eq!(editor.document().to_json().unwrap(),after);
}
#[test]
fn prefix_replay_observes_new_name_without_mutating_original() {
    let doc=fixture(); let before=doc.to_json().unwrap(); let p=planned(&doc);
    let mut c=NativeContext::new(&doc,"root").unwrap();
    assert_eq!(c.effective_name(&ExternalObject("root".into()),&p.steps),Resolution::Known(Some("part2".into())));
    assert_eq!(doc.to_json().unwrap(),before);
}
#[test]
fn invalid_late_scalar_discards_all_edits_and_preserves_redo() {
    let doc=fixture(); let p=planned(&doc); let snapshot=doc.clone(); let mut e=Editor::new(doc);
    apply(&mut e,&PreparedPlan::bind(&snapshot,"root",p.clone()).unwrap(),false).unwrap(); assert!(e.undo());
    let before=e.document().to_json().unwrap(); let mut invalid=p.clone();
    invalid.steps.push(Step { source_method:"test", effect:Effect::SetScalar {target:Target::Root,feature:ScalarFeature::IsComposite,value:Scalar::Text("invalid".into())} });
    assert!(matches!(apply(&mut e,&PreparedPlan::bind(&snapshot,"root",invalid).unwrap(),false),Err(Error::Model(_))));
    assert_eq!(e.document().to_json().unwrap(),before); assert!(e.redo());
}
#[test]
fn cancellation_and_class_mismatch_leave_document_and_history_untouched() {
    let doc=fixture(); let mut p=planned(&doc); let snapshot=doc.clone(); let mut e=Editor::new(doc);
    let before=e.document().to_json().unwrap();
    assert!(matches!(apply(&mut e,&PreparedPlan::bind(&snapshot,"root",p.clone()).unwrap(),true),Err(Error::Cancelled)));
    p.classifier=C::Package;
    assert!(matches!(PreparedPlan::bind(&snapshot,"root",p),Err(Error::ClassMismatch)));
    assert_eq!(e.document().to_json().unwrap(),before); assert!(!e.undo());
}
#[test]
fn unsupported_effect_after_scalar_cannot_partially_commit() {
    let doc=fixture(); let mut p=planned(&doc); let snapshot=doc.clone(); let mut e=Editor::new(doc);
    let before=e.document().to_json().unwrap();
    p.steps.push(Step {source_method:"test",effect:Effect::SetReference {target:Target::Root,feature:ReferenceFeature::Type,value:None}});
    assert!(matches!(apply(&mut e,&PreparedPlan::bind(&snapshot,"root",p.clone()).unwrap(),false),Err(Error::Gap(_))));
    assert_eq!(e.document().to_json().unwrap(),before); assert!(!e.undo());
}
#[test]
fn declared_short_name_suppresses_missing_declared_name_and_redefinition_is_explicit_gap() {
    let d=document(node("root","PartUsage",json!({"declaredShortName":"short"})));
    let mut c=NativeContext::new(&d,"root").unwrap();
    assert_eq!(c.effective_name(&ExternalObject("root".into()),&[]),Resolution::Known(None));
    let d=document(node("root","PartUsage",json!({"ownedRelationship":[node("r","Redefinition",json!({"redefinedFeature":"elsewhere"}))]})));
    let mut c=NativeContext::new(&d,"root").unwrap();
    assert!(matches!(c.effective_name(&ExternalObject("root".into()),&[]),Resolution::Unsupported(_)));
    assert!(matches!(c.general_view_definition(&[]),Resolution::Unresolved(_)));
}
#[test]
fn namespace_is_immediate_membership_container_not_nearest_ancestor() {
    let d=document(node("namespace","Package",json!({"ownedRelationship":[
        node("outer","OwningMembership",json!({"ownedRelatedElement":[
            node("inner","OwningMembership",json!({"ownedRelatedElement":[node("root","PartUsage",json!({}))]}))
        ]}))
    ]})));
    assert!(Index::new(&d).unwrap().owning_namespace("root").unwrap().is_none());
}

#[test]
fn snapshot_bound_plan_rejects_intervening_edit() {
    let doc=fixture(); let prepared=PreparedPlan::bind(&doc,"root",planned(&doc)).unwrap();
    let mut e=Editor::new(doc); let mut tx=e.begin();
    tx.set_attribute("root","declaredName",json!("intervening")).unwrap();
    e.commit(tx).unwrap(); let before=e.document().to_json().unwrap();
    assert!(matches!(apply(&mut e,&prepared,false),Err(Error::StalePlan)));
    assert_eq!(e.document().to_json().unwrap(),before);
}
