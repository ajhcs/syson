// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::{fs, path::Path};
use syson_metamodel::ClassifierId;
use syson_model::Document;
use syson_references::{self as refs, Decoded, Error, FragmentIdentity, Registry, Resource};

fn check(
    directory: &Path,
    name: &str,
    expected: ClassifierId,
    expected_declared_name: &str,
) -> Value {
    let wire = fs::read_to_string(directory.join(format!("{name}.wire"))).unwrap();
    let wire = wire.trim_end_matches('\n');
    let value = json!(wire);
    let Decoded::External {
        resource,
        fragment,
        class,
    } = refs::decode(Some(&value)).unwrap()
    else {
        panic!("not an external typed wire");
    };
    assert_eq!(class, Some(expected));
    let document = Document::from_json(&fs::read_to_string(directory.join(name)).unwrap()).unwrap();
    let registry = |fragments| {
        Registry::new(vec![Resource {
            key: resource.clone(),
            document: document.clone(),
            fragments,
        }])
        .unwrap()
    };
    let semantic = registry(FragmentIdentity::SemanticElementId);
    let object_id = registry(FragmentIdentity::JsonNodeId);
    let target = semantic
        .resolve("project-document", Some(&value), expected)
        .unwrap()
        .unwrap();
    assert_eq!(target.resource(), &resource);
    assert_eq!(target.element_id(), &fragment);
    assert_ne!(target.node_id(), &fragment);
    let node = semantic.target(&target).unwrap();
    assert_eq!(node["data"]["declaredName"], expected_declared_name);

    assert!(matches!(
        object_id.resolve("project-document", Some(&value), expected),
        Err(Error::Unresolved { .. })
    ));
    assert_eq!(object_id.target(&target), Err(Error::StaleSnapshot));
    let format_wire = |uri: &str, target_fragment: &str| {
        format!(
            "sysml:{} {uri}#{target_fragment}",
            expected.metadata().name()
        )
    };
    let emitted = format_wire(target.resource(), target.element_id());
    assert_eq!(emitted, wire);
    let raw_wire = json!(format_wire(target.resource(), target.node_id()));
    assert!(matches!(
        semantic.resolve("project-document", Some(&raw_wire), expected),
        Err(Error::Unresolved { .. })
    ));
    let raw_target = object_id
        .resolve("project-document", Some(&raw_wire), expected)
        .unwrap()
        .unwrap();
    assert_eq!(raw_target.node_id(), target.node_id());
    let (scheme, suffix) = resource.split_once(":///").unwrap();
    let other_scheme = if scheme == "kermllibrary" {
        "sysmllibrary"
    } else {
        "kermllibrary"
    };
    let wrong_uri = format!("{other_scheme}:///{suffix}");
    let wrong = json!(format_wire(&wrong_uri, &fragment));
    assert!(matches!(
        semantic.resolve("project-document", Some(&wrong), expected),
        Err(Error::Unresolved { .. })
    ));
    assert!(matches!(
        semantic.resolve("project-document", Some(&value), ClassifierId::Package),
        Err(Error::TypeMismatch { .. })
    ));
    let empty_registry = Registry::new(vec![]).unwrap();
    assert!(matches!(
        empty_registry.resolve("project-document", Some(&value), expected),
        Err(Error::Unresolved { .. })
    ));
    let wrong_kind = json!(format!("sysml:Package {resource}#{fragment}"));
    assert!(matches!(
        semantic.resolve("project-document", Some(&wrong_kind), expected),
        Err(Error::TypeMismatch { .. })
    ));
    json!({
        "source_file": name,
        "resource_uri": resource,
        "wire": emitted,
        "stored_id": target.node_id(),
        "stored_element_id": fragment,
        "declared_name": expected_declared_name,
        "negative": {
            "wrong_fragment_policy": "unresolved",
            "wrong_resource_scheme": "unresolved",
            "missing_resource": "unresolved",
            "wrong_target_class": "type_mismatch",
            "wrong_wire_class": "type_mismatch",
            "other_snapshot": "stale"
        }
    })
}

fn main() {
    let directory = std::env::args_os()
        .nth(1)
        .expect("resource directory argument");
    let directory = Path::new(&directory);
    let result = [
        check(
            directory,
            "ScalarValues.json",
            ClassifierId::DataType,
            "Integer",
        ),
        check(
            directory,
            "StandardViewDefinitions.json",
            ClassifierId::ViewDefinition,
            "GeneralView",
        ),
    ];
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
