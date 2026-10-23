// SPDX-License-Identifier: EPL-2.0
//! Independent tests of the documented typed-adapter subset, not Java eSet parity.
use serde_json::{Value, json};
use syson_metamodel::{ClassifierId, ClassifierKind, DOMAIN_CLASSIFIERS};
use syson_model::*;

fn document(class: &str) -> Document {
    Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"vendor":{"retain":[1,null,"é"]},"content":[{"id":"resource-id","eClass":class,"data":{"elementId":"stable-id","declaredName":"subject","vendor":123456}}]}).to_string()).unwrap()
}
fn edit(class: &str, attribute: &str, value: Value) -> Result<Document, ModelError> {
    let mut editor = Editor::new(document(class));
    let mut tx = editor.begin();
    tx.set_attribute("stable-id", attribute, value)?;
    editor.commit(tx)?;
    Ok(editor.document().clone())
}
#[test]
fn integer_lexical_and_range_boundaries_are_strict() {
    for raw in ["2147483647", "-2147483648", "-0", "0"] {
        let input: Value = serde_json::from_str(raw).unwrap();
        let expected = input.as_i64().unwrap();
        let doc = edit("sysml:LiteralInteger", "value", input).unwrap();
        assert_eq!(
            doc.element("stable-id").unwrap()["data"]["value"].as_i64(),
            Some(expected)
        );
    }
    for raw in [
        "2147483648",
        "-2147483649",
        "1.0",
        "1e0",
        "1E+0",
        "-0.0",
        "\"0\"",
        "true",
        "null",
        "[]",
        "{}",
    ] {
        assert!(
            edit(
                "sysml:LiteralInteger",
                "value",
                serde_json::from_str(raw).unwrap()
            )
            .is_err(),
            "accepted {raw}"
        );
    }
}
#[test]
fn double_normalization_preserves_finite_extremes_and_signed_underflow() {
    for (raw, expected) in [
        ("0", 0.0),
        ("-0.0", -0.0),
        ("1e-9999", 0.0),
        ("-1e-9999", -0.0),
        ("5e-324", f64::from_bits(1)),
        ("1.7976931348623157e308", f64::MAX),
        ("9007199254740993", 9007199254740992.0),
    ] {
        let doc = edit(
            "sysml:LiteralRational",
            "value",
            serde_json::from_str(raw).unwrap(),
        )
        .unwrap();
        let actual = doc.element("stable-id").unwrap()["data"]["value"]
            .as_f64()
            .unwrap();
        assert_eq!(actual.to_bits(), expected.to_bits(), "normalized {raw}");
        assert_eq!(Document::from_json(&doc.to_json().unwrap()).unwrap(), doc);
    }
    for raw in ["1e9999", "-1e9999", "\"1.2\"", "true", "null", "[]", "{}"] {
        assert!(
            edit(
                "sysml:LiteralRational",
                "value",
                serde_json::from_str(raw).unwrap()
            )
            .is_err()
        );
    }
}
#[test]
fn optional_and_required_strings_keep_explicit_null_separate_from_absence() {
    let before = document("sysml:PartUsage");
    assert!(
        before
            .inspect_node("stable-id")
            .unwrap()
            .attributes
            .iter()
            .find(|a| a.name == "declaredShortName")
            .unwrap()
            .value
            .is_none()
    );
    let cleared = edit("sysml:PartUsage", "declaredShortName", Value::Null).unwrap();
    assert_eq!(
        cleared
            .inspect_node("stable-id")
            .unwrap()
            .attributes
            .iter()
            .find(|a| a.name == "declaredShortName")
            .unwrap()
            .value,
        Some(Value::Null)
    );
    for text in ["", "a\0b\n\t", "é世界😀", "<script>&\"'"] {
        let doc = edit("sysml:LiteralString", "value", json!(text)).unwrap();
        assert_eq!(doc.element("stable-id").unwrap()["data"]["value"], text);
        assert_eq!(Document::from_json(&doc.to_json().unwrap()).unwrap(), doc);
    }
    assert!(edit("sysml:LiteralString", "value", Value::Null).is_err());
}
#[test]
fn every_advertised_writable_descriptor_is_accepted_without_touching_unrelated_data() {
    let mut class_count = 0;
    let mut descriptor_count = 0;
    let mut enum_types = std::collections::BTreeSet::new();
    for classifier in DOMAIN_CLASSIFIERS
        .iter()
        .filter(|c| c.kind == ClassifierKind::Class)
    {
        class_count += 1;
        let class = format!("sysml:{}", classifier.name());
        let initial = document(&class);
        let inspection = initial.inspect_node("stable-id").unwrap();
        for attribute in inspection.writable_attributes() {
            descriptor_count += 1;
            let value = match attribute.value_type.as_ref().unwrap() {
                ScalarType::String => json!("editable 世界"),
                ScalarType::Boolean => json!(true),
                ScalarType::Integer32 => json!(-2147483648i32),
                ScalarType::Double => json!(0.125),
                ScalarType::Enumeration {
                    classifier,
                    literals,
                } => {
                    enum_types.insert(*classifier);
                    json!(literals[0])
                }
            };
            let changed = edit(&class, attribute.name, value.clone())
                .unwrap_or_else(|e| panic!("advertised {class}.{} rejected {e}", attribute.name));
            assert_eq!(
                changed.element("stable-id").unwrap()["data"][attribute.name],
                value
            );
            assert_eq!(changed.value()["vendor"], initial.value()["vendor"]);
            assert_eq!(
                changed.element("stable-id").unwrap()["data"]["vendor"],
                123456
            );
            assert_eq!(changed.element("stable-id").unwrap()["id"], "resource-id");
            assert_eq!(
                Document::from_json(&changed.to_json().unwrap()).unwrap(),
                changed
            );
        }
    }
    assert_eq!(class_count, 175);
    assert!(descriptor_count > 0);
    assert_eq!(enum_types.len(), 7);
    eprintln!(
        "independent advertised writable descriptors accepted: {descriptor_count} across {class_count} classes and {} enum types",
        enum_types.len()
    );
}
#[test]
fn enum_reset_and_external_branch_failures_poison_the_whole_transaction() {
    for (class, attribute, bad) in [
        ("sysml:PartUsage", "direction", Value::Null),
        ("sysml:Membership", "visibility", Value::Null),
        ("sysml:AcceptActionUsage", "isIndividual", json!(true)),
        ("sysml:PartUsage", "ownedRelationship", json!([])),
        ("sysml:PartUsage", "elementId", json!("changed")),
    ] {
        let mut editor = Editor::new(document(class));
        let initial = editor.document().clone();
        let mut tx = editor.begin();
        tx.set_attribute("stable-id", "declaredName", json!("first staged edit"))
            .unwrap();
        let last_staged = tx.document().clone();
        assert!(tx.set_attribute("stable-id", attribute, bad).is_err());
        assert_eq!(tx.document(), &last_staged);
        assert_eq!(editor.commit(tx), Err(ModelError::FailedTransaction));
        assert_eq!(editor.document(), &initial);
        assert!(!editor.undo());
    }
}
#[test]
fn inherited_namespace_and_type_queries_use_registry_without_relabeling_imports() {
    let input = json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{"id":"r0","eClass":"sysml:AttributeDefinition","data":{"elementId":"root","ownedRelationship":[{"id":"r1","eClass":"sysml:OwningMembership","data":{"elementId":"m1","ownedRelatedElement":[{"id":"r2","eClass":"sysml:Package","data":{"elementId":"package","declaredName":"Pkg","ownedRelationship":[{"id":"r3","eClass":"sysml:OwningMembership","data":{"elementId":"m2","ownedRelatedElement":[{"id":"r4","eClass":"sysml:AttributeUsage","data":{"elementId":"usage","declaredName":"x"}}]}}]}}]}}]}}]});
    let doc = Document::from_json(&input.to_string()).unwrap();
    let projection = doc.projection().unwrap();
    assert_eq!(
        projection
            .find_by_type("Pkg::x", ClassifierId::Feature)
            .unwrap(),
        Some("usage")
    );
    for class in [
        "other:PartUsage",
        "PartUsage",
        "sysml:FeatureDirectionKind",
        "sysml:Unknown",
    ] {
        let doc = document(class);
        let before = doc.clone();
        assert!(doc.inspect_node("stable-id").is_err());
        assert_eq!(
            doc.projection()
                .unwrap()
                .find_by_type("subject", ClassifierId::Element)
                .unwrap(),
            None
        );
        assert_eq!(doc, before);
    }
}
