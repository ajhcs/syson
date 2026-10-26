// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::sync::{Arc, atomic::AtomicBool};
use syson_model::{Document, Editor, NodeIds};
use syson_text_export::{Error, Newline, Options, export};
use syson_text_import::Importer;
fn ids(n: u32) -> NodeIds {
    NodeIds::new(
        &format!("30000000-0000-4000-8000-{n:012x}"),
        &format!("40000000-0000-4000-8000-{n:012x}"),
    )
    .unwrap()
}
fn import(source: &str) -> Document {
    let mut editor = Editor::new(Document::empty());
    Importer::pinned()
        .unwrap()
        .stage(
            editor.begin(),
            source,
            None,
            (1..1025).map(ids),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .commit(&mut editor)
        .unwrap();
    editor.document().clone()
}
fn named(document: &Document, name: &str) -> String {
    document
        .elements()
        .iter()
        .find(|r| r.name.as_deref() == Some(name))
        .unwrap()
        .element_id
        .clone()
}
fn data<'a>(value: &'a mut Value, id: &str) -> Option<&'a mut Value> {
    if value["data"]["elementId"] == id {
        return value.get_mut("data");
    }
    match value {
        Value::Array(rows) => rows.iter_mut().find_map(|v| data(v, id)),
        Value::Object(rows) => rows.values_mut().find_map(|v| data(v, id)),
        _ => None,
    }
}
fn change(document: &Document, id: &str, edit: impl FnOnce(&mut Value)) -> Document {
    let mut value = document.value().clone();
    edit(data(&mut value, id).unwrap());
    Document::from_json(&value.to_string()).unwrap()
}
const SOURCE: &str = "package Vehicle { part def Wheel; part wheel:Wheel; }";
#[test]
fn exact_package_part_typing_bytes_roundtrip_through_actual_f14() {
    let document = import(SOURCE);
    let before = document.clone();
    let output = export(&document, Options::default()).unwrap();
    assert_eq!(
        output,
        "package Vehicle {\n    part def Wheel;\n    part wheel : Wheel;\n}"
    );
    assert_eq!(document, before);
    let reloaded = import(&output);
    assert_eq!(export(&reloaded, Options::default()).unwrap(), output);
    let usage = named(&reloaded, "wheel");
    let definition = named(&reloaded, "Wheel");
    assert_eq!(
        reloaded.element(&usage).unwrap()["data"]["ownedRelationship"][0]["data"]["type"],
        definition
    );
}
#[test]
fn explicit_crlf_and_tabs_have_no_platform_or_trailing_newline_dependency() {
    let document = import("package P { package Q {part def D;} }");
    assert_eq!(
        export(
            &document,
            Options {
                newline: Newline::CrLf,
                indentation: "\t"
            }
        )
        .unwrap(),
        "package P {\r\n\tpackage Q {\r\n\t\tpart def D;\r\n\t}\r\n}"
    );
    assert!(matches!(
        export(
            &document,
            Options {
                newline: Newline::Lf,
                indentation: "\n"
            }
        ),
        Err(Error::Unsupported("indentation"))
    ));
}
#[test]
fn empty_and_multiple_root_packages_keep_order() {
    assert_eq!(export(&Document::empty(), Options::default()).unwrap(), "");
    assert_eq!(
        export(&import("package B;package A;"), Options::default()).unwrap(),
        "package B;\npackage A;"
    );
}
#[test]
fn printable_spaces_unicode_and_native_keywords_roundtrip() {
    let document = import("package 'Road vehicle' {part def '车轮';part 'part':'车轮';}");
    let output = export(&document, Options::default()).unwrap();
    assert_eq!(
        output,
        "package 'Road vehicle' {\n    part def '车轮';\n    part 'part' : '车轮';\n}"
    );
    assert_eq!(
        export(&import(&output), Options::default()).unwrap(),
        output
    );
}
#[test]
fn nested_feature_memberships_and_forward_references_keep_order() {
    let document = import("package P {part a:T;part def T {part child:P::T;} }");
    let output = export(&document, Options::default()).unwrap();
    assert_eq!(
        output,
        "package P {\n    part a : T;\n    part def T {\n        part child : T;\n    }\n}"
    );
    assert_eq!(
        export(&import(&output), Options::default()).unwrap(),
        output
    );
}
#[test]
fn cross_package_deresolution_qualifies_only_when_required() {
    let document =
        import("package V {part def Wheel;part a:Lib::Wheel;} package Lib {part def Wheel;}");
    let output = export(&document, Options::default()).unwrap();
    assert!(output.contains("part a : Lib::Wheel;"));
    assert_eq!(
        export(&import(&output), Options::default()).unwrap(),
        output
    );
}
#[test]
fn shadowed_root_path_is_refused_instead_of_binding_a_different_type() {
    let document =
        import("package Lib {part def T;} package V {package Lib;part a:V::Local;part def Local;}");
    let usage = named(&document, "a");
    let target = named(&document, "T");
    let document = change(&document, &usage, |d| {
        d["ownedRelationship"][0]["data"]["type"] = json!(target)
    });
    assert_eq!(
        export(&document, Options::default()),
        Err(Error::UnresolvableType)
    );
}
#[test]
fn semantic_typing_never_falls_back_to_local_node_identity() {
    let document = import(SOURCE);
    let usage = named(&document, "wheel");
    let wire = document.element(&named(&document, "Wheel")).unwrap()["id"].clone();
    let wrong = change(&document, &usage, |d| {
        d["ownedRelationship"][0]["data"]["type"] = wire
    });
    assert_eq!(
        export(&wrong, Options::default()),
        Err(Error::InvalidTyping)
    );
    let external = change(&document, &usage, |d| {
        d["ownedRelationship"][0]["data"]["type"] =
            json!("sysml:PartDefinition sysmllibrary:///x#y")
    });
    assert_eq!(
        export(&external, Options::default()),
        Err(Error::InvalidTyping)
    );
}
#[test]
fn dangling_wrong_class_wrong_typed_feature_and_multiple_types_refuse() {
    let document = import(SOURCE);
    let usage = named(&document, "wheel");
    for target in ["missing".to_owned(), named(&document, "Vehicle")] {
        let wrong = change(&document, &usage, |d| {
            d["ownedRelationship"][0]["data"]["type"] = json!(target)
        });
        assert_eq!(
            export(&wrong, Options::default()),
            Err(Error::InvalidTyping)
        );
    }
    let wrong = change(&document, &usage, |d| {
        d["ownedRelationship"][0]["data"]["typedFeature"] = json!(named(&document, "Wheel"))
    });
    assert_eq!(
        export(&wrong, Options::default()),
        Err(Error::InvalidTyping)
    );
    let mut editor = Editor::new(document.clone());
    let mut tx = editor.begin();
    tx.type_part_usage(&usage, &named(&document, "Wheel"), ids(99))
        .unwrap();
    editor.commit(tx).unwrap();
    assert_eq!(
        export(editor.document(), Options::default()),
        Err(Error::InvalidTyping)
    );
}
#[test]
fn unknown_fields_modifiers_and_untyped_parts_are_not_silently_dropped() {
    let document = import(SOURCE);
    for field in [
        "documentation",
        "declaredShortName",
        "isAbstract",
        "futureField",
    ] {
        let changed = change(&document, &named(&document, "Wheel"), |d| {
            d[field] = json!(false)
        });
        assert!(matches!(
            export(&changed, Options::default()),
            Err(Error::Unsupported(_))
        ));
    }
    let changed = change(&document, &named(&document, "wheel"), |d| {
        d["isComposite"] = json!(false)
    });
    assert!(matches!(
        export(&changed, Options::default()),
        Err(Error::Unsupported(_))
    ));
    let changed = change(&document, &named(&document, "wheel"), |d| {
        d["ownedRelationship"] = json!([])
    });
    assert!(matches!(
        export(&changed, Options::default()),
        Err(Error::Unsupported(_))
    ));
    let mut value = document.value().clone();
    value["extra"] = json!(1);
    assert!(matches!(
        export(
            &Document::from_json(&value.to_string()).unwrap(),
            Options::default()
        ),
        Err(Error::Unsupported(_))
    ));
}
#[test]
fn unsupported_classifier_and_wrong_membership_kind_fail_without_output() {
    let document = import(SOURCE);
    let mut value = document.value().clone();
    value["content"][0]["data"]["ownedRelationship"][0]["data"]["ownedRelatedElement"][0]["eClass"] =
        json!("sysml:PortDefinition");
    assert!(matches!(
        export(
            &Document::from_json(&value.to_string()).unwrap(),
            Options::default()
        ),
        Err(Error::Unsupported("declaration classifier"))
    ));
    let mut value = document.value().clone();
    value["content"][0]["data"]["ownedRelationship"][0]["eClass"] =
        json!("sysml:FeatureMembership");
    assert!(matches!(
        export(
            &Document::from_json(&value.to_string()).unwrap(),
            Options::default()
        ),
        Err(Error::Unsupported("membership kind"))
    ));
    assert_eq!(document, import(SOURCE));
}
#[test]
fn duplicate_siblings_and_lossy_name_inputs_refuse() {
    let document = import("package P {part def A;part def B;}");
    let changed = change(&document, &named(&document, "B"), |d| {
        d["declaredName"] = json!("A")
    });
    assert_eq!(
        export(&changed, Options::default()),
        Err(Error::DuplicateName)
    );
    for name in ["a'b", "a\\b", "a\nb", ""] {
        let changed = change(&document, &named(&document, "A"), |d| {
            d["declaredName"] = json!(name)
        });
        assert_eq!(
            export(&changed, Options::default()),
            Err(Error::NonRoundTripName)
        );
    }
}
#[test]
fn actual_f14_gate_refuses_wider_reserved_names_and_parser_size_limits() {
    let document = import("package P;");
    for name in ["true", "classifier", "xor"] {
        let changed = change(&document, &named(&document, "P"), |d| {
            d["declaredName"] = json!(name)
        });
        assert!(
            matches!(
                export(&changed, Options::default()),
                Err(Error::NotImportable(_))
            ),
            "{name}"
        );
    }
    let changed = change(&document, &named(&document, "P"), |d| {
        d["declaredName"] = json!("x".repeat(65_537))
    });
    assert_eq!(export(&changed, Options::default()), Err(Error::Limit));
}
