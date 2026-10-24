use serde_json::{json,Value};
use syson_initializer::*;
use syson_initializer_graph::*;
use syson_model::Document;
fn node(id:&str,class:&str,extra:Value)->Value {
    let mut data=extra.as_object().unwrap().clone();data.insert("elementId".into(),json!(id));
    json!({"id":format!("resource-{id}"),"eClass":format!("sysml:{class}"),"data":data})
}
fn reference(id:&str,target:Value)->Value {node(id,"Redefinition",json!({"redefinedFeature":target}))}
fn feature(id:&str,relations:Vec<Value>,extra:Value)->Value {
    let mut data=extra.as_object().unwrap().clone();data.insert("ownedRelationship".into(),json!(relations));
    node(id,"PartUsage",Value::Object(data))
}
fn doc(roots:Vec<Value>)->Document {
    Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":roots}).to_string()).unwrap()
}
fn context(d:&Document)->NativeContext {
    let mut c=NativeContext::new(d,"root").unwrap();c.reference_encoding=ReferenceEncoding::SemanticElementIds;c
}
#[test]
fn first_redefinition_uses_raw_order_instead_of_lexical_identity_order() {
    let d=doc(vec![
        feature("root",vec![reference("z-first",json!("target1")),reference("a-second",json!("target2"))],json!({})),
        feature("target1",vec![],json!({"declaredName":"first"})),
        feature("target2",vec![],json!({"declaredName":"second"}))]);
    assert_eq!(context(&d).effective_name(&ExternalObject("root".into()),&[]),Resolution::Known(Some("first".into())));
}
#[test]
fn local_chain_observes_scalar_prefix_and_preserves_input_document() {
    let d=doc(vec![
        feature("root",vec![reference("r1",json!("middle"))],json!({})),
        feature("middle",vec![reference("r2",json!("target"))],json!({})),
        feature("target",vec![],json!({"declaredName":"before"}))]);
    let before=d.to_json().unwrap();
    let steps=[Step {source_method:"test",effect:Effect::SetScalar {target:Target::External(ExternalObject("target".into())),feature:ScalarFeature::DeclaredName,value:Scalar::Text("after".into())}}];
    assert_eq!(context(&d).effective_name(&ExternalObject("root".into()),&steps),Resolution::Known(Some("after".into())));
    assert_eq!(d.to_json().unwrap(),before);
}
#[test]
fn first_null_redefinition_does_not_fall_through_to_later_reference() {
    let d=doc(vec![
        feature("root",vec![reference("r1",Value::Null),reference("r2",json!("target"))],json!({})),
        feature("target",vec![],json!({"declaredName":"later"}))]);
    assert_eq!(context(&d).effective_name(&ExternalObject("root".into()),&[]),Resolution::Known(None));
}
#[test]
fn absent_reference_and_self_reference_are_known_null() {
    for relation in [node("r","Redefinition",json!({})),reference("r",json!("root"))] {
        let d=doc(vec![feature("root",vec![relation],json!({}))]);
        assert_eq!(context(&d).effective_name(&ExternalObject("root".into()),&[]),Resolution::Known(None));
    }
}
#[test]
fn long_and_short_names_share_the_source_non_null_branch_condition() {
    for (data,long,short) in [
        (json!({"declaredName":"long"}),Some("long"),None),
        (json!({"declaredShortName":"short"}),None,Some("short")),
        (json!({"declaredName":"","declaredShortName":null}),Some(""),None),
    ] {
        let d=doc(vec![feature("root",vec![reference("r",json!("unresolved"))],data)]);
        let i=Index::new(&d).unwrap();
        assert_eq!(i.effective_name_with_references("root",NameKind::Name,ReferenceEncoding::SemanticElementIds).unwrap(),long.map(str::to_owned));
        assert_eq!(i.effective_name_with_references("root",NameKind::ShortName,ReferenceEncoding::SemanticElementIds).unwrap(),short.map(str::to_owned));
    }
}
#[test]
fn missing_foreign_and_resource_ids_never_resolve_as_semantic_ids() {
    for target in ["missing","remote#target","resource-target"] {
        let d=doc(vec![feature("root",vec![reference("r",json!(target))],json!({})),feature("target",vec![],json!({"declaredName":"target"}))]);
        assert!(matches!(context(&d).effective_name(&ExternalObject("root".into()),&[]),Resolution::Unresolved(_)));
    }
}
#[test]
fn non_feature_and_proxy_wire_shapes_are_explicitly_unsupported() {
    let d=doc(vec![feature("root",vec![reference("r",json!("namespace"))],json!({})),node("namespace","Package",json!({}))]);
    assert!(matches!(context(&d).effective_name(&ExternalObject("root".into()),&[]),Resolution::Unsupported(_)));
    let d=doc(vec![feature("root",vec![reference("r",json!({"$ref":"elsewhere"}))],json!({}))]);
    assert!(matches!(context(&d).effective_name(&ExternalObject("root".into()),&[]),Resolution::Unsupported(_)));
}
#[test]
fn naming_cycle_is_reported_without_fabricated_absence_or_recursive_overflow() {
    let d=doc(vec![
        feature("root",vec![reference("r1",json!("other"))],json!({})),
        feature("other",vec![reference("r2",json!("root"))],json!({}))]);
    assert!(matches!(context(&d).effective_name(&ExternalObject("root".into()),&[]),Resolution::Unsupported(ContextGap {contract:"namingFeature",..})));
}

#[test]
fn malformed_declared_short_name_is_rejected_instead_of_known_absence() {
    for extra in [json!({"declaredShortName":false}),json!({"declaredName":"name","declaredShortName":27}),json!({"declaredName":[]})] {
        let d=doc(vec![feature("root",vec![],extra)]);
        assert!(matches!(context(&d).effective_name(&ExternalObject("root".into()),&[]),Resolution::Unsupported(ContextGap {contract:"effectiveName",..})));
    }
}
