// SPDX-License-Identifier: EPL-2.0
//! Developer-only actual public F12/kernel calls, independent of native output.
use serde_json::{Value as Json, json};
use std::sync::{Arc, atomic::AtomicBool};
use syson_ast::{Builder, Datum, Lexeme, Limits, Node, NodeId, Slot, Value};
use syson_ast_adapter::operator_expression::{OperatorPostprocessor, Report};
type Result<T> = std::result::Result<T, String>;
fn error(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
fn text<'a>(v: &'a Json, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Json::as_str)
        .ok_or_else(|| format!("missing string {key}"))
}
fn has_undefined(v: &Json) -> bool {
    match v {
        Json::Object(fields) => {
            fields.get("kind").and_then(Json::as_str) == Some("undefined")
                || fields.values().any(has_undefined)
        }
        Json::Array(items) => items.iter().any(has_undefined),
        _ => false,
    }
}
fn build_node(
    b: &mut Builder,
    spec: &Json,
    known: &mut Vec<NodeId>,
    setup: &mut Vec<Json>,
) -> Result<NodeId> {
    let id = b
        .allocate(Some(text(spec, "node_type")?), None)
        .map_err(error)?;
    known.push(id);
    let bare = spec["fields"]["operands"]["kind"].as_str() == Some("absent");
    if !bare {
        b.defaults(id).map_err(error)?;
    }
    setup.push(json!({"node_type":text(spec,"node_type")?,"setup":if bare {"bare_synthetic_fields"} else {"defaulted_creation"}}));
    for (key, descriptor) in spec["fields"].as_object().ok_or("node fields object")? {
        if text(descriptor, "kind")? != "absent" {
            let value = decode(b, descriptor, known, setup)?;
            b.assign(id, key, syson_ast::Assignment::Set, Datum::new(value))
                .map_err(error)?;
        }
    }
    Ok(id)
}
fn decode(
    b: &mut Builder,
    d: &Json,
    known: &mut Vec<NodeId>,
    setup: &mut Vec<Json>,
) -> Result<Value> {
    Ok(match text(d, "kind")? {
        "null" => Value::Null,
        "boolean" => Value::Boolean(d["value"].as_bool().ok_or("boolean descriptor")?),
        "string" => Value::String(text(d, "value")?.into()),
        "number" => {
            let bits = text(d, "bits")?;
            if bits.len() != 16
                || !bits
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            {
                return Err("16 lowercase binary64 hexadecimal digits required".into());
            }
            Value::Number(f64::from_bits(
                u64::from_str_radix(bits, 16).map_err(error)?,
            ))
        }
        "node" => Value::Node(build_node(b, d, known, setup)?),
        "array" => Value::Array(
            d["items"]
                .as_array()
                .ok_or("array descriptor")?
                .iter()
                .map(|item| decode(b, item, known, setup).map(Datum::new))
                .collect::<Result<_>>()?,
        ),
        _ => return Err("unsupported synthetic input descriptor".into()),
    })
}
fn lexeme(v: &Lexeme) -> Json {
    json!({"text":v.text,"bytes":[v.span.bytes.start,v.span.bytes.end],"utf16":[v.span.utf16.start,v.span.utf16.end]})
}
fn datum(d: &Datum) -> Json {
    let value = match &d.value {
        Value::Null => json!({"kind":"null"}),
        Value::Boolean(v) => json!({"kind":"boolean","value":v}),
        Value::Number(v) => json!({"kind":"number","bits":format!("{:016x}",v.to_bits())}),
        Value::String(v) => json!({"kind":"string","units":v.encode_utf16().collect::<Vec<_>>()}),
        Value::Node(id) => json!({"kind":"node","id":id.value()}),
        Value::Array(items) => {
            json!({"kind":"array","items":items.iter().map(datum).collect::<Vec<_>>()})
        }
        Value::UnresolvedReference { text, target_type } => {
            json!({"kind":"unresolved_reference","text":text,"target_type":target_type})
        }
    };
    json!({"value":value,"lexeme":d.lexeme.as_ref().map(lexeme)})
}
fn node(id: NodeId, n: &Node) -> Json {
    let fields: serde_json::Map<String, Json> = n
        .fields()
        .iter()
        .map(|(key, value)| (key.clone(), datum(value)))
        .collect();
    json!({"id":id.value(),"node_type":n.kind(),"fields":fields,"origin":n.origin().map(lexeme),
        "container":n.container().map(|c|json!({"node":c.node.value(),"property":c.property,"index":c.index}))})
}
fn snapshot(b: &Builder, root: NodeId, known: &[NodeId]) -> Result<Json> {
    let nodes = known
        .iter()
        .map(|id| b.node(*id).map(|n| node(*id, n)).map_err(error))
        .collect::<Result<Vec<_>>>()?;
    Ok(
        json!({"root":root.value(),"nodes":nodes,"scope":"All public fields for tracked input nodes and F12 Report IDs; private builder/orphan state unexposed"}),
    )
}
fn locate(b: &Builder, root: NodeId, path: &str) -> Result<NodeId> {
    if path == "root" {
        return Ok(root);
    }
    let mut parts = path
        .strip_prefix("root/")
        .ok_or("root target path required")?
        .split('/');
    let mut current = root;
    while let Some(key) = parts.next() {
        let field = b.node(current).map_err(error)?.slot(key);
        current = match field {
            Slot::Present(Datum {
                value: Value::Node(id),
                ..
            }) => *id,
            Slot::Present(Datum {
                value: Value::Array(items),
                ..
            }) => {
                let index: usize = parts
                    .next()
                    .ok_or("array target index")?
                    .parse()
                    .map_err(error)?;
                match &items.get(index).ok_or("target array bounds")?.value {
                    Value::Node(id) => *id,
                    _ => return Err("target array element is not node".into()),
                }
            }
            _ => return Err("target property is not node".into()),
        };
    }
    Ok(current)
}
fn observe(p: &OperatorPostprocessor, spec: &Json) -> Result<Json> {
    let id = text(spec, "id")?;
    if has_undefined(&spec["root"]) {
        return Ok(
            json!({"id":id,"domain":spec["domain"],"steps":[],"outcome":{"kind":"input_unsupported","reason":"Explicit undefined has no public Rust Value variant; it is not normalized to an absent field"}}),
        );
    }
    let mut b = p
        .builder(Limits::default(), Arc::new(AtomicBool::new(false)))
        .map_err(error)?;
    let mut known = Vec::new();
    let mut setup = Vec::new();
    let root = build_node(&mut b, &spec["root"], &mut known, &mut setup)?;
    let before = snapshot(&b, root, &known)?;
    let mut steps = Vec::new();
    for path in spec["targets"].as_array().ok_or("targets array")? {
        let path = path.as_str().ok_or("target path string")?;
        let target = locate(&b, root, path)?;
        let step_before = snapshot(&b, root, &known)?;
        let report = match p.apply(&mut b, target) {
            Ok(Report::NotSelected) => json!({"kind":"NotSelected"}),
            Ok(Report::Unchanged {
                normalized_operands,
            }) => json!({"kind":"Unchanged","normalized_operands":normalized_operands}),
            Ok(Report::Inserted {
                expression,
                membership,
                feature,
                normalized_operands,
            }) => {
                known.extend([expression, membership, feature]);
                json!({"kind":"Inserted","expression":expression.value(),"membership":membership.value(),"feature":feature.value(),"normalized_operands":normalized_operands})
            }
            Err(e) => {
                let after = snapshot(&b, root, &known)?;
                steps.push(json!({"target_path":path,"report":{"kind":"rust_error","error":error(e)},"before":step_before,"after":after}));
                return Ok(
                    json!({"id":id,"domain":spec["domain"],"setup_calls":setup,"before":before,"steps":steps,"outcome":{"kind":"rust_error","published_arena":false}}),
                );
            }
        };
        steps.push(json!({"target_path":path,"report":report,"before":step_before,"after":snapshot(&b,root,&known)?}));
    }
    let after_apply = snapshot(&b, root, &known)?;
    let outcome = match b.finish(root) {
        Ok(arena) => {
            json!({"kind":"arena","root":arena.root().value(),"nodes":arena.nodes().iter().map(|(id,n)|node(*id,n)).collect::<Vec<_>>()})
        }
        Err(e) => json!({"kind":"finish_error","error":error(e),"published_arena":false}),
    };
    Ok(
        json!({"id":id,"domain":spec["domain"],"setup_calls":setup,"before":before,"steps":steps,"after_apply":after_apply,"finalization":{"kind":"Builder::finish"},"outcome":outcome}),
    )
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 2, "one explicit corpus path required");
    let path = std::path::Path::new(&args[1]);
    assert!(path.is_absolute());
    let path = path.canonicalize().unwrap();
    assert!(path.starts_with("/workspace") && path.is_file());
    let raw = std::fs::read(&path).unwrap();
    let corpus: Json = serde_json::from_slice(&raw).unwrap();
    assert_eq!(corpus["schema"], 1);
    assert_eq!(corpus["cases"].as_array().unwrap().len(), 42);
    let processor = OperatorPostprocessor::pinned().unwrap();
    let cases: Vec<_> = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|spec| {
            observe(&processor, spec).unwrap_or_else(
                |e| json!({"id":spec["id"],"domain":spec["domain"],"capture_error":e}),
            )
        })
        .collect();
    println!(
        "{}",
        json!({"schema":1,"cases":cases,"corpus_path":path,"corpus_bytes":raw.len(),
        "scope":"Actual public F12 apply/builder/finish on synthetic inputs. Native void return is not a Rust Report. Caller receipt must bind corpus/source/binary bytes. No parser, linking, import, UI or feature acceptance."})
    );
}
