// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::sync::{Arc, OnceLock, atomic::AtomicBool};
use syson_model::{Document, Editor, NodeIds};
use syson_semantic_validation::{Error, NAMESPACE, PART, validate};
use syson_text_import::Importer;
const SOURCE: &str = "package Vehicle { part def Wheel; part def Car {part wheel:Wheel;} }";
fn flag() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}
fn ids(n: u32) -> NodeIds {
    NodeIds::new(
        &format!("10000000-0000-4000-8000-{n:012x}"),
        &format!("20000000-0000-4000-8000-{n:012x}"),
    )
    .unwrap()
}
fn imported(source: &str) -> Document {
    static I: OnceLock<Importer> = OnceLock::new();
    I.get_or_init(|| Importer::pinned().unwrap())
        .stage(
            Editor::new(Document::empty()).begin(),
            source,
            None,
            (1..300).map(ids),
            flag(),
        )
        .unwrap()
        .document()
        .clone()
}
fn value(d: &Document) -> Value {
    serde_json::from_str(&d.to_json().unwrap()).unwrap()
}
fn document(v: Value) -> Document {
    Document::from_json(&v.to_string()).unwrap()
}
fn named<'a>(v: &'a mut Value, name: &str) -> &'a mut Value {
    fn find<'a>(v: &'a mut Value, name: &str) -> Option<&'a mut Value> {
        if v["data"]["declaredName"] == name {
            return Some(v);
        }
        match v {
            Value::Array(a) => a.iter_mut().find_map(|v| find(v, name)),
            Value::Object(o) => o.values_mut().find_map(|v| find(v, name)),
            _ => None,
        }
    }
    find(v, name).unwrap()
}
fn run(source: &str, d: &Document) -> syson_semantic_validation::Report {
    validate(source, d, &[NAMESPACE, PART], flag()).unwrap()
}
#[test]
fn actual_f14_document_passes_selected_checks_without_mutation() {
    let d = imported(SOURCE);
    let before = d.clone();
    let report = run(SOURCE, &d);
    assert_eq!(report.checks.len(), 5);
    assert!(report.checks.iter().all(|c| c.passed));
    assert!(report.diagnostics.is_empty());
    assert_eq!(d, before);
}
#[test]
fn missing_part_definition_yields_source_positioned_warning_and_actual_f46_label() {
    let source = "package '🚗' {\n  part def Wheel; part wheel:Wheel;\n}";
    let mut v = value(&imported(source));
    named(&mut v, "wheel")["data"]["ownedRelationship"] = json!([]);
    let report = run(source, &document(v));
    let diagnostic = &report.diagnostics[0];
    assert_eq!(diagnostic.rule, PART);
    assert_eq!(diagnostic.qualified_name.as_deref(), Some("'🚗'::wheel"));
    assert_eq!(diagnostic.severity, "WARNING");
    assert_eq!(&source[diagnostic.span.bytes.clone()], "part wheel:Wheel;");
    assert_eq!(diagnostic.line, 2);
    assert_eq!(diagnostic.column_utf16, 19);
    assert_eq!(
        diagnostic.span.utf16.start,
        source[..diagnostic.span.bytes.start].encode_utf16().count()
    );
}
#[test]
fn source_utf16_column_counts_supplementary_characters_on_same_line() {
    let source = "package '🚗' {part def Wheel;part wheel:Wheel;}";
    let mut v = value(&imported(source));
    named(&mut v, "wheel")["data"]["ownedRelationship"] = json!([]);
    let diagnostic = &run(source, &document(v)).diagnostics[0];
    assert_eq!(
        diagnostic.column_utf16,
        source[..diagnostic.span.bytes.start].encode_utf16().count() + 1
    );
    assert_ne!(diagnostic.column_utf16, diagnostic.span.bytes.start + 1);
}
#[test]
fn namespace_distinguishibility_uses_member_class_compatibility() {
    let source = "package P {part def T;part def T;}";
    let mut tx = Editor::new(Document::empty()).begin();
    tx.create_package(None, ids(1), None, "P").unwrap();
    tx.create_part_definition(&ids(1).element, ids(2), ids(3), "T")
        .unwrap();
    tx.create_part_definition(&ids(1).element, ids(4), ids(5), "T")
        .unwrap();
    let report = run(source, tx.document());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].rule, NAMESPACE);
    assert_eq!(report.diagnostics[0].qualified_name.as_deref(), Some("P"));
    let source = "package P {package T;part def T;}";
    let mut tx = Editor::new(Document::empty()).begin();
    tx.create_package(None, ids(1), None, "P").unwrap();
    tx.create_package(Some(&ids(1).element), ids(2), Some(ids(3)), "T")
        .unwrap();
    tx.create_part_definition(&ids(1).element, ids(4), ids(5), "T")
        .unwrap();
    assert!(run(source, tx.document()).diagnostics.is_empty());
}
#[test]
fn short_name_collisions_are_checked_with_both_member_names() {
    let source = "package P {part def A;part def B;}";
    let mut v = value(&imported(source));
    named(&mut v, "B")["data"]["declaredShortName"] = json!("A");
    assert_eq!(run(source, &document(v)).diagnostics[0].rule, NAMESPACE);
}
#[test]
fn unrelated_containment_scopes_do_not_conflict() {
    let source = "package P {package A {part def T;} package B {part def T;}}";
    assert!(run(source, &imported(source)).diagnostics.is_empty());
}
#[test]
fn null_and_wrong_class_types_fail_but_any_direct_part_definition_passes() {
    for target in [Value::Null, json!(ids(1).element)] {
        let mut v = value(&imported(SOURCE));
        named(&mut v, "wheel")["data"]["ownedRelationship"][0]["data"]["type"] = target;
        assert_eq!(run(SOURCE, &document(v)).diagnostics[0].rule, PART);
    }
    let mut v = value(&imported(SOURCE));
    let mut extra = named(&mut v, "wheel")["data"]["ownedRelationship"][0].clone();
    extra["id"] = json!(ids(100).resource);
    extra["data"]["elementId"] = json!(ids(100).element);
    extra["data"]["type"] = Value::Null;
    named(&mut v, "wheel")["data"]["ownedRelationship"]
        .as_array_mut()
        .unwrap()
        .insert(0, extra);
    assert!(run(SOURCE, &document(v)).diagnostics.is_empty());
}
#[test]
fn selected_semantic_identity_never_falls_back_to_json_node_id() {
    let mut v = value(&imported(SOURCE));
    let definition = named(&mut v, "Wheel")["id"].clone();
    named(&mut v, "wheel")["data"]["ownedRelationship"][0]["data"]["type"] = definition.clone();
    assert_eq!(
        validate(SOURCE, &document(v), &[PART], flag()),
        Err(Error::UnresolvedTyping(definition.as_str().unwrap().into()))
    );
}
#[test]
fn unresolved_external_and_nonstring_typing_never_produce_clean_verdict() {
    for target in [
        json!(
            "sysmllibrary:///00000000-0000-4000-8000-000000000001#00000000-0000-4000-8000-000000000002"
        ),
        json!(123),
    ] {
        let mut v = value(&imported(SOURCE));
        named(&mut v, "wheel")["data"]["ownedRelationship"][0]["data"]["type"] = target;
        assert!(validate(SOURCE, &document(v), &[PART], flag()).is_err());
    }
}
#[test]
fn explicit_rule_selection_deduplicates_and_unknown_rule_refuses_whole_request() {
    let d = imported(SOURCE);
    assert_eq!(
        validate(SOURCE, &d, &[PART, PART], flag())
            .unwrap()
            .checks
            .len(),
        1
    );
    assert!(matches!(
        validate(SOURCE, &d, &[], flag()),
        Err(Error::UnsupportedShape(_))
    ));
    assert_eq!(
        validate(SOURCE, &d, &[PART, "validateAll"], flag()),
        Err(Error::UnsupportedRule("validateAll".into()))
    );
    assert!(
        validate("package P;", &imported("package P;"), &[PART], flag())
            .unwrap()
            .checks
            .is_empty()
    );
}
#[test]
fn changed_name_class_order_parent_or_extra_declaration_refuses_source_association() {
    for source in [
        "package Changed {part def Wheel;part def Car {part wheel:Wheel;}}",
        "package Vehicle {part def Car {part wheel:Wheel;}part def Wheel;}",
        "package Vehicle {part def Wheel;part def Car;part wheel:Wheel;}",
        "package Vehicle {part def Wheel;}",
    ] {
        assert_eq!(
            validate(source, &imported(SOURCE), &[PART], flag()),
            Err(Error::SourceAssociation)
        );
    }
    let mut v = value(&imported(SOURCE));
    named(&mut v, "Wheel")["eClass"] = json!("sysml:Package");
    assert_eq!(
        validate(SOURCE, &document(v), &[PART], flag()),
        Err(Error::SourceAssociation)
    );
}
#[test]
fn unsupported_classifiers_and_containment_channels_are_explicit() {
    let mut v = value(&imported(SOURCE));
    named(&mut v, "Wheel")["eClass"] = json!("sysml:PortDefinition");
    assert!(matches!(
        validate(SOURCE, &document(v), &[PART], flag()),
        Err(Error::UnsupportedClassifier(_))
    ));
    let mut v = value(&imported(SOURCE));
    let duplicate = named(&mut v, "Wheel").clone();
    let mut duplicate = duplicate;
    duplicate["id"] = json!(ids(100).resource);
    duplicate["data"]["elementId"] = json!(ids(100).element);
    named(&mut v, "Vehicle")["data"]["ownedRelatedElement"] = json!([duplicate]);
    assert!(matches!(
        validate(SOURCE, &document(v), &[PART], flag()),
        Err(Error::UnsupportedShape(_))
    ));
}
#[test]
fn membership_must_own_one_declaration_and_typing_must_be_on_usage() {
    let mut v = value(&imported(SOURCE));
    let typing = named(&mut v, "wheel")["data"]["ownedRelationship"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    named(&mut v, "Wheel")["data"]["ownedRelationship"] = json!([typing]);
    assert_eq!(
        validate(SOURCE, &document(v), &[PART], flag()),
        Err(Error::UnsupportedShape("typing only on PartUsage"))
    );
    let mut v = value(&imported(SOURCE));
    named(&mut v, "Vehicle")["data"]["ownedRelationship"][0]["data"]["ownedRelatedElement"] =
        json!([]);
    assert!(matches!(
        validate(SOURCE, &document(v), &[PART], flag()),
        Err(Error::UnsupportedShape(_))
    ));
}
#[test]
fn unsupported_syntax_and_cancelled_runs_do_not_mutate_editor_or_history() {
    let mut e = Editor::new(imported(SOURCE));
    let before = e.document().clone();
    assert!(matches!(
        validate("action A;", e.document(), &[PART], flag()),
        Err(Error::Parse(_))
    ));
    assert_eq!(
        validate(
            SOURCE,
            e.document(),
            &[PART],
            Arc::new(AtomicBool::new(true))
        ),
        Err(Error::Cancelled)
    );
    assert_eq!(e.document(), &before);
    assert!(!e.undo());
}
#[test]
fn empty_input_has_explicit_zero_applicable_checks_and_quoted_names_associate() {
    let report = validate(" ", &Document::empty(), &[NAMESPACE, PART], flag()).unwrap();
    assert!(report.checks.is_empty());
    assert!(report.diagnostics.is_empty());
    let source = "package 'Road vehicle' {part def 'W\\'heel';part axle:'W\\'heel';}";
    assert!(run(source, &imported(source)).diagnostics.is_empty());
}

#[test]
fn unadmitted_fields_and_mismatched_or_absent_typed_feature_refuse() {
    for field in ["memberName", "declaredName", "memberElement"] {
        let mut v = value(&imported(SOURCE));
        named(&mut v, "Vehicle")["data"]["ownedRelationship"][0]["data"][field] = json!("override");
        assert!(matches!(
            validate(SOURCE, &document(v), &[NAMESPACE], flag()),
            Err(Error::UnsupportedShape(_))
        ));
    }
    for target in [
        Value::Null,
        json!(ids(1).element),
        json!(ids(1).resource),
        json!(123),
    ] {
        let mut v = value(&imported(SOURCE));
        named(&mut v, "wheel")["data"]["ownedRelationship"][0]["data"]["typedFeature"] = target;
        assert!(matches!(
            validate(SOURCE, &document(v), &[PART], flag()),
            Err(Error::UnsupportedShape(_))
        ));
    }
    let mut v = value(&imported(SOURCE));
    named(&mut v, "wheel")["data"]["ownedRelationship"][0]["data"]
        .as_object_mut()
        .unwrap()
        .remove("typedFeature");
    assert!(matches!(
        validate(SOURCE, &document(v), &[PART], flag()),
        Err(Error::UnsupportedShape(_))
    ));
}
