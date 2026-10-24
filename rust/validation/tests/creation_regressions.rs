// SPDX-License-Identifier: EPL-2.0
//! Source-derived membership/type expectations; synthetic objects isolate command rules.
use serde_json::{Value, json};
use syson_model::{Document, Editor, ModelError, NodeIds};
fn ids(n: u32) -> NodeIds {
    NodeIds::new(
        &format!("00000000-0000-4000-8000-{n:012x}"),
        &format!("10000000-0000-4000-8000-{n:012x}"),
    )
    .unwrap()
}
fn document(class: &str) -> Document {
    Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{"id":"parent-resource","eClass":class,"data":{"elementId":"parent","declaredName":"Parent","vendor":42}}]}).to_string()).unwrap()
}
fn matrix() -> Value {
    serde_json::from_str(include_str!(
        "../../../porting/validation/fixtures/source-membership-matrix.json"
    ))
    .unwrap()
}
#[test]
fn all_source_element_contexts_apply_exact_membership_rule_for_three_constructors() {
    let source = matrix();
    let contexts = source["contexts"].as_array().unwrap();
    assert_eq!(contexts.len(), 175);
    let mut count = 0;
    for context in contexts {
        let class = format!("sysml:{}", context["class"].as_str().unwrap());
        for kind in ["package", "definition", "usage"] {
            let mut editor = Editor::new(document(&class));
            assert_eq!(
                editor.document().can_create_in("parent"),
                context["element"].as_bool().unwrap()
            );
            let mut tx = editor.begin();
            match kind {
                "package" => tx.create_package(Some("parent"), ids(1), Some(ids(2)), "Child"),
                "definition" => tx.create_part_definition("parent", ids(1), ids(2), "Child"),
                _ => tx.create_part_usage("parent", ids(1), ids(2), "Child"),
            }
            .unwrap();
            editor.commit(tx).unwrap();
            let expected = if kind == "usage" && context["type"] == true {
                "sysml:FeatureMembership"
            } else {
                "sysml:OwningMembership"
            };
            let relation = editor.document().element(&ids(2).element).unwrap();
            assert_eq!(relation["eClass"], expected, "{class} {kind}");
            assert_eq!(
                relation["data"]["ownedRelatedElement"][0]["data"]["elementId"],
                ids(1).element
            );
            assert_eq!(
                editor.document().element("parent").unwrap()["data"]["vendor"],
                42
            );
            assert_eq!(
                Document::from_json(&editor.document().to_json().unwrap()).unwrap(),
                *editor.document()
            );
            count += 1;
        }
    }
    assert_eq!(count, 525);
}
#[test]
fn source_part_usage_and_definition_subtypes_support_typing_pairs() {
    let source = matrix();
    let contexts = source["contexts"].as_array().unwrap();
    let mut pairs = 0;
    for usage in contexts.iter().filter(|row| row["part_usage"] == true) {
        for definition in contexts.iter().filter(|row| row["part_definition"] == true) {
            let mut value = document(&format!("sysml:{}", usage["class"].as_str().unwrap()))
                .value()
                .clone();
            value["content"].as_array_mut().unwrap().push(json!({"id":"definition-resource","eClass":format!("sysml:{}",definition["class"].as_str().unwrap()),"data":{"elementId":"definition","declaredName":"Definition"}}));
            let mut editor = Editor::new(Document::from_json(&value.to_string()).unwrap());
            let mut tx = editor.begin();
            tx.type_part_usage("parent", "definition", ids(3)).unwrap();
            editor.commit(tx).unwrap();
            let relation = editor.document().element(&ids(3).element).unwrap();
            assert_eq!(relation["eClass"], "sysml:FeatureTyping");
            assert_eq!(relation["data"]["type"], "definition");
            assert_eq!(relation["data"]["typedFeature"], "parent");
            pairs += 1;
        }
    }
    assert!(pairs > 0);
    eprintln!("independent source-derived polymorphic typing pairs accepted: {pairs}");
}
#[test]
fn unresolved_parent_or_endpoint_never_acquires_capability_and_errors_are_atomic() {
    for class in [
        "ActionUsage",
        "foreign:ActionUsage",
        "sysml:Unknown",
        "sysml:VisibilityKind",
    ] {
        let mut editor = Editor::new(document(class));
        assert!(!editor.document().can_create_in("parent"));
        let original = editor.document().clone();
        let mut tx = editor.begin();
        tx.create_package(None, ids(10), None, "StagedRoot")
            .unwrap();
        let staged = tx.document().clone();
        assert!(
            tx.create_part_usage("parent", ids(1), ids(2), "Child")
                .is_err()
        );
        assert_eq!(tx.document(), &staged);
        assert_eq!(editor.commit(tx), Err(ModelError::FailedTransaction));
        assert_eq!(editor.document(), &original);
        assert!(!editor.undo());
    }
    assert!(!document("sysml:ActionUsage").can_create_in("missing"));
}
