// Development-only actual selected reference consumer, not an expectation oracle.
use serde_json::{Value, json};
use syson_metamodel::ClassifierId;
use syson_model::Document;
use syson_references::{Decoded, FragmentIdentity, Registry, Resource, decode};
fn tuple(node: &Value, resource: &str) -> Value {
    json!({"status":"resolved","resource":resource,"node_id":node["id"],"element_id":node["data"]["elementId"],"class":node["eClass"].as_str().unwrap().strip_prefix("sysml:").unwrap(),"name":node["data"]["declaredName"]})
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    let vehicle = Document::from_json(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();
    let views = Document::from_json(&std::fs::read_to_string(&args[2]).unwrap()).unwrap();
    let mut inputs = Vec::new();
    for (label, id) in [
        ("local", "775ea645-9c42-4e52-8cd8-de30ed8b865b"),
        ("external", "5dba8007-30b4-4988-9d3f-d5687ba94faa"),
    ] {
        let node = vehicle.element(id).unwrap();
        inputs.push((label, node.clone(), node["data"]["type"].clone()));
    }
    let Decoded::External { resource: uri, .. } = decode(Some(&inputs[1].2)).unwrap() else {
        panic!("actual external wire required")
    };
    let registry = Registry::new(vec![
        Resource {
            key: "vehicle".into(),
            document: vehicle,
            fragments: FragmentIdentity::SemanticElementId,
        },
        Resource {
            key: uri,
            document: views,
            fragments: FragmentIdentity::SemanticElementId,
        },
    ])
    .unwrap();
    let mut cases = serde_json::Map::new();
    for (label, source, wire) in inputs {
        let target = registry
            .resolve("vehicle", Some(&wire), ClassifierId::Type)
            .unwrap()
            .unwrap();
        let node = registry.target(&target).unwrap();
        cases.insert(label.into(),json!({"source":tuple(&source,"vehicle"),"target":tuple(node,target.resource()),"wire":wire,"type_is_Type":true}));
    }
    let missing = json!(
        "sysmllibrary:///00000000-0000-4000-8000-000000000099#00000000-0000-4000-8000-000000000099"
    );
    let absent = matches!(
        registry.resolve("vehicle", Some(&missing), ClassifierId::Type),
        Err(syson_references::Error::Unresolved { .. })
    );
    println!(
        "{}",
        json!({"cases":cases,"missing_without_load":absent,"local_context_mapping":{"vehicle":"memory:/Batmobile.json"},"scope":"Exact selected wire targets; class prefix projects admitted sysml QName to EClass name. Explicit identity policies are not native alias generality."})
    );
}
