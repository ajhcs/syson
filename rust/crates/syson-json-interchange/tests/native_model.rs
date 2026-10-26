// SPDX-License-Identifier: EPL-2.0
use std::sync::{Arc, OnceLock, atomic::AtomicBool};

use serde_json::{Value, json};
use syson_json_interchange::{Bundle, MAX_RESOURCE_BYTES, native_model, selected_model};
use syson_model::{Document, Editor, NodeIds};
use syson_text_import::Importer;

const KEY: &str = "reverse-model";
const SOURCE: &str = "package Vehicle {part def Car {part wheel:Library::Wheel;}} package Library {part def Wheel;part def Spare;part spare:Spare;}";
fn ids(n: u32) -> NodeIds {
    NodeIds::new(
        &format!("10000000-0000-4000-8000-{n:012x}"),
        &format!("20000000-0000-4000-8000-{n:012x}"),
    )
    .unwrap()
}
fn importer() -> &'static Importer {
    static IMPORTER: OnceLock<Importer> = OnceLock::new();
    IMPORTER.get_or_init(|| Importer::pinned().unwrap())
}
fn selected() -> Document {
    importer()
        .stage(
            Editor::new(Document::empty()).begin(),
            SOURCE,
            None,
            (1..100).map(ids),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .document()
        .clone()
}
fn forward(document: &Document) -> Bundle {
    selected_model::load(
        KEY,
        document,
        selected_model::SourceEncoding::LocalSemanticElementIds,
    )
    .unwrap()
}
fn native(document: &Document) -> Document {
    forward(document).document(KEY).unwrap().clone()
}
fn reverse(document: &Document) -> Result<Document, native_model::Error> {
    native_model::load(
        KEY,
        document,
        native_model::SourceEncoding::LocalJsonNodeIds,
    )
}
fn named(document: &Document, name: &str) -> String {
    document
        .elements()
        .iter()
        .find(|n| n.name.as_deref() == Some(name))
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
fn typing<'a>(value: &'a mut Value, usage: &str) -> &'a mut Value {
    &mut node_mut(value, usage)["data"]["ownedRelationship"][0]
}
fn edit(document: &Document, change: impl FnOnce(&mut Value)) -> Document {
    let mut value = document.value().clone();
    change(&mut value);
    Document::from_json(&value.to_string()).unwrap()
}
fn refuses(document: &Document) {
    let before = document.clone();
    assert!(reverse(document).is_err());
    assert_eq!(document, &before);
}
#[test]
fn six_class_cross_package_native_to_selected_round_trip_preserves_every_value() {
    let original = selected();
    let classes: std::collections::BTreeSet<_> =
        original.elements().into_iter().map(|n| n.class).collect();
    assert_eq!(classes.len(), 6);
    let native = native(&original);
    let before = native.clone();
    let restored = reverse(&native).unwrap();
    assert_eq!(restored, original);
    assert_eq!(native, before);
    assert_eq!(native.elements(), restored.elements());
}
#[test]
fn native_open_then_actual_editor_and_f14_import_then_save_reload_keeps_identity() {
    let original = selected();
    let original_native = native(&original);
    let mut editor = Editor::new(reverse(&original_native).unwrap());
    let wheel = named(editor.document(), "wheel");
    let mut transaction = editor.begin();
    transaction.rename(&wheel, "frontWheel").unwrap();
    assert!(editor.commit(transaction).unwrap());
    let staged = importer()
        .stage(
            editor.begin(),
            "package Added {part def Axle;part axle:Axle;}",
            None,
            (1000..1100).map(ids),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(staged.commit(&mut editor).unwrap());
    let expected = editor.document().clone();
    let saved = forward(&expected).save().unwrap();
    let reloaded = Bundle::load(saved).unwrap();
    let reopened = reverse(reloaded.document(KEY).unwrap()).unwrap();
    assert_eq!(reopened, expected);
    assert_eq!(named(&reopened, "frontWheel"), wheel);
    for row in original.elements() {
        let reopened_node = reopened.element(&row.element_id).unwrap();
        assert_eq!(
            reopened_node["id"],
            original.element(&row.element_id).unwrap()["id"]
        );
        assert_eq!(reopened_node["data"]["elementId"], row.element_id);
    }
    for (usage, target) in [
        ("frontWheel", "Wheel"),
        ("spare", "Spare"),
        ("axle", "Axle"),
    ] {
        let usage = named(&reopened, usage);
        let data = &reopened.element(&usage).unwrap()["data"]["ownedRelationship"][0]["data"];
        assert_eq!(data["type"], named(&reopened, target));
        assert_eq!(data["typedFeature"], usage);
    }
    assert_eq!(original_native, native(&original));
}
#[test]
fn same_class_cross_domain_collisions_lookup_only_native_node_ids() {
    let selected = selected();
    let (wheel, spare, usage, other) = (
        named(&selected, "Wheel"),
        named(&selected, "Spare"),
        named(&selected, "wheel"),
        named(&selected, "spare"),
    );
    let changed = edit(&selected, |value| {
        for (semantic, node) in [
            (&wheel, &spare),
            (&spare, &wheel),
            (&usage, &other),
            (&other, &usage),
        ] {
            node_mut(value, semantic)["id"] = json!(node);
        }
    });
    let native = native(&changed);
    let result = reverse(&native).unwrap();
    assert_eq!(result, changed);
    let typing = &result.element(&usage).unwrap()["data"]["ownedRelationship"][0]["data"];
    assert_eq!(typing["type"], wheel);
    assert_eq!(typing["typedFeature"], usage);
}
#[test]
fn opaque_values_fake_nodes_and_large_numbers_are_not_rewritten() {
    let original = selected();
    let usage = named(&original, "wheel");
    let changed = edit(&original, |value| {
        value["vendor"] = json!({"large":900719925474099312345u128,"values":[true,null]});
        value["ns"]["vendor"] = json!("https://example.invalid/vendor");
        node_mut(value, &usage)["annotation"] = json!({"type":"unchanged"});
        node_mut(value, &usage)["data"]["vendorObject"] = json!({
            "eClass":"sysml:FeatureTyping", "data":{"type":ids(999).resource,"typedFeature":ids(998).resource}
        });
    });
    let native = native(&changed);
    assert_eq!(reverse(&native).unwrap(), changed);
}
#[test]
fn selected_semantic_wires_are_not_autodetected_or_used_as_fallback() {
    refuses(&selected());
}
#[test]
fn missing_null_nonstring_dangling_external_and_typed_local_wires_refuse() {
    let selected = selected();
    let native = native(&selected);
    let usage = named(&native, "wheel");
    let target = native.element(&named(&native, "Wheel")).unwrap()["id"]
        .as_str()
        .unwrap();
    for feature in ["type", "typedFeature"] {
        for wire in [
            Value::Null,
            json!(123),
            json!("missing"),
            json!(ids(999).resource),
            json!(format!("sysml:PartDefinition {target}")),
            json!(format!(
                "sysml:PartDefinition sysmllibrary:///{target}#{target}"
            )),
            json!(named(&selected, "Wheel")),
        ] {
            refuses(&edit(&native, |value| {
                typing(value, &usage)["data"][feature] = wire
            }));
        }
        refuses(&edit(&native, |value| {
            typing(value, &usage)["data"]
                .as_object_mut()
                .unwrap()
                .remove(feature);
        }));
    }
}
#[test]
fn wrong_target_class_or_other_same_class_containing_usage_refuse() {
    let native = native(&selected());
    let usage = named(&native, "wheel");
    let package_node = native.element(&named(&native, "Vehicle")).unwrap()["id"].clone();
    refuses(&edit(&native, |value| {
        typing(value, &usage)["data"]["type"] = package_node
    }));
    let other_usage = native.element(&named(&native, "spare")).unwrap()["id"].clone();
    let changed = edit(&native, |value| {
        typing(value, &usage)["data"]["typedFeature"] = other_usage
    });
    // Native reference classes alone cannot establish the actual owner association.
    assert!(matches!(
        reverse(&changed),
        Err(native_model::Error::Selected(_))
    ));
    refuses(&changed);
}
#[test]
fn unsupported_classes_known_references_and_owning_shapes_refuse() {
    let native = native(&selected());
    let usage = named(&native, "wheel");
    let definition = named(&native, "Wheel");
    let package = named(&native, "Vehicle");
    for class in ["sysml:PortDefinition", "sysml:ActionDefinition"] {
        let changed = edit(&native, |value| {
            node_mut(value, &definition)["eClass"] = json!(class)
        });
        assert!(matches!(
            reverse(&changed),
            Err(native_model::Error::UnsupportedClass(_))
        ));
    }
    refuses(&edit(&native, |value| {
        node_mut(value, &package)["data"]["owner"] =
            native.element(&definition).unwrap()["id"].clone();
    }));
    refuses(&edit(&native, |value| {
        node_mut(value, &package)["data"]["ownedRelationship"][0]["eClass"] =
            json!("sysml:FeatureMembership");
    }));
    refuses(&edit(&native, |value| {
        node_mut(value, &usage)["data"]["ownedRelationship"] = json!([]);
    }));
    refuses(&edit(&native, |value| {
        let mut extra = typing(value, &usage).clone();
        extra["id"] = json!(ids(999).resource);
        extra["data"]["elementId"] = json!(ids(999).element);
        node_mut(value, &usage)["data"]["ownedRelationship"]
            .as_array_mut()
            .unwrap()
            .push(extra);
    }));
}
#[test]
fn late_failure_does_not_mutate_input_or_replace_published_selected_snapshot() {
    let native = native(&selected());
    let published = reverse(&native).unwrap();
    let before = published.clone();
    let changed = edit(&native, |value| {
        typing(value, &named(&native, "spare"))["data"]["typedFeature"] =
            native.element(&named(&native, "wheel")).unwrap()["id"].clone();
    });
    refuses(&changed);
    assert_eq!(published, before);
}
#[test]
fn empty_resources_invalid_keys_and_oversized_values_have_explicit_boundaries() {
    assert_eq!(reverse(&Document::empty()).unwrap(), Document::empty());
    let native = native(&selected());
    assert!(matches!(
        native_model::load("", &native, native_model::SourceEncoding::LocalJsonNodeIds),
        Err(native_model::Error::Native(error)) if matches!(*error, syson_json_interchange::Error::Identity)
    ));
    let oversized = edit(&native, |value| {
        value["vendor"] = json!("x".repeat(MAX_RESOURCE_BYTES))
    });
    assert!(matches!(
        reverse(&oversized),
        Err(native_model::Error::Limit)
    ));
}
