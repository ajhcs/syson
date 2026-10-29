// SPDX-License-Identifier: EPL-2.0
//! Developer-only public F09 observations; no callback or parser executes.
use serde_json::{Value, json};
use std::sync::Arc;
use syson_ast_dispatch::{ChainOrder, ConstructInput, Entry, Falsey, Registry, parser_registry};
use syson_reflection::ReflectionDb;
type Result<T> = std::result::Result<T, String>;
fn error(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
fn name<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string {key}"))
}
fn array<'a>(items: impl IntoIterator<Item = &'a Value>) -> Value {
    json!({"kind":"array","items":items.into_iter().cloned().collect::<Vec<_>>()})
}
fn undefined() -> Value {
    json!({"kind":"undefined"})
}
fn entry(v: &Value) -> Result<Entry<Value>> {
    let falsey = match name(v, "kind")? {
        "undefined" => Some(Falsey::Undefined),
        "null" => Some(Falsey::Null),
        "boolean" => {
            if v["value"].as_bool().ok_or("boolean value required")? {
                None
            } else {
                Some(Falsey::False)
            }
        }
        "string" => {
            if name(v, "value")?.is_empty() {
                Some(Falsey::EmptyString)
            } else {
                None
            }
        }
        "number" => {
            let bits = name(v, "bits")?;
            if bits.len() != 16
                || !bits
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err("number requires 16 lowercase hexadecimal bits".into());
            }
            let number = f64::from_bits(u64::from_str_radix(bits, 16).map_err(error)?);
            if number.is_nan() {
                Some(Falsey::NaN)
            } else if number == 0.0 {
                Some(Falsey::Zero)
            } else {
                None
            }
        }
        "array" => {
            v["items"].as_array().ok_or("array items required")?;
            None
        }
        _ => return Err("payload kind is outside the typed comparison domain".into()),
    };
    Ok(falsey.map_or_else(|| Entry::Truthy(v.clone()), Entry::Falsey))
}
fn input(v: &Value) -> Result<ConstructInput<'_>> {
    match name(v, "kind")? {
        "undefined" => Ok(ConstructInput::Undefined),
        "symbol" => {
            name(v, "description")?;
            Ok(ConstructInput::Symbol)
        }
        "string" => Ok(ConstructInput::String(name(v, "value")?)),
        _ => Err("construct input is outside String/Symbol/Undefined domain".into()),
    }
}
fn values(
    db: &ReflectionDb,
    c: &Value,
) -> Result<Vec<(syson_reflection::TypeHandle, Entry<Value>)>> {
    c["entries"]
        .as_array()
        .ok_or("entries array required")?
        .iter()
        .map(|r| {
            Ok((
                db.resolve_type(name(r, "name")?).map_err(error)?,
                entry(&r["value"])?,
            ))
        })
        .collect()
}
fn observe(c: &Value, db: Arc<ReflectionDb>) -> Result<Value> {
    let method = name(c, "method")?;
    let all = match name(c, "mode")? {
        "all" => true,
        "lookup" => false,
        _ => return Err("unknown mode".into()),
    };
    if name(c, "constructor")? == "SysMLCstNodeBuilder" {
        if method != "postprocessingMap" {
            return Err("builder exposes only the pinned registry here".into());
        }
        let registry = parser_registry(db.clone()).map_err(error)?;
        let encode = |p: &syson_ast_dispatch::Postprocessor| json!({"kind":"postprocessor","name":format!("{p:?}")});
        if all {
            let mut rows = Vec::new();
            for (ty, p) in registry.expand_most_derived().map_err(error)? {
                rows.push(json!([db.name(&ty).map_err(error)?, encode(p)]));
            }
            return Ok(json!({"kind":"map","entries":rows}));
        }
        return Ok(registry
            .construct_plan(input(&c["input"])?)
            .map_err(error)?
            .map_or_else(undefined, encode));
    }
    if name(c, "constructor")? != "TypesIndex" {
        return Err("unknown constructor".into());
    }
    if method == "expandAndMerge" {
        let entries = values(&db, c)?
            .into_iter()
            .map(|(ty, value)| {
                let block = match value {
                    Entry::Falsey(f) => Entry::Falsey(f),
                    Entry::Truthy(v) => Entry::Truthy(
                        v["items"]
                            .as_array()
                            .ok_or("truthy merge payload must be an array")?
                            .clone(),
                    ),
                };
                Ok((ty, block))
            })
            .collect::<Result<Vec<_>>>()?;
        // The actual public merge API ignores default; it is not reinterpreted.
        let registry = Registry::new(db.clone(), entries, None).map_err(error)?;
        let reverse = c["reverse"].as_bool().ok_or("reverse bool required")?;
        if all {
            let mut rows = Vec::new();
            for ty in db.all_generated_types() {
                rows.push(json!([
                    db.name(ty).map_err(error)?,
                    array(registry.merge_blocks(ty, reverse).map_err(error)?)
                ]));
            }
            return Ok(json!({"kind":"map","entries":rows}));
        }
        let ConstructInput::String(key) = input(&c["input"])? else {
            return Err("merge lookup requires an admitted string type".into());
        };
        let ty = db.resolve_type(key).map_err(error)?;
        return Ok(array(registry.merge_blocks(&ty, reverse).map_err(error)?));
    }
    let registry =
        Registry::new(db.clone(), values(&db, c)?, Some(entry(&c["default"])?)).map_err(error)?;
    if method == "expandToDerivedTypes" {
        if all {
            let mut rows = Vec::new();
            for (ty, value) in registry.expand_most_derived().map_err(error)? {
                rows.push(json!([db.name(&ty).map_err(error)?, value]));
            }
            return Ok(json!({"kind":"map","entries":rows}));
        }
        return Ok(registry
            .construct_plan(input(&c["input"])?)
            .map_err(error)?
            .cloned()
            .unwrap_or_else(undefined));
    }
    if method != "chain" {
        return Err("unknown method".into());
    }
    let order = match name(c, "order")? {
        "subtype-first" => ChainOrder::SubtypeFirst,
        "supertype-first" => ChainOrder::SupertypeFirst,
        _ => return Err("unknown chain order".into()),
    };
    if all {
        let mut rows = Vec::new();
        for ty in db.all_generated_types() {
            rows.push(json!([
                db.name(ty).map_err(error)?,
                array(registry.chain(ty, order).map_err(error)?)
            ]));
        }
        return Ok(json!({"kind":"map","entries":rows}));
    }
    let ConstructInput::String(key) = input(&c["input"])? else {
        return Err("chain lookup requires an admitted string type".into());
    };
    let ty = db.resolve_type(key).map_err(error)?;
    Ok(array(registry.chain(&ty, order).map_err(error)?))
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 2, "one explicit corpus path required");
    assert!(std::path::Path::new(&args[1]).is_absolute());
    let corpus: Value = serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    assert_eq!(corpus["schema"], 1);
    let db = Arc::new(ReflectionDb::pinned());
    let observations: Vec<_> = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let value =
                observe(c, db.clone()).unwrap_or_else(|e| json!({"kind":"rust_error","name":e}));
            json!({"id":c["id"],"value":value})
        })
        .collect();
    println!(
        "{}",
        json!({"schema":1,"observations":observations,
        "postprocessor_bindings":[
            {"rust_id":"FixOperatorExpression","source_registration":"OperatorExpression","source_symbol":"fixOperatorExpression"},
            {"rust_id":"AddLoopMember","source_registration":"WhileLoopActionUsage","source_symbol":"addLoopMember"},
            {"rust_id":"FinalizeImport","source_registration":"Import","source_symbol":"finalizeImport"},
            {"rust_id":"CreateEmptyTransitionParameters","source_registration":"TransitionUsage","source_symbol":"createEmptyParametersInTransitionUsage"},
            {"rust_id":"CreateMissingSuccessionEnds","source_registration":"SuccessionAsUsage","source_symbol":"createMissingEndsInSuccessionAsUsage"}],
        "scope":"Actual public immutable dispatch plans. Descriptor values/order are observed; shared JS object identity, native function equality/arity/execution, parser callbacks, AST defaults/caches and application parity remain unqualified."})
    );
}
