use std::{
    io::{Read, Write},
    process::ExitCode,
};
use serde_json::json;
use syson_model::{Document, NodeIds};

fn allocated(prefix: &str, n: usize) -> String {
    format!("{prefix}000000-0000-4000-8000-{n:012x}")
}

fn imported_envelope(source: Document) -> Result<Document, String> {
    let mut document = source.value().clone();
    let roots = document["content"].as_array_mut().ok_or("missing content")?;
    if roots.is_empty() || roots.iter().any(|root| root["eClass"] != "sysml:Package") {
        return Err("expected unwrapped finite package roots".into());
    }

    let mut memberships = Vec::with_capacity(roots.len());
    for (i, existing) in std::mem::take(roots).into_iter().enumerate() {
        let member_id = NodeIds::new(&allocated("31", i + 1), &allocated("41", i + 1))
            .map_err(|error| format!("{error:?}"))?;
        memberships.push(json!({
            "id": member_id.resource, "eClass": "sysml:OwningMembership",
            "data": {"elementId": member_id.element, "ownedRelatedElement": [existing]},
        }));
    }
    let root_id = NodeIds::new(&allocated("31", 0), &allocated("41", 0))
        .map_err(|error| format!("{error:?}"))?;
    roots.push(json!({
        "id": root_id.resource, "eClass": "sysml:Namespace",
        "data": {
            "elementId": root_id.element,
            "eAnnotations": [{"source": "org.eclipse.syson.sysml.imported"}],
            "ownedRelationship": memberships
        },
    }));
    Document::from_json(&document.to_string()).map_err(|error| format!("{error:?}"))
}

fn run() -> Result<(), String> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).map_err(|error| error.to_string())?;
    let selected = Document::from_json(&input).map_err(|error| format!("{error:?}"))?;
    let wrapped = imported_envelope(selected)?;
    let saved = wrapped.to_json().map_err(|error| format!("{error:?}"))?;
    let reparsed = Document::from_json(&saved).map_err(|error| format!("{error:?}"))?;
    if reparsed != wrapped {
        return Err("the wrapped model changed during save and reopen".into());
    }
    std::io::stdout().write_all(saved.as_bytes()).map_err(|error| error.to_string())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
