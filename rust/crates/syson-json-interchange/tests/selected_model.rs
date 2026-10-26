// SPDX-License-Identifier: EPL-2.0
use std::sync::{Arc, OnceLock, atomic::AtomicBool};

use serde_json::{Value, json};
use syson_json_interchange::{Bundle, Input, MAX_RESOURCE_BYTES, selected_model};
use syson_metamodel::ClassifierId as C;
use syson_model::{Document, Editor, NodeIds};
use syson_references::FragmentIdentity;
use syson_text_import::Importer;

const KEY: &str = "selected-model";
const SOURCE: &str = "package P {part def T;part def U;part a:T;part b:U;}";
fn ids(n: u32) -> NodeIds {
    NodeIds::new(
        &format!("10000000-0000-4000-8000-{n:012x}"),
        &format!("20000000-0000-4000-8000-{n:012x}"),
    )
    .unwrap()
}
fn imported() -> Document {
    import_source(SOURCE)
}
fn import_source(source: &str) -> Document {
    static IMPORTER: OnceLock<Importer> = OnceLock::new();
    IMPORTER
        .get_or_init(|| Importer::pinned().unwrap())
        .stage(
            Editor::new(Document::empty()).begin(),
            source,
            None,
            (1..100).map(ids),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .document()
        .clone()
}
fn named(document: &Document, name: &str) -> String {
    document
        .elements()
        .iter()
        .find(|row| row.name.as_deref() == Some(name))
        .unwrap()
        .element_id
        .clone()
}
fn node_mut<'a>(value: &'a mut Value, semantic: &str) -> &'a mut Value {
    fn find<'a>(value: &'a mut Value, semantic: &str) -> Option<&'a mut Value> {
        if value["data"]["elementId"] == semantic {
            return Some(value);
        }
        match value {
            Value::Array(rows) => rows.iter_mut().find_map(|v| find(v, semantic)),
            Value::Object(rows) => rows.values_mut().find_map(|v| find(v, semantic)),
            _ => None,
        }
    }
    find(value, semantic).unwrap()
}
fn edit(document: &Document, change: impl FnOnce(&mut Value)) -> Document {
    let mut value = document.value().clone();
    change(&mut value);
    Document::from_json(&value.to_string()).unwrap()
}
fn typing<'a>(value: &'a mut Value, usage: &str) -> &'a mut Value {
    &mut node_mut(value, usage)["data"]["ownedRelationship"][0]
}
fn bridge(document: &Document) -> Result<Bundle, selected_model::Error> {
    selected_model::load(
        KEY,
        document,
        selected_model::SourceEncoding::LocalSemanticElementIds,
    )
}
fn native(document: &Document) -> Result<Bundle, syson_json_interchange::Error> {
    Bundle::load(vec![Input {
        key: KEY.into(),
        json: document.to_json().unwrap(),
        fragments: FragmentIdentity::JsonNodeId,
    }])
}
fn resolved(bundle: &Bundle, usage: &str, feature: &str, expected: C) -> String {
    let wire = &bundle.document(KEY).unwrap().element(usage).unwrap()["data"]["ownedRelationship"]
        [0]["data"][feature];
    bundle
        .resolve(KEY, wire, expected)
        .unwrap()
        .unwrap()
        .element_id()
        .into()
}
fn refuses(document: &Document) {
    let before = document.clone();
    assert!(bridge(document).is_err());
    assert_eq!(document, &before);
}
#[test]
fn actual_f14_import_bridges_then_saves_and_reloads_through_normal_f33() {
    let document = imported();
    let original = document.clone();
    assert!(native(&document).is_err());
    let bundle = bridge(&document).unwrap();
    let reloaded = Bundle::load(bundle.save().unwrap()).unwrap();
    for (usage, target) in [("a", "T"), ("b", "U")] {
        let usage = named(&document, usage);
        assert_eq!(
            resolved(&reloaded, &usage, "type", C::PartDefinition),
            named(&document, target)
        );
        assert_eq!(
            resolved(&reloaded, &usage, "typedFeature", C::PartUsage),
            usage
        );
    }
    assert_eq!(
        reloaded.document(KEY).unwrap().elements(),
        document.elements()
    );
    assert_eq!(document, original);
}
#[test]
fn cross_domain_collisions_never_select_the_other_same_class_target() {
    let original = imported();
    let (t, u, a, b) = (
        named(&original, "T"),
        named(&original, "U"),
        named(&original, "a"),
        named(&original, "b"),
    );
    let document = edit(&original, |value| {
        node_mut(value, &t)["id"] = json!(u);
        node_mut(value, &u)["id"] = json!(t);
        node_mut(value, &a)["id"] = json!(b);
        node_mut(value, &b)["id"] = json!(a);
    });
    // Unchanged native loading assumes native node IDs. It cannot infer that a
    // caller supplied selected-model semantic wires and accepts the wrong IDs.
    let incorrectly_tagged = native(&document).unwrap();
    assert_eq!(
        resolved(&incorrectly_tagged, &a, "type", C::PartDefinition),
        u
    );
    assert_eq!(
        resolved(&incorrectly_tagged, &a, "typedFeature", C::PartUsage),
        b
    );
    let converted = bridge(&document).unwrap();
    assert_eq!(resolved(&converted, &a, "type", C::PartDefinition), t);
    assert_eq!(resolved(&converted, &a, "typedFeature", C::PartUsage), a);
    assert_eq!(resolved(&converted, &b, "type", C::PartDefinition), u);
    assert_eq!(resolved(&converted, &b, "typedFeature", C::PartUsage), b);
    let reload = Bundle::load(converted.save().unwrap()).unwrap();
    assert_eq!(resolved(&reload, &a, "type", C::PartDefinition), t);
}
#[test]
fn unknown_envelope_node_and_data_values_and_all_identities_survive() {
    let original = imported();
    let package = named(&original, "P");
    let usage = named(&original, "a");
    let document = edit(&original, |value| {
        value["vendor"] = json!({"integer":900719925474099312345u128,"items":[true,null]});
        value["ns"]["vendor"] = json!("https://vendor.invalid/opaque");
        node_mut(value, &package)["annotation"] = json!({"text":"unchanged"});
        node_mut(value, &usage)["data"]["vendorReference"] = json!(named(&original, "U"));
    });
    let converted = bridge(&document).unwrap();
    let result = converted.document(KEY).unwrap();
    assert_eq!(result.value()["vendor"], document.value()["vendor"]);
    assert_eq!(result.value()["ns"], document.value()["ns"]);
    assert_eq!(
        result.element(&package).unwrap()["annotation"],
        document.element(&package).unwrap()["annotation"]
    );
    assert_eq!(
        result.element(&usage).unwrap()["data"]["vendorReference"],
        document.element(&usage).unwrap()["data"]["vendorReference"]
    );
    assert_eq!(result.elements(), document.elements());
    let before = document.value().clone();
    let mut normalized = result.value().clone();
    for usage in [named(&document, "a"), named(&document, "b")] {
        typing(&mut normalized, &usage)["data"] =
            document.element(&usage).unwrap()["data"]["ownedRelationship"][0]["data"].clone();
    }
    assert_eq!(normalized, before);
}
#[test]
fn native_wire_documents_require_native_loading_and_are_not_auto_detected() {
    let converted = bridge(&imported()).unwrap();
    let document = converted.document(KEY).unwrap();
    assert!(native(document).is_ok());
    assert!(matches!(
        bridge(document),
        Err(selected_model::Error::Reference {
            feature: "typedFeature",
            ..
        })
    ));
}
#[test]
fn missing_null_nonstring_external_and_node_id_types_refuse_atomically() {
    let original = imported();
    let usage = named(&original, "a");
    let definition_node = original.element(&named(&original, "T")).unwrap()["id"].clone();
    for value in [
        Value::Null,
        json!(17),
        json!("missing"),
        json!(
            "sysml:PartDefinition sysmllibrary:///aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa#bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb"
        ),
        definition_node,
    ] {
        let changed = edit(&original, |doc| typing(doc, &usage)["data"]["type"] = value);
        refuses(&changed);
    }
    let changed = edit(&original, |doc| {
        typing(doc, &usage)["data"]
            .as_object_mut()
            .unwrap()
            .remove("type");
    });
    refuses(&changed);
}
#[test]
fn typed_feature_must_identify_its_actual_containing_usage_by_semantic_id() {
    let original = imported();
    let usage = named(&original, "a");
    for value in [
        Value::Null,
        json!(17),
        json!(named(&original, "b")),
        original.element(&usage).unwrap()["id"].clone(),
    ] {
        refuses(&edit(&original, |doc| {
            typing(doc, &usage)["data"]["typedFeature"] = value
        }));
    }
    refuses(&edit(&original, |doc| {
        typing(doc, &usage)["data"]
            .as_object_mut()
            .unwrap()
            .remove("typedFeature");
    }));
}
#[test]
fn wrong_class_targets_and_missing_or_multiple_typing_refuse() {
    let original = imported();
    let usage = named(&original, "a");
    refuses(&edit(&original, |doc| {
        typing(doc, &usage)["data"]["type"] = json!(named(&original, "P"))
    }));
    refuses(&edit(&original, |doc| {
        node_mut(doc, &usage)["data"]["ownedRelationship"] = json!([])
    }));
    refuses(&edit(&original, |doc| {
        let mut extra = typing(doc, &usage).clone();
        extra["id"] = json!(ids(999).resource);
        extra["data"]["elementId"] = json!(ids(999).element);
        node_mut(doc, &usage)["data"]["ownedRelationship"]
            .as_array_mut()
            .unwrap()
            .push(extra);
    }));
}
#[test]
fn unsupported_classes_membership_shapes_and_misplaced_typing_refuse() {
    let original = imported();
    let package = named(&original, "P");
    let usage = named(&original, "a");
    assert!(matches!(
        bridge(&edit(&original, |doc| {
            node_mut(doc, &named(&original, "T"))["eClass"] = json!("sysml:PortDefinition")
        })),
        Err(selected_model::Error::UnsupportedClass(_))
    ));
    refuses(&edit(&original, |doc| {
        node_mut(doc, &package)["data"]["ownedRelationship"][0]["eClass"] =
            json!("sysml:FeatureMembership");
    }));
    refuses(&edit(&original, |doc| {
        let moved =
            node_mut(doc, &package)["data"]["ownedRelationship"][1]["data"]["ownedRelatedElement"]
                .as_array_mut()
                .unwrap()
                .remove(0);
        node_mut(doc, &package)["data"]["ownedRelationship"][0]["data"]["ownedRelatedElement"]
            .as_array_mut()
            .unwrap()
            .push(moved);
    }));
    refuses(&edit(&original, |doc| {
        let moved = node_mut(doc, &usage)["data"]["ownedRelationship"]
            .as_array_mut()
            .unwrap()
            .remove(0);
        node_mut(doc, &package)["data"]["ownedRelationship"]
            .as_array_mut()
            .unwrap()
            .insert(0, moved);
    }));
}
#[test]
fn final_native_validation_refuses_other_known_reference_features() {
    let original = imported();
    let document = edit(&original, |value| {
        node_mut(value, &named(&original, "P"))["data"]["owner"] = json!(named(&original, "T"));
    });
    let before = document.clone();
    assert!(matches!(
        bridge(&document),
        Err(selected_model::Error::Native(
            syson_json_interchange::Error::UnsupportedReference { .. }
        ))
    ));
    assert_eq!(document, before);
}
#[test]
fn late_failure_drops_the_private_conversion_without_replacing_published_bundle() {
    let original = imported();
    let published = bridge(&original).unwrap();
    let before = published.save().unwrap()[0].json.clone();
    let changed = edit(&original, |doc| {
        typing(doc, &named(&original, "b"))["data"]["type"] = json!("missing");
    });
    refuses(&changed);
    assert_eq!(published.save().unwrap()[0].json, before);
}
#[test]
fn empty_resources_work_and_invalid_keys_or_oversized_values_refuse() {
    assert_eq!(bridge(&Document::empty()).unwrap().save().unwrap().len(), 1);
    assert!(matches!(
        selected_model::load(
            "",
            &imported(),
            selected_model::SourceEncoding::LocalSemanticElementIds
        ),
        Err(selected_model::Error::Native(
            syson_json_interchange::Error::Identity
        ))
    ));
    let oversized = edit(&imported(), |value| {
        value["vendor"] = json!("x".repeat(MAX_RESOURCE_BYTES));
    });
    assert!(matches!(
        bridge(&oversized),
        Err(selected_model::Error::Limit)
    ));
}

#[test]
fn nested_feature_membership_keeps_cross_package_targets_after_native_reload() {
    let source = "package Vehicle {part def Car {part wheel:Library::Wheel;}} package Library {part def Wheel;}";
    let document = import_source(source);
    let before = document.clone();
    let owner = named(&document, "Car");
    let usage = named(&document, "wheel");
    let target = named(&document, "Wheel");
    let membership = &document.element(&owner).unwrap()["data"]["ownedRelationship"][0];
    assert_eq!(membership["eClass"], "sysml:FeatureMembership");
    let converted = bridge(&document).unwrap();
    let reloaded = Bundle::load(converted.save().unwrap()).unwrap();
    assert_eq!(
        resolved(&reloaded, &usage, "type", C::PartDefinition),
        target
    );
    assert_eq!(
        resolved(&reloaded, &usage, "typedFeature", C::PartUsage),
        usage
    );
    assert_eq!(
        reloaded.document(KEY).unwrap().element(&owner).unwrap()["data"]["ownedRelationship"][0]["id"],
        membership["id"]
    );
    assert_eq!(
        reloaded.document(KEY).unwrap().elements(),
        document.elements()
    );
    assert_eq!(document, before);
}
