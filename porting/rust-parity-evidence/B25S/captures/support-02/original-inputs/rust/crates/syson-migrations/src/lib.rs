// SPDX-License-Identifier: EPL-2.0
//! Bounded immutable JSON preparation matching selected native migration helpers.
#![forbid(unsafe_code)]
use serde::de::{Deserialize, Deserializer, IgnoredAny, MapAccess, Visitor};
use serde_json::{Value, value::RawValue};
use std::fmt;
use syson_metamodel::{Package, is_subtype, lookup};

pub const MAX_BYTES: usize = 1024 * 1024;
pub const MAX_DEPTH: usize = 64;
pub const MAX_VALUES: usize = 65_536;
pub const SYSML_URI: &str = "http://www.eclipse.org/syson/sysml";

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Input(String),
    NamespaceShape,
    UnknownClass(String),
    Limit,
}
#[derive(Clone, Copy)]
pub struct Removal {
    pub class: &'static str,
    pub features: &'static [&'static str],
}
#[derive(Debug, PartialEq)]
pub struct Prepared {
    pub json: Value,
    /// Selected participant version only; native migration history is external.
    pub participant_version: &'static str,
    pub matched_resource: bool,
}
struct Namespace(Option<Box<RawValue>>);
impl<'de> Deserialize<'de> for Namespace {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Root;
        impl<'de> Visitor<'de> for Root {
            type Value = Namespace;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a resource object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Namespace, A::Error> {
                let mut ns = None;
                while let Some(key) = map.next_key::<String>()? {
                    if key == "ns" {
                        ns = Some(map.next_value()?);
                    } else {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
                Ok(Namespace(ns))
            }
        }
        deserializer.deserialize_map(Root)
    }
}
struct First(Option<Value>);
impl<'de> Deserialize<'de> for First {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Entries;
        impl<'de> Visitor<'de> for Entries {
            type Value = First;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a namespace object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<First, A::Error> {
                let mut first_key = None;
                let mut first = None;
                while let Some(key) = map.next_key::<String>()? {
                    if first_key.is_none() {
                        first_key = Some(key.clone());
                    }
                    if first_key.as_ref() == Some(&key) {
                        first = Some(map.next_value()?);
                    } else {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
                Ok(First(first))
            }
        }
        deserializer.deserialize_map(Entries)
    }
}
fn sysml(input: &str) -> Result<bool, Error> {
    let Namespace(ns) = serde_json::from_str(input).map_err(|e| Error::Input(e.to_string()))?;
    let Some(ns) = ns.filter(|v| v.get().trim_start().starts_with('{')) else {
        return Ok(false);
    };
    let First(first) = serde_json::from_str(ns.get()).map_err(|e| Error::Input(e.to_string()))?;
    match first {
        None => Ok(false),
        Some(Value::String(s)) => Ok(s == SYSML_URI),
        Some(Value::Bool(_) | Value::Number(_)) => Ok(false),
        _ => Err(Error::NamespaceShape),
    }
}
fn bounded(value: &Value, depth: usize, count: &mut usize) -> Result<(), Error> {
    *count += 1;
    if depth > MAX_DEPTH || *count > MAX_VALUES {
        return Err(Error::Limit);
    }
    match value {
        Value::Object(o) => {
            for v in o.values() {
                bounded(v, depth + 1, count)?;
            }
        }
        Value::Array(a) => {
            for v in a {
                bounded(v, depth + 1, count)?;
            }
        }
        _ => {}
    }
    Ok(())
}
// Object preorder IDs identify targets without copying ancestor keys. Applying
// removals postorder visits every original object before an ancestor can detach
// its subtree; the returned tree matches the native collected-object effects.
fn collect(
    value: &Value,
    class: syson_metamodel::ClassifierId,
    selected: bool,
    next: &mut usize,
    targets: &mut Vec<usize>,
) {
    match value {
        Value::Object(object) => {
            let id = *next;
            *next += 1;
            if selected {
                targets.push(id);
            }
            let matches = object
                .get("eClass")
                .and_then(Value::as_str)
                .and_then(|s| s.strip_prefix("sysml:"))
                .and_then(|s| lookup(Package::Domain, s))
                .is_some_and(|child| is_subtype(child, class));
            for child in object.values() {
                collect(child, class, matches && child.is_object(), next, targets);
            }
        }
        Value::Array(array) => {
            for child in array {
                collect(child, class, false, next, targets);
            }
        }
        _ => {}
    }
}
fn remove(value: &mut Value, next: &mut usize, targets: &[usize], features: &[&str]) {
    match value {
        Value::Object(object) => {
            let id = *next;
            *next += 1;
            for child in object.values_mut() {
                remove(child, next, targets, features);
            }
            if targets.binary_search(&id).is_ok() {
                for feature in features {
                    object.remove(*feature);
                }
            }
        }
        Value::Array(array) => {
            for child in array {
                remove(child, next, targets, features);
            }
        }
        _ => {}
    }
}
/// Parse original bytes to preserve the native first-namespace-entry predicate.
/// Collect each pass before removal, and run passes in their declared order.
/// Object-child selection intentionally matches the native helper's quirk.
/// No version persistence, SQL, resource loading or process delegate occurs.
pub fn prepare(input: &str, version: &'static str, passes: &[Removal]) -> Result<Prepared, Error> {
    if input.len() > MAX_BYTES {
        return Err(Error::Limit);
    }
    let mut json: Value = serde_json::from_str(input).map_err(|e| Error::Input(e.to_string()))?;
    if !json.is_object() {
        return Err(Error::Input("resource root must be an object".into()));
    }
    bounded(&json, 0, &mut 0)?;
    let matched_resource = sysml(input)?;
    if matched_resource {
        for pass in passes {
            let class = lookup(Package::Domain, pass.class)
                .ok_or_else(|| Error::UnknownClass(pass.class.into()))?;
            let mut targets = Vec::new();
            collect(&json, class, false, &mut 0, &mut targets);
            remove(&mut json, &mut 0, &targets, pass.features);
        }
    }
    Ok(Prepared {
        json,
        participant_version: version,
        matched_resource,
    })
}
