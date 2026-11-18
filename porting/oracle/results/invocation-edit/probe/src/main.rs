// SPDX-License-Identifier: EPL-2.0
#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{env, fs, io, path::PathBuf};
use syson_metamodel::ClassifierId;
use syson_model::{Document, Editor};
use syson_references::{FragmentIdentity, Registry, Resource};
use syson_workflow::{SnapshotStore, StoreError};

const PRIOR_RUST: &str = include_str!(
    "../../../invocation-rust/observation.json"
);
const EDITED: &str = "RenamedA";
const CANCELLED: &str = "CancelledEdit";
const PART_CASES: [(&str, bool); 2] = [("part-call", false), ("abstract-part-call", true)];
const CALC_CASE: &str = "typed-calculation-call";

fn one<'a>(rows: impl Iterator<Item = &'a Value>, label: &str) -> &'a Value {
    let mut rows = rows;
    let selected = rows.next().expect(label);
    assert!(rows.next().is_none(), "ambiguous {label}");
    selected
}

fn owned<'a>(parent: &'a Value, feature: &str, class: &str) -> &'a Value {
    one(
        parent["data"][feature]
            .as_array()
            .unwrap()
            .iter()
            .filter(|node| node["eClass"] == class),
        class,
    )
}

fn package(document: &Value) -> &Value {
    let [root] = document["content"].as_array().unwrap().as_slice() else {
        panic!("expected one captured namespace");
    };
    assert_eq!(root["eClass"], "sysml:Namespace");
    let membership = owned(root, "ownedRelationship", "sysml:OwningMembership");
    let package = owned(membership, "ownedRelatedElement", "sysml:Package");
    assert_eq!(package["data"]["declaredName"], "P");
    package
}

fn member<'a>(package: &'a Value, class: &str, name: &str) -> &'a Value {
    one(
        package["data"]["ownedRelationship"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|relation| relation["data"]["ownedRelatedElement"].as_array())
            .flatten()
            .filter(|node| node["eClass"] == class && node["data"]["declaredName"] == name),
        name,
    )
}

fn inspect_link(
    index: &Registry,
    resource: &str,
    feature: &Value,
    feature_class: ClassifierId,
    definition: &Value,
    definition_class: ClassifierId,
) -> Value {
    let link = owned(feature, "ownedRelationship", "sysml:FeatureTyping");
    let resolve = |field, node: &Value, class| {
        let target = index.resolve(resource, Some(&link["data"][field]), class)
            .unwrap()
            .unwrap();
        assert_eq!(target.resource(), resource);
        assert_eq!(target.node_id(), node["id"].as_str().unwrap());
        assert_eq!(target.element_id(), node["data"]["elementId"].as_str().unwrap());
        assert_ne!(target.node_id(), target.element_id());
        assert_eq!(index.target(&target).unwrap(), node);
        json!({"resource_id": target.node_id(), "element_id": target.element_id()})
    };
    json!({"relationship": {"resource_id": link["id"], "element_id": link["data"]["elementId"]},
           "type": resolve("type", definition, definition_class),
           "typed_feature": resolve("typedFeature", feature, feature_class)})
}

fn inspect_part(document: &Document, key: &str, definition_name: &str, is_abstract: bool) -> Value {
    let package = package(document.value());
    let target = member(package, "sysml:PartDefinition", definition_name);
    assert_eq!(target["data"]["isAbstract"].as_bool().unwrap_or(false), is_abstract);
    let usage = member(package, "sysml:PartUsage", "x");
    let feature = owned(usage, "ownedRelationship", "sysml:FeatureValue");
    let invocation = owned(feature, "ownedRelatedElement", "sysml:InvocationExpression");
    let index = registry(document, key);
    json!({
        "abstract": is_abstract,
        "usage_typing": inspect_link(&index, key, usage, ClassifierId::PartUsage, target, ClassifierId::PartDefinition),
        "invocation_typing": inspect_link(&index, key, invocation, ClassifierId::InvocationExpression, target, ClassifierId::PartDefinition)
    })
}

fn registry(document: &Document, key: &str) -> Registry {
    Registry::new(vec![Resource {
        key: key.into(),
        document: document.clone(),
        fragments: FragmentIdentity::JsonNodeId,
    }]).unwrap()
}

