// SPDX-License-Identifier: EPL-2.0
#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{env, path::PathBuf};
use syson_metamodel::ClassifierId;
use syson_model::Document;
use syson_references::{Error, FragmentIdentity, Registry, Resource, Target};
use syson_workflow::SnapshotStore;

const RELEASE_OBSERVATION: &str = include_str!(
    "../../../invocation-emf/observation.json"
);
const MISSING_NODE: &str = "00000000-0000-4000-8000-000000000000";

struct Case {
    key: &'static str,
    name: &'static str,
    kind: ClassifierId,
    abstract_definition: Option<bool>,
}

const CASES: [Case; 3] = [
    Case {
        key: "part-call",
        name: "A",
        kind: ClassifierId::PartDefinition,
        abstract_definition: Some(false),
    },
    Case {
        key: "abstract-part-call",
        name: "A",
        kind: ClassifierId::PartDefinition,
        abstract_definition: Some(true),
    },
    Case {
        key: "typed-calculation-call",
        name: "F",
        kind: ClassifierId::CalculationDefinition,
        abstract_definition: None,
    },
];

fn owned_one<'a>(owner: &'a Value, feature: &str, class: &str) -> &'a Value {
    let rows = owner["data"][feature].as_array().unwrap();
    let mut matches = rows.iter().filter(|node| node["eClass"] == class);
    let found = matches.next().unwrap();
    assert!(matches.next().is_none(), "expected one {class} in {feature}");
    found
}

fn declared<'a>(package: &'a Value, name: &str) -> &'a Value {
    let mut matches = package["data"]["ownedRelationship"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|relation| relation["data"]["ownedRelatedElement"].as_array())
        .flatten()
        .filter(|element| element["data"]["declaredName"] == name);
    let found = matches.next().unwrap();
    assert!(matches.next().is_none(), "expected one definition of {name}");
    found
}

fn registry(key: &str, document: &Document) -> Registry {
    Registry::new(vec![Resource {
        key: key.into(),
        document: document.clone(),
        fragments: FragmentIdentity::JsonNodeId,
    }])
    .unwrap()
}

fn resolve(
    index: &Registry,
    key: &str,
    wire: &Value,
    expected: &Value,
    kind: ClassifierId,
) -> Target {
    let resolved = index.resolve(key, Some(wire), kind).unwrap().unwrap();
    assert_eq!(resolved.resource(), key);
    assert_eq!(resolved.node_id(), expected["id"].as_str().unwrap());
    assert_eq!(
        resolved.element_id(),
        expected["data"]["elementId"].as_str().unwrap()
    );
    assert_ne!(resolved.node_id(), resolved.element_id());
    assert_eq!(index.target(&resolved).unwrap(), expected);
    resolved
}

fn verify(document: &Document, case: &Case) -> Value {
    let [root] = document.value()["content"].as_array().unwrap().as_slice() else {
        panic!("expected one release namespace");
    };
    assert_eq!(root["eClass"], "sysml:Namespace");
    let root_member = owned_one(root, "ownedRelationship", "sysml:OwningMembership");
    let package = owned_one(root_member, "ownedRelatedElement", "sysml:Package");
    assert_eq!(package["data"]["declaredName"], "P");

    let usage = declared(package, "x");
    let value = owned_one(usage, "ownedRelationship", "sysml:FeatureValue");
    let invocation = owned_one(value, "ownedRelatedElement", "sysml:InvocationExpression");
    let typing = owned_one(invocation, "ownedRelationship", "sysml:FeatureTyping");
    assert_eq!(
        invocation["data"]["ownedRelationship"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let target = declared(package, case.name);

    let index = registry(case.key, document);
    let type_wire = &typing["data"]["type"];
    let typed_wire = &typing["data"]["typedFeature"];
    let ty = resolve(&index, case.key, type_wire, target, case.kind);
    let typed = resolve(
        &index,
        case.key,
        typed_wire,
        invocation,
        ClassifierId::InvocationExpression,
    );
    assert_eq!(
        index
            .resolve(case.key, Some(type_wire), ClassifierId::Type)
            .unwrap()
            .unwrap()
            .node_id(),
        ty.node_id()
    );
    assert_eq!(
        index
            .resolve(case.key, Some(typed_wire), ClassifierId::Feature)
            .unwrap()
            .unwrap()
            .node_id(),
        typed.node_id()
    );

    let wrong_role = json!(ty.element_id());
    for (resource, wire) in [
        (case.key, &wrong_role),
        (case.key, &json!(MISSING_NODE)),
        ("missing-resource", type_wire),
    ] {
        assert!(matches!(
            index.resolve(resource, Some(wire), case.kind),
            Err(Error::Unresolved { .. })
        ));
    }
    assert!(matches!(
        index.resolve(case.key, Some(typed_wire), case.kind),
        Err(Error::TypeMismatch { .. })
    ));
    assert!(matches!(
        index.resolve(case.key, Some(type_wire), ClassifierId::InvocationExpression),
        Err(Error::TypeMismatch { .. })
    ));
    assert_eq!(
        registry(case.key, document).target(&typed),
        Err(Error::StaleSnapshot)
    );

    let additional = if let Some(expected) = case.abstract_definition {
        let actual = target["data"]["isAbstract"].as_bool().unwrap_or(false);
        assert_eq!(actual, expected);
        json!({"is_abstract": actual})
    } else {
        let membership = owned_one(target, "ownedRelationship", "sysml:ReturnParameterMembership");
        let parameter = owned_one(membership, "ownedRelatedElement", "sysml:ReferenceUsage");
        assert_eq!(parameter["data"]["declaredName"], "r");
        assert_eq!(parameter["data"]["direction"], "out");
        let parameter_typing = owned_one(parameter, "ownedRelationship", "sysml:FeatureTyping");
        let parameter_type = resolve(
            &index,
            case.key,
            &parameter_typing["data"]["type"],
            declared(package, "N"),
            ClassifierId::AttributeDefinition,
        );
        resolve(
            &index,
            case.key,
            &parameter_typing["data"]["typedFeature"],
            parameter,
            ClassifierId::ReferenceUsage,
        );
        json!({"return_parameter": "r", "direction": "out",
               "return_type": "N", "return_type_resource_id": parameter_type.node_id(),
               "return_type_element_id": parameter_type.element_id()})
    };

    json!({"case": case.key, "target_class": target["eClass"], "target": case.name,
           "type": {"resource_id": ty.node_id(), "element_id": ty.element_id()},
           "typed_feature": {"resource_id": typed.node_id(), "element_id": typed.element_id()},
           "controls_rejected": 6, "additional": additional})
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
    if mode == "stage" {
        assert!(
            root.symlink_metadata()
                .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        );
    } else {
        assert!(root.is_dir());
    }
    let store = SnapshotStore::open(root).unwrap();
    let observation: Value = serde_json::from_str(RELEASE_OBSERVATION).unwrap();
    let mut rows = Vec::new();
    for case in &CASES {
        let raw = observation["cases"][case.key]["files"]["model.emf.json"]["data"]
            .as_str()
            .unwrap();
        let value: Value = serde_json::from_str(raw).unwrap();
        let document = if mode == "stage" {
            Document::from_json(raw).unwrap()
        } else {
            store.load(case.key).unwrap()
        };
        assert_eq!(document.value(), &value);
        rows.push(verify(&document, case));
        if mode == "stage" {
            store.save_new(case.key, &document).unwrap();
        }
    }
    println!("{}", json!({"phase": mode, "cases": rows}));
}
