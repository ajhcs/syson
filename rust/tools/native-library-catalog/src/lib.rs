// SPDX-License-Identifier: EPL-2.0
//! Physical library JSON catalogue, with explicit, separate identity domains.
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
    fmt,
};
pub const MAX_RESOURCES: usize = 128;
pub const MAX_FILE: usize = 4 * 1024 * 1024;
pub const MAX_TOTAL: usize = 32 * 1024 * 1024;
pub const MAX_DEPTH: usize = 256;
pub const MAX_NODES: usize = 200_000;
pub const MAX_INDEX_BYTES: usize = 128 * 1024 * 1024;
pub const MAX_WALK_BYTES: usize = 256 * 1024 * 1024;
pub type Result<T> = std::result::Result<T, String>;
pub struct LibrarySource<'a> {
    pub classpath_path: &'a str,
    pub resource_uri: &'a str,
    pub json: &'a str,
}
pub struct NodeId<'a>(pub &'a str);
pub struct SemanticId<'a>(pub &'a str);
pub struct Node {
    pub node_id: String,
    pub semantic_id: String,
    pub e_class: String,
    pub declared_name: Option<String>,
    pub pointer: String,
}
pub struct Resource {
    pub classpath_path: String,
    pub resource_uri: String,
    json: String,
    nodes: Vec<Node>,
    node_ids: BTreeMap<String, usize>,
    semantic_ids: BTreeMap<String, usize>,
}
pub struct Catalog {
    resources: Vec<Resource>,
    uris: BTreeMap<String, usize>,
}
fn need(ok: bool, why: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(why.into()) }
}
fn bounded(s: &str) -> bool {
    !s.is_empty() && s.len() <= 1024 && !s.chars().any(char::is_control)
}
fn uuid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
            }
        })
}
fn depth(raw: &str) -> Result<()> {
    let (mut n, mut quoted, mut escaped) = (0usize, false, false);
    for b in raw.bytes() {
        if quoted {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                quoted = false;
            }
        } else {
            match b {
                b'"' => quoted = true,
                b'{' | b'[' => {
                    n += 1;
                    need(n <= MAX_DEPTH, "JSON depth limit")?;
                }
                b'}' | b']' => n = n.saturating_sub(1),
                _ => {}
            }
        }
    }
    Ok(())
}
#[derive(Clone, Copy)]
struct Check<'a>(&'a Cell<usize>);
impl<'de> DeserializeSeed<'de> for Check<'_> {
    type Value = ();
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> std::result::Result<(), D::Error> {
        self.0.set(self.0.get() + 1);
        if self.0.get() > 2_000_000 {
            return Err(serde::de::Error::custom("JSON value limit"));
        }
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Check<'_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("unique-key JSON")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> std::result::Result<(), A::Error> {
        let mut keys = BTreeSet::new();
        while let Some(k) = a.next_key::<String>()? {
            if !keys.insert(k) {
                return Err(serde::de::Error::custom("duplicate JSON key"));
            }
            a.next_value_seed(self)?;
        }
        Ok(())
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> std::result::Result<(), A::Error> {
        while a.next_element_seed(self)?.is_some() {}
        Ok(())
    }
    fn visit_str<E: serde::de::Error>(self, _: &str) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_bool<E: serde::de::Error>(self, _: bool) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: serde::de::Error>(self, _: i64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: serde::de::Error>(self, _: u64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: serde::de::Error>(self, _: f64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<(), E> {
        Ok(())
    }
}
impl Resource {
    pub fn json(&self) -> &str {
        &self.json
    }
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }
}
impl Catalog {
    pub fn resources(&self) -> &[Resource] {
        &self.resources
    }
    pub fn resource(&self, uri: &str) -> Option<&Resource> {
        self.uris.get(uri).map(|i| &self.resources[*i])
    }
    pub fn by_node_id(&self, uri: &str, id: NodeId<'_>) -> Option<&Node> {
        let r = self.resource(uri)?;
        r.node_ids.get(id.0).map(|i| &r.nodes[*i])
    }
    pub fn by_semantic_id(&self, uri: &str, id: SemanticId<'_>) -> Option<&Node> {
        let r = self.resource(uri)?;
        r.semantic_ids.get(id.0).map(|i| &r.nodes[*i])
    }
    /// Physical index only; explicit URI bindings imply no authenticity or EMF admission.
    pub fn from_resources(inputs: &[LibrarySource<'_>]) -> Result<Self> {
        need(inputs.len() <= MAX_RESOURCES, "resource count limit")?;
        let (mut paths, mut uris, mut total) = (BTreeSet::new(), BTreeSet::new(), 0usize);
        for s in inputs {
            let scheme = if s.classpath_path.starts_with("kerml.libraries/") {
                "kermllibrary:///"
            } else if s.classpath_path.starts_with("sysml.libraries/") {
                "sysmllibrary:///"
            } else {
                return Err("unknown library family".into());
            };
            let file = s.classpath_path.split_once('/').unwrap().1;
            need(
                bounded(s.classpath_path)
                    && file.ends_with(".json")
                    && !file.contains(['/', '\\', ':'])
                    && file != ".json",
                "classpath path",
            )?;
            need(
                s.resource_uri.strip_prefix(scheme).is_some_and(uuid),
                "resource URI/family",
            )?;
            need(
                paths.insert(s.classpath_path) && uris.insert(s.resource_uri),
                "duplicate resource path/URI",
            )?;
            need(s.json.len() <= MAX_FILE, "resource byte limit")?;
            total += s.json.len();
            need(total <= MAX_TOTAL, "aggregate byte limit")?;
            depth(s.json)?;
        }
        let mut result = Self {
            resources: Vec::new(),
            uris: BTreeMap::new(),
        };
        let count = Cell::new(0);
        let mut node_count = 0;
        let (mut index_bytes, mut walk_bytes) = (0usize, 0usize);
        for s in inputs {
            let v = value(s.json, &count)?;
            need(
                v["ns"]["sysml"] == "http://www.eclipse.org/syson/sysml" && v["content"].is_array(),
                "library document shape",
            )?;
            let mut r = Resource {
                classpath_path: s.classpath_path.into(),
                resource_uri: s.resource_uri.into(),
                json: s.json.into(),
                nodes: Vec::new(),
                node_ids: BTreeMap::new(),
                semantic_ids: BTreeMap::new(),
            };
            let mut stack = vec![(&v, String::new())];
            while let Some((v, pointer)) = stack.pop() {
                match v {
                    Value::Object(m) => {
                        if m.contains_key("eClass") {
                            node_count += 1;
                            need(node_count <= MAX_NODES, "node count limit")?;
                            let id = m
                                .get("id")
                                .and_then(Value::as_str)
                                .ok_or("node ID required")?;
                            let class = m["eClass"].as_str().ok_or("eClass required")?;
                            let semantic = v["data"]["elementId"]
                                .as_str()
                                .ok_or("semantic ID required")?;
                            need(
                                uuid(id)
                                    && uuid(semantic)
                                    && bounded(class)
                                    && class.starts_with("sysml:"),
                                "node identity/class shape",
                            )?;
                            let name = match &v["data"]["declaredName"] {
                                Value::Null => None,
                                Value::String(n) if n.len() <= 4096 => Some(n.clone()),
                                _ => return Err("declaredName shape/limit".into()),
                            };
                            index_bytes += id.len() * 2
                                + semantic.len() * 2
                                + class.len()
                                + name.as_ref().map_or(0, String::len)
                                + pointer.len();
                            need(index_bytes <= MAX_INDEX_BYTES, "indexed string byte limit")?;
                            let i = r.nodes.len();
                            need(
                                r.node_ids.insert(id.into(), i).is_none()
                                    && r.semantic_ids.insert(semantic.into(), i).is_none(),
                                "duplicate node/semantic identity",
                            )?;
                            r.nodes.push(Node {
                                node_id: id.into(),
                                semantic_id: semantic.into(),
                                e_class: class.into(),
                                declared_name: name,
                                pointer: pointer.clone(),
                            });
                        }
                        for (key, value) in m {
                            if value.is_object() || value.is_array() {
                                let segment = key.replace('~', "~0").replace('/', "~1");
                                walk_bytes += pointer.len() + 1 + segment.len();
                                need(
                                    walk_bytes <= MAX_WALK_BYTES
                                        && pointer.len() + 1 + segment.len() <= 32768,
                                    "walk string byte/pointer limit",
                                )?;
                                stack.push((value, format!("{pointer}/{segment}")));
                            }
                        }
                    }
                    Value::Array(a) => {
                        for (i, value) in a.iter().enumerate() {
                            if value.is_object() || value.is_array() {
                                let segment = i.to_string();
                                walk_bytes += pointer.len() + 1 + segment.len();
                                need(
                                    walk_bytes <= MAX_WALK_BYTES
                                        && pointer.len() + 1 + segment.len() <= 32768,
                                    "walk string byte/pointer limit",
                                )?;
                                stack.push((value, format!("{pointer}/{segment}")));
                            }
                        }
                    }
                    _ => {}
                }
            }
            result
                .uris
                .insert(r.resource_uri.clone(), result.resources.len());
            result.resources.push(r);
        }
        Ok(result)
    }
}

fn value(raw: &str, count: &Cell<usize>) -> Result<Value> {
    depth(raw)?;
    let mut d = serde_json::Deserializer::from_str(raw);
    d.disable_recursion_limit();
    Check(count)
        .deserialize(&mut d)
        .map_err(|e| e.to_string())?;
    d.end().map_err(|e| e.to_string())?;
    let mut d = serde_json::Deserializer::from_str(raw);
    d.disable_recursion_limit();
    <Value as serde::Deserialize>::deserialize(&mut d).map_err(|e| e.to_string())
}
pub struct ManifestEntry {
    pub classpath_path: String,
    pub resource_uri: String,
    pub file: String,
}
/// Strict local manifest only; resource URI bindings remain caller-supplied facts.
pub fn manifest(raw: &str) -> Result<Vec<ManifestEntry>> {
    need(raw.len() <= 256 * 1024, "manifest byte limit")?;
    let v = value(raw, &Cell::new(0))?;
    let a = v.as_array().ok_or("manifest array required")?;
    need(a.len() <= MAX_RESOURCES, "resource count limit")?;
    a.iter()
        .map(|row| {
            let m = row.as_object().ok_or("manifest row object")?;
            need(
                m.len() == 3
                    && ["classpath_path", "resource_uri", "file"]
                        .iter()
                        .all(|k| m.contains_key(*k)),
                "manifest fields",
            )?;
            let get = |key| {
                row[key]
                    .as_str()
                    .filter(|s| bounded(s))
                    .map(String::from)
                    .ok_or_else(|| "manifest string required".to_string())
            };
            Ok(ManifestEntry {
                classpath_path: get("classpath_path")?,
                resource_uri: get("resource_uri")?,
                file: get("file")?,
            })
        })
        .collect()
}
