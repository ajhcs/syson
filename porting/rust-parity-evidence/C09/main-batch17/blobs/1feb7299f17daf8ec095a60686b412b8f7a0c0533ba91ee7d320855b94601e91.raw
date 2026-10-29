// Development consumer of the public immutable reflection APIs.
// Stdout is JSON; no generated fixtures or oracle expectations are loaded.
// Agreement with a current-runtime Node capture does not establish native
// Docker Node20.15.1, whole-artifact or downstream workflow correspondence.
use serde_json::{Value, json};
use syson_reflection::{MandatoryKind, ReflectionDb, TypeHandle};

fn names(db: &ReflectionDb, handles: &[TypeHandle]) -> Vec<&'static str> {
    handles.iter().map(|ty| db.name(ty).unwrap()).collect()
}

fn returned(db: &ReflectionDb, handle: TypeHandle) -> Value {
    json!({"status": "return", "value": db.name(&handle).unwrap()})
}

fn main() {
    let db = ReflectionDb::pinned();
    let rows: Vec<Value> = db
        .all_generated_types()
        .iter()
        .map(|ty| {
            let mandatory: Vec<Value> = db
                .effective_mandatory(ty)
                .unwrap()
                .iter()
                .map(|p| {
                    let kind = match p.kind {
                        MandatoryKind::Array => "array",
                        MandatoryKind::Boolean => "boolean",
                    };
                    json!({"name": p.name, "kind": kind,
                "declared_by": p.declared_by.as_ref().map(|owner| db.name(owner).unwrap())})
                })
                .collect();
            let subtype_row: Vec<bool> = db
                .all_generated_types()
                .iter()
                .map(|supertype| db.is_subtype(ty, supertype).unwrap())
                .collect();
            json!({"name": db.name(ty).unwrap(),
            "ancestors": names(&db, db.proper_supertypes(ty).unwrap()),
            "descendants": names(&db, db.proper_subtypes(ty).unwrap()),
            "mandatory": mandatory, "subtype_row": subtype_row})
        })
        .collect();
    let reference_names = [
        "ClassifierReference",
        "ConjugatedPortReference",
        "ElementReference",
        "FeatureReference",
        "MembershipReference",
        "MetaclassReference",
        "NamespaceReference",
        "TypeReference",
    ];
    let generated_references: Vec<Value> = reference_names
        .iter()
        .map(|name| {
            let property_id = format!("{name}:parts");
            let target = db.generated_reference_target(&property_id).unwrap();
            json!({"property_id": property_id, "result": returned(&db, target)})
        })
        .collect();
    let mut positions = Vec::new();
    for name in reference_names.into_iter().chain(["Package"]) {
        let container = db.resolve_type(name).unwrap();
        for length in 1..=4 {
            for index in 0..length {
                let position = db.reference_position(&container, index, length).unwrap();
                let target = db.reference_target_type(&position).unwrap();
                positions.push(json!({"container": name, "index": index,
                    "length": length, "result": returned(&db, target)}));
            }
        }
    }
    // Debug refusals are separate Rust-domain observations, never normalized
    // into JavaScript return values or exceptions for equality comparison.
    let unknown_debug: Vec<Value> = ["", "NotAType", "__proto__", "Type\0", "type"]
        .into_iter()
        .map(|name| {
            json!({"name": name,
            "result_debug": format!("{:?}", db.resolve_type(name))})
        })
        .collect();
    let invalid_reference_debug: Vec<Value> = [
        "",
        "TypeReference:other",
        "Package:parts",
        "__proto__:parts",
    ]
    .into_iter()
    .map(|id| {
        json!({"property_id": id,
        "result_debug": format!("{:?}", db.generated_reference_target(id))})
    })
    .collect();
    let ty = db.resolve_type("TypeReference").unwrap();
    let invalid_position_debug: Vec<Value> = [(0, 0), (1, 1), (usize::MAX, 1)]
        .into_iter()
        .map(|(index, length)| {
            json!({"index": index, "length": length,
            "result_debug": format!("{:?}", db.reference_position(&ty, index, length))})
        })
        .collect();
    let observation = json!({
        "all_generated_types": names(&db, db.all_generated_types()),
        "ast_types_filtered": names(&db, db.ast_types_filtered()),
        "rows": rows, "generated_references": generated_references,
        "positions": positions,
        "rust_domain_refusals_debug": {"unknown_names": unknown_debug,
            "generated_references": invalid_reference_debug, "positions": invalid_position_debug},
        "ordering_qualification_debug": format!("{:?}", db.ordering_qualification()),
        "limits": "Public Rust API observation only; a matching pinned-bundle/current-Node capture does not bind the native Docker runtime, deployed artifact or consuming workflows.",
    });
    serde_json::to_writer_pretty(std::io::stdout().lock(), &observation).unwrap();
    println!();
}