fn rename_expected(document: &Document, semantic_id: &str) -> Value {
    let mut oracle = document.value().clone();
    let [root] = oracle["content"].as_array_mut().unwrap().as_mut_slice() else {
        panic!("expected one captured namespace");
    };
    let package = &mut root["data"]["ownedRelationship"][0]["data"]["ownedRelatedElement"][0];
    let mut targets = package["data"]["ownedRelationship"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .flat_map(|relation| relation["data"]["ownedRelatedElement"].as_array_mut())
        .flatten()
        .filter(|candidate| candidate["data"]["elementId"] == semantic_id);
    let target = targets.next().unwrap();
    assert!(targets.next().is_none());
    assert_eq!(target["data"]["declaredName"], "A");
    target["data"]["declaredName"] = json!(EDITED);
    oracle
}

fn stage_edit(original: &Document, semantic_id: &str, oracle: &Value) -> Document {
    let mut editor = Editor::new(original.clone());
    let mut cancelled = editor.begin();
    cancelled.rename(semantic_id, CANCELLED).unwrap();
    assert_eq!(cancelled.document().element(semantic_id).unwrap()["data"]["declaredName"], CANCELLED);
    assert_eq!(editor.document(), original);
    cancelled.cancel();
    assert_eq!(editor.document(), original);
    assert!(!editor.undo() && !editor.redo());

    let mut edit = editor.begin();
    edit.rename(semantic_id, EDITED).unwrap();
    assert_eq!(edit.document().value(), oracle);
    assert_eq!(editor.document(), original);
    assert!(editor.commit(edit).unwrap());
    assert_eq!(editor.document().value(), oracle);
    assert!(editor.undo());
    assert_eq!(editor.document(), original);
    assert!(!editor.undo());
    assert!(editor.redo());
    assert_eq!(editor.document().value(), oracle);
    assert!(!editor.redo());
    editor.document().clone()
}

fn input(record: &Value, case: &str) -> Document {
    let raw = record["attempts"][1]["snapshots"][case]["data"].as_str().unwrap();
    Document::from_json(raw).unwrap()
}

fn part(record: &Value, store: &SnapshotStore, key: &str, abstract_part: bool, stage: bool) -> Value {
    let original = input(record, key);
    let before = inspect_part(&original, key, "A", abstract_part);
    let semantic = before["invocation_typing"]["type"]["element_id"].as_str().unwrap();
    let oracle = rename_expected(&original, semantic);
    let original_key = format!("{key}-original");
    let edit_key = format!("{key}-candidate");
    let candidate = if stage {
        let edited = stage_edit(&original, semantic, &oracle);
        let original_path = store.save_new(&original_key, &original).unwrap();
        let saved = fs::read(&original_path).unwrap();
        assert!(matches!(store.save_new(&original_key, &edited),
                         Err(StoreError::Io(error)) if error.kind() == io::ErrorKind::AlreadyExists));
        assert_eq!(fs::read(&original_path).unwrap(), saved);
        let retained = original_path.with_extension("staged");
        let retained = Document::from_json(&fs::read_to_string(retained).unwrap()).unwrap();
        assert_eq!(retained.value(), &oracle);
        store.save_new(&edit_key, &edited).unwrap();
        edited
    } else {
        store.load(&edit_key).unwrap()
    };
    assert_eq!(candidate.value(), &oracle);
    assert_eq!(store.load(&original_key).unwrap(), original);
    let after = inspect_part(&candidate, key, EDITED, abstract_part);
    assert_eq!(before, after);
    json!({"case": key, "original": original_key, "candidate": edit_key,
           "original_name": "A", "candidate_name": EDITED, "references": after})
}

fn calculation(record: &Value, store: &SnapshotStore, stage: bool) -> Value {
    let original = input(record, CALC_CASE);
    let key = format!("{CALC_CASE}-original");
    if stage {
        store.save_new(&key, &original).unwrap();
    }
    let loaded = store.load(&key).unwrap();
    assert_eq!(loaded, original);
    let package = package(loaded.value());
    let calculation = member(package, "sysml:CalculationDefinition", "F");
    let attribute = member(package, "sysml:AttributeDefinition", "N");
    let usage = member(package, "sysml:AttributeUsage", "x");
    let value = owned(usage, "ownedRelationship", "sysml:FeatureValue");
    let invocation = owned(value, "ownedRelatedElement", "sysml:InvocationExpression");
    let membership = owned(calculation, "ownedRelationship", "sysml:ReturnParameterMembership");
    let parameter = owned(membership, "ownedRelatedElement", "sysml:ReferenceUsage");
    assert_eq!(parameter["data"]["declaredName"], "r");
    assert_eq!(parameter["data"]["direction"], "out");
    let index = registry(&loaded, CALC_CASE);
    json!({"case": CALC_CASE, "original": key,
           "invocation_typing": inspect_link(&index, CALC_CASE, invocation,
                                             ClassifierId::InvocationExpression, calculation, ClassifierId::CalculationDefinition),
           "return_typing": inspect_link(&index, CALC_CASE, parameter,
                                         ClassifierId::ReferenceUsage, attribute, ClassifierId::AttributeDefinition)})
}

fn main() {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let [mode, path] = args.as_slice() else {
        panic!("expected stage or reopen and an absolute snapshot directory");
    };
    let mode = mode.to_str().unwrap();
    assert!(matches!(mode, "stage" | "reopen"));
    let root = PathBuf::from(path.as_os_str());
    assert!(root.is_absolute());
    let stage = mode == "stage";
    if stage {
        assert!(root.symlink_metadata().is_err_and(|e| e.kind() == io::ErrorKind::NotFound));
    } else {
        assert!(root.is_dir());
    }
    let store = SnapshotStore::open(root).unwrap();
    let record: Value = serde_json::from_str(PRIOR_RUST).unwrap();
    let mut rows = PART_CASES.iter().map(|&(case, flag)| part(&record, &store, case, flag, stage)).collect::<Vec<_>>();
    rows.push(calculation(&record, &store, stage));
    let controls = if stage {
        json!(["cancel_preserves_original", "cancel_creates_no_history", "stage_isolated_until_commit",
               "undo_restores_original", "redo_restores_candidate", "same_key_refuses_and_preserves_original"])
    } else {
        json!([])
    };
    println!("{}", json!({"phase": mode, "cases": rows, "edit_controls_per_part": controls}));
}
