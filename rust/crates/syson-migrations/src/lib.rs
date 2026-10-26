// SPDX-License-Identifier: EPL-2.0
//! Bounded immutable JSON preparation matching selected native migration helpers.
#![forbid(unsafe_code)]
use serde::de::{Deserialize, Deserializer, IgnoredAny, MapAccess, Visitor};
use serde_json::{Value, value::RawValue};
use std::{collections::HashSet, fmt, io::Write};
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
    /// Inspection only; use to_json to retain namespace ordering on re-entry.
    pub json: Value,
    namespace_order: Vec<String>,
    /// Selected participant version only; native migration history is external.
    pub participant_version: &'static str,
    pub matched_resource: bool,
}
struct Output(Vec<u8>);
impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.0.len().saturating_add(bytes.len()) > MAX_BYTES {
            return Err(std::io::Error::other("output byte budget"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Prepared {
    /// Serialize current values with original namespace insertion order. This
    /// is required when feeding another participant: Value sorts object keys.
    /// Duplicate namespace keys use their final value at first insertion place.
    pub fn to_json(&self) -> Result<String, Error> {
        bounded(&self.json, 0, &mut 0)?;
        let root = self
            .json
            .as_object()
            .ok_or_else(|| Error::Input("resource root must be an object".into()))?;
        let mut out = Output(Vec::new());
        fn byte(out: &mut Output, text: &[u8]) -> Result<(), Error> {
            out.write_all(text).map_err(|_| Error::Limit)
        }
        fn entry(
            out: &mut Output,
            first: &mut bool,
            key: &str,
            value: &Value,
        ) -> Result<(), Error> {
            if !*first {
                byte(out, b",")?;
            }
            *first = false;
            serde_json::to_writer(&mut *out, key).map_err(|_| Error::Limit)?;
            byte(out, b":")?;
            serde_json::to_writer(&mut *out, value).map_err(|_| Error::Limit)
        }
        byte(&mut out, b"{")?;
        let mut first = true;
        for (key, value) in root {
            if key == "ns" && value.is_object() {
                if !first {
                    byte(&mut out, b",")?;
                }
                first = false;
                byte(&mut out, b"\"ns\":{")?;
                let map = value.as_object().unwrap();
                let mut ns_first = true;
                let original: HashSet<&str> =
                    self.namespace_order.iter().map(String::as_str).collect();
                for key in &self.namespace_order {
                    if let Some(value) = map.get(key) {
                        entry(&mut out, &mut ns_first, key, value)?;
                    }
                }
                for (key, value) in map {
                    if !original.contains(key.as_str()) {
                        entry(&mut out, &mut ns_first, key, value)?;
                    }
                }
                byte(&mut out, b"}")?;
            } else {
                entry(&mut out, &mut first, key, value)?;
            }
        }
        byte(&mut out, b"}")?;
        String::from_utf8(out.0).map_err(|e| Error::Input(e.to_string()))
    }
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
struct First(Option<Value>, Vec<String>);
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
                let mut order = Vec::new();
                let mut seen = HashSet::new();
                while let Some(key) = map.next_key::<String>()? {
                    if seen.insert(key.clone()) {
                        order.push(key.clone());
                    }
                    if first_key.is_none() {
                        first_key = Some(key.clone());
                    }
                    if first_key.as_ref() == Some(&key) {
                        first = Some(map.next_value()?);
                    } else {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
                Ok(First(first, order))
            }
        }
        deserializer.deserialize_map(Entries)
    }
}
fn sysml(input: &str) -> Result<(bool, Vec<String>), Error> {
    let Namespace(ns) = serde_json::from_str(input).map_err(|e| Error::Input(e.to_string()))?;
    let Some(ns) = ns.filter(|v| v.get().trim_start().starts_with('{')) else {
        return Ok((false, Vec::new()));
    };
    let First(first, order) =
        serde_json::from_str(ns.get()).map_err(|e| Error::Input(e.to_string()))?;
    let matched = match first {
        None => false,
        Some(Value::String(s)) => s == SYSML_URI,
        Some(Value::Bool(_) | Value::Number(_)) => false,
        _ => return Err(Error::NamespaceShape),
    };
    Ok((matched, order))
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
    let (matched_resource, namespace_order) = sysml(input)?;
    if matched_resource {
        for pass in passes {
            let class = lookup(Package::Domain, pass.class)
                .filter(|id| id.metadata().kind == syson_metamodel::ClassifierKind::Class)
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
        namespace_order,
    })
}

#[path = "../b25b.rs"]
pub mod b25b;
