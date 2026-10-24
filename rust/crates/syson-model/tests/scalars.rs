// Copyright (c) 2024, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use syson_metamodel::{ClassifierId, ClassifierKind, DOMAIN_CLASSIFIERS};
use syson_model::*;
fn document(class: &str) -> Document {
    Document::from_json(&json!({"json":{"version":"1.0","encoding":"utf-8"},
        "ns":{"sysml":"http://www.eclipse.org/syson/sysml"},
        "content":[{"id":"resource","eClass":class,"data":{"elementId":"element","declaredName":"subject","unknownVendor":42}}]}).to_string()).unwrap()
}
fn edited(class: &str, name: &str, value: Value) -> std::result::Result<Document, ModelError> {
    let mut editor = Editor::new(document(class));
    let mut tx = editor.begin();
    tx.set_attribute("element", name, value)?;
    editor.commit(tx)?;
    Ok(editor.document().clone())
}
#[test]
fn all175_domain_class_attribute_contracts_can_be_inspected_without_materializing_defaults() {
    let mut count = 0;
    for classifier in DOMAIN_CLASSIFIERS
        .iter()
        .filter(|c| c.kind == ClassifierKind::Class)
    {
        let doc = document(&format!("sysml:{}", classifier.name()));
        let original = doc.clone();
        let info = doc.inspect_node("element").unwrap();
        assert_eq!(info.classifier, classifier.id);
        assert!(info.attributes.iter().any(|a| a.name == "declaredName"));
        assert!(
            info.attributes.iter().any(
                |a| a.name == "elementId" && a.blocked == Some(AttributeBlock::IdentityManaged)
            )
        );
        assert_eq!(doc, original);
        count += 1;
    }
    assert_eq!(count, 175);
}
#[test]
fn scalar_descriptor_preserves_origin_default_absence_and_unknown_fields() {
    let doc = document("sysml:PartUsage");
    let info = doc.inspect_node("element").unwrap();
    let composite = info
        .attributes
        .iter()
        .find(|a| a.name == "isComposite")
        .unwrap();
    assert_eq!(composite.declared_by, ClassifierId::Feature);
    assert_eq!(composite.explicit_default_literal, Some("false"));
    assert_eq!(composite.value, None);
    assert_eq!(composite.value_type, Some(ScalarType::Boolean));
    assert!(composite.writable());
    assert!(info.writable_attributes().any(|a| a.name == "declaredName"));
    assert!(!info.unresolved_supertypes.is_empty());
    assert_eq!(doc.value()["content"][0]["data"].get("isComposite"), None);
    let changed = edited("sysml:PartUsage", "isComposite", json!(false)).unwrap();
    assert_eq!(
        changed.element("element").unwrap()["data"]["unknownVendor"],
        42
    );
}
#[test]
fn generic_string_boolean_and_null_edits_match_typed_attribute_storage() {
    let name = edited("sysml:PartUsage", "declaredName", json!("Hello '世界'")).unwrap();
    assert_eq!(
        name.element("element").unwrap()["data"]["declaredName"],
        "Hello '世界'"
    );
    let cleared = edited("sysml:PartUsage", "declaredName", Value::Null).unwrap();
    assert!(cleared.element("element").unwrap()["data"]["declaredName"].is_null());
    assert_eq!(
        edited("sysml:PartUsage", "isComposite", json!(true))
            .unwrap()
            .element("element")
            .unwrap()["data"]["isComposite"],
        true
    );
    assert!(matches!(
        edited("sysml:PartUsage", "isComposite", json!("true")),
        Err(ModelError::Attribute(AttributeError::WrongValueType { .. }))
    ));
    assert!(matches!(
        edited("sysml:PartUsage", "isComposite", Value::Null),
        Err(ModelError::Attribute(AttributeError::RequiredValue { .. }))
    ));
    assert!(matches!(
        edited("sysml:LiteralString", "value", Value::Null),
        Err(ModelError::Attribute(AttributeError::RequiredValue { .. }))
    ));
}
#[test]
fn enum_representation_is_exact_literal_string_and_reset_is_explicitly_unsupported() {
    let info = document("sysml:PartUsage").inspect_node("element").unwrap();
    let direction = info
        .attributes
        .iter()
        .find(|a| a.name == "direction")
        .unwrap();
    assert!(direction.unsettable);
    assert!(!direction.accepts_null());
    assert_eq!(
        direction.value_type,
        Some(ScalarType::Enumeration {
            classifier: ClassifierId::FeatureDirectionKind,
            literals: vec!["in", "inout", "out"]
        })
    );
    for value in ["in", "inout", "out"] {
        assert_eq!(
            edited("sysml:PartUsage", "direction", json!(value))
                .unwrap()
                .element("element")
                .unwrap()["data"]["direction"],
            value
        );
    }
    assert!(matches!(
        edited("sysml:PartUsage", "direction", json!(1)),
        Err(ModelError::Attribute(AttributeError::WrongValueType { .. }))
    ));
    assert!(matches!(
        edited("sysml:PartUsage", "direction", json!("IN")),
        Err(ModelError::Attribute(
            AttributeError::InvalidEnumLiteral { .. }
        ))
    ));
    assert!(matches!(
        edited("sysml:PartUsage", "direction", Value::Null),
        Err(ModelError::Attribute(
            AttributeError::UnsupportedNullReset { .. }
        ))
    ));
    assert!(matches!(
        edited("sysml:Membership", "visibility", Value::Null),
        Err(ModelError::Attribute(
            AttributeError::UnsupportedNullReset { .. }
        ))
    ));
}
#[test]
fn java_int_boundaries_and_double_normalization_are_enforced() {
    for n in [i32::MIN, i32::MAX] {
        assert_eq!(
            edited("sysml:LiteralInteger", "value", json!(n))
                .unwrap()
                .element("element")
                .unwrap()["data"]["value"],
            n
        );
    }
    for n in [
        json!(i64::from(i32::MAX) + 1),
        json!(i64::from(i32::MIN) - 1),
        json!(1.0),
        json!("1"),
        Value::Null,
    ] {
        assert!(edited("sysml:LiteralInteger", "value", n).is_err());
    }
    let input: Value = serde_json::from_str("1.2345678901234567890123456789").unwrap();
    let expected = input.as_f64().unwrap();
    let doc = edited("sysml:LiteralRational", "value", input).unwrap();
    assert_eq!(
        doc.element("element").unwrap()["data"]["value"],
        json!(expected)
    );
    let huge: Value = serde_json::from_str("1e99999").unwrap();
    assert!(matches!(
        edited("sysml:LiteralRational", "value", huge),
        Err(ModelError::Attribute(AttributeError::WrongValueType { .. }))
    ));
}
#[test]
fn derived_many_valued_identity_and_unknown_edits_reject_with_distinct_diagnostics() {
    assert!(matches!(
        edited("sysml:PartUsage", "name", json!("x")),
        Err(ModelError::Attribute(AttributeError::Blocked {
            reason: AttributeBlock::Derived,
            ..
        }))
    ));
    assert!(matches!(
        edited("sysml:PartUsage", "aliasIds", json!(["x"])),
        Err(ModelError::Attribute(AttributeError::Blocked {
            reason: AttributeBlock::ManyValued,
            ..
        }))
    ));
    assert!(matches!(
        edited("sysml:PartUsage", "elementId", json!("other")),
        Err(ModelError::Attribute(AttributeError::Blocked {
            reason: AttributeBlock::IdentityManaged,
            ..
        }))
    ));
    assert!(matches!(
        edited("sysml:PartUsage", "unknownVendor", json!(7)),
        Err(ModelError::Attribute(AttributeError::UnknownAttribute(_)))
    ));
    assert!(matches!(
        edited("sysml:UnknownClass", "declaredName", json!("x")),
        Err(ModelError::UnsupportedClass(_))
    ));
}
#[test]
fn crossreference_and_containment_editing_are_explicitly_unsupported() {
    assert!(matches!(
        edited("sysml:PartUsage", "type", json!("target")),
        Err(ModelError::Attribute(
            AttributeError::ReferenceEditUnsupported {
                containment: false,
                ..
            }
        ))
    ));
    assert!(matches!(
        edited("sysml:PartUsage", "ownedRelationship", json!([])),
        Err(ModelError::Attribute(
            AttributeError::ReferenceEditUnsupported {
                containment: true,
                ..
            }
        ))
    ));
    let info = document("sysml:PartUsage").inspect_node("element").unwrap();
    assert!(
        info.diagnostics
            .iter()
            .any(|d| d.name == "ownedRelationship")
    );
    assert!(
        document("sysmlcustomnodes:SysMLPackageNodeStyleDescription")
            .inspect_node("element")
            .is_err()
    );
}
#[test]
fn scalar_error_after_success_poison_rolls_back_whole_transaction_and_history() {
    let mut editor = Editor::new(document("sysml:PartUsage"));
    let original = editor.document().clone();
    let mut tx = editor.begin();
    tx.set_attribute("element", "declaredName", json!("staged"))
        .unwrap();
    assert!(
        tx.set_attribute("element", "direction", json!(false))
            .is_err()
    );
    assert_eq!(editor.commit(tx), Err(ModelError::FailedTransaction));
    assert_eq!(*editor.document(), original);
    assert!(!editor.undo());
}
#[test]
fn scalar_history_cancel_and_roundtrip_do_not_materialize_other_fields() {
    let mut editor = Editor::new(document("sysml:PartUsage"));
    let original = editor.document().clone();
    let mut tx = editor.begin();
    tx.set_attribute("element", "direction", json!("out"))
        .unwrap();
    editor.commit(tx).unwrap();
    let changed = editor.document().clone();
    assert_eq!(
        Document::from_json(&changed.to_json().unwrap()).unwrap(),
        changed
    );
    assert!(
        changed.element("element").unwrap()["data"]
            .get("isComposite")
            .is_none()
    );
    assert!(editor.undo());
    assert_eq!(*editor.document(), original);
    assert!(editor.redo());
    assert_eq!(*editor.document(), changed);
    let mut tx = editor.begin();
    tx.set_attribute("element", "isComposite", json!(false))
        .unwrap();
    tx.cancel();
    assert_eq!(*editor.document(), changed);
}
#[test]
fn strict_projection_query_uses_full_transitive_type_lattice() {
    let projection = document("sysml:ActionUsage").projection().unwrap();
    for ty in [
        ClassifierId::ActionUsage,
        ClassifierId::Usage,
        ClassifierId::Feature,
        ClassifierId::Type,
        ClassifierId::Namespace,
        ClassifierId::Element,
    ] {
        assert_eq!(
            projection.find_by_type("subject", ty).unwrap(),
            Some("element")
        );
    }
    assert_eq!(
        projection
            .find_by_type("subject", ClassifierId::PartDefinition)
            .unwrap(),
        None
    );
    for class in [
        "UnknownClass",
        "PartUsage",
        "other:PartUsage",
        "sysml:UnknownClass",
    ] {
        let projection = document(class).projection().unwrap();
        assert_eq!(
            projection
                .find_by_type("subject", ClassifierId::PartUsage)
                .unwrap(),
            None
        );
    }
}

#[test]
fn unresolved_external_branch_is_diagnostic_not_a_guessed_writable_attribute() {
    let doc = document("sysml:AcceptActionUsage");
    let info = doc.inspect_node("element").unwrap();
    assert!(info.diagnostics.iter().any(|d| d.name == "isIndividual"
        && matches!(d.issue, AttributeError::UnresolvedAttributeSemantics { .. })));
    assert!(!info.writable_attributes().any(|a| a.name == "isIndividual"));
    assert!(matches!(
        edited("sysml:AcceptActionUsage", "isIndividual", json!(true)),
        Err(ModelError::Attribute(
            AttributeError::UnresolvedAttributeSemantics { .. }
        ))
    ));
}
