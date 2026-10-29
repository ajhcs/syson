// Development-only direct public AST construction observations; no oracle input.
use serde_json::{Value as Json, json};
use std::sync::{Arc, atomic::AtomicBool};
use syson_ast::{Datum, Lexeme, Value};
use syson_ast_adapter::{Adapter, Limits};
fn lexeme(value: &Lexeme) -> Json {
    json!({"text":value.text,"bytes":[value.span.bytes.start,value.span.bytes.end],
        "utf16":[value.span.utf16.start,value.span.utf16.end]})
}
fn datum(value: &Datum) -> Json {
    let item = match &value.value {
        Value::Null => json!({"kind":"null"}),
        Value::Boolean(v) => json!({"kind":"boolean","value":v}),
        Value::Number(v) => json!({"kind":"number","bits":format!("{:016x}",v.to_bits())}),
        Value::String(v) => json!({"kind":"string","units":v.encode_utf16().collect::<Vec<_>>()}),
        Value::Node(v) => json!({"kind":"node","id":v.value()}),
        Value::Array(v) => json!({"kind":"array","items":v.iter().map(datum).collect::<Vec<_>>()}),
        Value::UnresolvedReference { text, target_type } => {
            json!({"kind":"unresolved_reference", "text":text,"target_type":target_type})
        }
    };
    json!({"value":item,"lexeme":value.lexeme.as_ref().map(lexeme)})
}
fn main() {
    let path = std::env::args().nth(1).expect("explicit corpus path");
    let corpus: Json = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(corpus["schema"], 1);
    let adapter = Adapter::pinned().unwrap();
    let mut cases = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let source = case["source"].as_str().unwrap();
        let outcome =
            match adapter.construct(source, Limits::default(), Arc::new(AtomicBool::new(false))) {
                Ok(arena) => {
                    let nodes: Vec<_> = arena
                        .nodes()
                        .iter()
                        .map(|(id, node)| {
                            let fields: serde_json::Map<String, Json> = node
                                .fields()
                                .iter()
                                .map(|(key, value)| (key.clone(), datum(value)))
                                .collect();
                            json!({"id":id.value(),"node_type":node.kind(),"fields":fields,
                        "origin":node.origin().map(lexeme),"container":node.container().map(|c|
                            json!({"node":c.node.value(),"property":c.property,"index":c.index}))})
                        })
                        .collect();
                    json!({"kind":"arena","root":arena.root().value(),"nodes":nodes})
                }
                Err(error) => json!({"kind":"rust_error","error":format!("{error:?}")}),
            };
        cases.push(json!({"id":case["id"],"source":source,"outcome":outcome}));
    }
    println!(
        "{}",
        json!({"schema":1,"cases":cases,"scope":"All public Arena fields, origins and containers; no native projection or expectation inputs. Finite construction only; no linker/importer/UI acceptance."})
    );
}
