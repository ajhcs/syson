// SPDX-License-Identifier: EPL-2.0
//! Dev-only read-only observation of exact P01 fixture bytes. No expected result
//! is used to derive observed declarations, spans, reference segments or errors.
use serde_json::{Value, json};
use std::{
    error::Error,
    fs,
    sync::{Arc, atomic::AtomicBool},
};
use syson_ast::{Lexeme, Value as AstValue};
use syson_ast_adapter::{Adapter, Error as AdapterError, Limits};

fn span(lexeme: &Lexeme) -> Value {
    json!({
        "text": lexeme.text,
        "bytes": [lexeme.span.bytes.start, lexeme.span.bytes.end],
        "utf16": [lexeme.span.utf16.start, lexeme.span.utf16.end]
    })
}
fn error(error: AdapterError) -> Value {
    match error {
        AdapterError::Malformed { offset, expected } => json!({
            "kind": "Malformed", "offset_bytes": offset, "expected": expected
        }),
        AdapterError::Unsupported { offset } => json!({
            "kind": "Unsupported", "offset_bytes": offset
        }),
        AdapterError::Limit => json!({"kind": "Limit"}),
        AdapterError::InvalidLimits => json!({"kind": "InvalidLimits"}),
        AdapterError::Cancelled => json!({"kind": "Cancelled"}),
        AdapterError::Kernel(cause) => json!({"kind": "Kernel", "detail": format!("{cause:?}")}),
        AdapterError::Metadata => json!({"kind": "Metadata"}),
        AdapterError::Conversion => json!({"kind": "Conversion"}),
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let manifest = args.next().ok_or("expected one fixture manifest path")?;
    if args.next().is_some() {
        return Err("expected exactly one argument".into());
    }
    let packet: Value = serde_json::from_slice(&fs::read(&manifest)?)?;
    let fixtures = packet["fixtures"]
        .as_array()
        .ok_or("fixtures must be an array")?;
    if fixtures.len() != 10 {
        return Err("expected original ten-fixture packet".into());
    }
    let adapter = Adapter::pinned().map_err(|e| format!("adapter initialization: {e:?}"))?;
    let mut results = Vec::new();
    for fixture in fixtures {
        let id = fixture["id"]
            .as_str()
            .ok_or("fixture id must be a string")?;
        let source = fixture["source"]
            .as_str()
            .ok_or("fixture source must be a string")?;
        let path = fixture["path"]
            .as_str()
            .ok_or("fixture path must be a string")?;
        if fs::read(path)? != source.as_bytes() {
            return Err(format!(
                "fixture {id}: supplied file differs from original manifest bytes"
            )
            .into());
        }
        let mut row = json!({
            "id": id, "source": source, "source_path": path,
            "source_bytes": source.len(), "source_utf16": source.encode_utf16().count(),
            "manifest_source_matches_file_bytes": true
        });
        match adapter.construct(source, Limits::default(), Arc::new(AtomicBool::new(false))) {
            Err(cause) => {
                row["status"] = json!("REFUSED");
                row["error"] = error(cause);
            }
            Ok(arena) => {
                let mut declarations = Vec::new();
                let mut references = Vec::new();
                let mut all_nodes = Vec::new();
                for node in arena.nodes().values() {
                    all_nodes.push(json!({"kind": node.kind(), "origin": node.origin().map(span)}));
                    if matches!(
                        node.kind(),
                        Some("Package" | "PartDefinition" | "PartUsage")
                    ) {
                        let declared_name = node.fields().get("declaredName");
                        let raw = match declared_name.map(|datum| &datum.value) {
                            Some(AstValue::String(raw)) => Some(raw.as_str()),
                            None => None,
                            _ => return Err("unexpected non-string declaration name".into()),
                        };
                        declarations.push(json!({
                            "kind": node.kind(), "raw_declaredName": raw,
                            "name_lexeme": declared_name.and_then(|datum| datum.lexeme.as_ref()).map(span),
                            "origin": node.origin().map(span)
                        }));
                    }
                    if node.kind() == Some("TypeReference") {
                        let Some(parts) = node.fields().get("parts") else {
                            return Err("missing parts".into());
                        };
                        let AstValue::Array(parts) = &parts.value else {
                            return Err("non-array parts".into());
                        };
                        let mut segments = Vec::new();
                        for part in parts {
                            let AstValue::UnresolvedReference { text, target_type } = &part.value
                            else {
                                return Err("unexpected resolved/non-reference segment".into());
                            };
                            segments.push(json!({
                                "text": text, "target_type": target_type,
                                "lexeme": part.lexeme.as_ref().map(span)
                            }));
                        }
                        references
                            .push(json!({"origin": node.origin().map(span), "parts": segments}));
                    }
                }
                row["status"] = json!("CONSTRUCTED");
                row["root_origin"] =
                    json!(arena.node(arena.root()).and_then(|n| n.origin()).map(span));
                row["declarations_preorder"] = json!(declarations);
                row["type_references"] = json!(references);
                row["all_nodes"] = json!(all_nodes);
            }
        }
        results.push(row);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "scope": "Actual finite Adapter::construct observations; no linking, Java translation, recovery or full-language validity verdict",
            "manifest_path": manifest, "results": results
        }))?
    );
    Ok(())
}
