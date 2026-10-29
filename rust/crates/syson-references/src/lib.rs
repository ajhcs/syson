// SPDX-License-Identifier: EPL-2.0
//! Selected EMF-JSON reference lookup in immutable supplied resources, without loading.
#![forbid(unsafe_code)]
use serde_json::Value;
use std::{collections::BTreeMap, sync::Arc};
use syson_metamodel::{self as mm, ClassifierId};
use syson_model::Document;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FragmentIdentity {
    JsonNodeId,
    SemanticElementId,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decoded {
    Absent,
    LocalNode(String),
    External {
        resource: String,
        fragment: String,
        class: Option<ClassifierId>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid(&'static str),
    Unsupported(&'static str),
    Unresolved {
        resource: String,
        fragment: String,
    },
    TypeMismatch {
        expected: ClassifierId,
        actual: ClassifierId,
    },
    StaleSnapshot,
}
fn uuid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}
fn classifier(s: &str) -> Result<ClassifierId, Error> {
    s.strip_prefix("sysml:")
        .and_then(|s| mm::lookup(mm::Package::Domain, s))
        .filter(|c| c.metadata().kind == mm::ClassifierKind::Class)
        .ok_or(Error::Unsupported("unadmitted classifier"))
}
/// Case, UUID spelling and URI keys are preserved; this is not an EMF URI parser.
pub fn decode(value: Option<&Value>) -> Result<Decoded, Error> {
    let Some(value) = value.filter(|v| !v.is_null()) else {
        return Ok(Decoded::Absent);
    };
    let wire = value
        .as_str()
        .ok_or(Error::Unsupported("reference is not a string"))?;
    if wire.is_empty() || wire.chars().any(char::is_control) {
        return Err(Error::Invalid("empty or control-bearing reference"));
    }
    let (class, address) = if let Some((prefix, address)) = wire.split_once(' ') {
        if address.is_empty() || address.contains(char::is_whitespace) {
            return Err(Error::Invalid("invalid type/address separator"));
        }
        (Some(classifier(prefix)?), address)
    } else {
        (None, wire)
    };
    if address.contains('%') || address.contains(char::is_whitespace) {
        return Err(Error::Unsupported("encoded or whitespace URI family"));
    }
    if uuid(address) {
        return if class.is_none() {
            Ok(Decoded::LocalNode(address.into()))
        } else {
            Err(Error::Unsupported("typed local form"))
        };
    }
    let Some((resource, fragment)) = address.split_once('#') else {
        return Err(Error::Unsupported(
            "not a selected local or library reference",
        ));
    };
    if fragment.is_empty() || fragment.contains('#') {
        return Err(Error::Invalid("empty or multiple fragment"));
    }
    let path = resource
        .strip_prefix("sysmllibrary:///")
        .or_else(|| resource.strip_prefix("kermllibrary:///"))
        .ok_or(Error::Unsupported("unadmitted resource scheme"))?;
    if !uuid(path) {
        return Err(Error::Unsupported("non-UUID resource path"));
    }
    if fragment.contains('/') {
        return Err(Error::Unsupported("structural fragment"));
    }
    if !uuid(fragment) {
        return Err(Error::Invalid("non-UUID fragment"));
    }
    Ok(Decoded::External {
        resource: resource.into(),
        fragment: fragment.into(),
        class,
    })
}
/// An explicit caller key plus immutable document; keys never request a load.
pub struct Resource {
    pub key: String,
    pub document: Document,
    pub fragments: FragmentIdentity,
}
struct Entry {
    document: Document,
    nodes: BTreeMap<String, String>,
    fragments: FragmentIdentity,
}
/// Fields are private so only a producing registry can mint a valid target.
#[derive(Clone, Debug)]
pub struct Target {
    snapshot: Arc<()>,
    resource: String,
    node_id: String,
    element_id: String,
}
impl Target {
    pub fn resource(&self) -> &str {
        &self.resource
    }
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
    pub fn element_id(&self) -> &str {
        &self.element_id
    }
}
/// Owns all documents and has no mutation, replacement, loader or fallback API.
/// Callers supply resource keys and fragment policies; filenames and URI schemes
/// never select a policy implicitly. Document admission bounds each document,
/// not registry size or lookup latency; semantic lookup walks the document.
pub struct Registry {
    snapshot: Arc<()>,
    entries: BTreeMap<String, Entry>,
}
impl Registry {
    pub fn new(resources: Vec<Resource>) -> Result<Self, Error> {
        let mut entries = BTreeMap::new();
        for resource in resources {
            if resource.key.is_empty() || entries.contains_key(&resource.key) {
                return Err(Error::Invalid("empty or duplicate resource key"));
            }
            let nodes = resource
                .document
                .elements()
                .into_iter()
                .map(|e| (e.resource_id, e.element_id))
                .collect();
            entries.insert(
                resource.key,
                Entry {
                    document: resource.document,
                    nodes,
                    fragments: resource.fragments,
                },
            );
        }
        Ok(Self {
            snapshot: Arc::new(()),
            entries,
        })
    }
    /// Bare IDs use JSON-node identity in current_resource. External fragments
    /// use only their registered policy, even when another identity would match.
    pub fn resolve(
        &self,
        current_resource: &str,
        value: Option<&Value>,
        expected: ClassifierId,
    ) -> Result<Option<Target>, Error> {
        let (resource, fragment, policy, class) = match decode(value)? {
            Decoded::Absent => return Ok(None),
            Decoded::LocalNode(node) => (
                current_resource,
                node,
                Some(FragmentIdentity::JsonNodeId),
                None,
            ),
            Decoded::External {
                resource,
                fragment,
                class,
            } => {
                return self
                    .lookup(&resource, &fragment, None, class, expected)
                    .map(Some);
            }
        };
        self.lookup(resource, &fragment, policy, class, expected)
            .map(Some)
    }
    fn lookup(
        &self,
        resource: &str,
        fragment: &str,
        policy: Option<FragmentIdentity>,
        class: Option<ClassifierId>,
        expected: ClassifierId,
    ) -> Result<Target, Error> {
        let missing = || Error::Unresolved {
            resource: resource.into(),
            fragment: fragment.into(),
        };
        let entry = self.entries.get(resource).ok_or_else(missing)?;
        let element = match policy.unwrap_or(entry.fragments) {
            FragmentIdentity::JsonNodeId => entry.nodes.get(fragment).ok_or_else(missing)?.as_str(),
            FragmentIdentity::SemanticElementId => fragment,
        };
        let node = entry.document.element(element).ok_or_else(missing)?;
        let actual = classifier(node["eClass"].as_str().unwrap())?;
        for required in class.into_iter().chain([expected]) {
            if required.metadata().kind != mm::ClassifierKind::Class {
                return Err(Error::Unsupported("expected classifier is not a class"));
            }
            if !mm::is_subtype(actual, required) {
                return Err(Error::TypeMismatch {
                    expected: required,
                    actual,
                });
            }
        }
        Ok(Target {
            snapshot: self.snapshot.clone(),
            resource: resource.into(),
            node_id: node["id"].as_str().unwrap().into(),
            element_id: element.into(),
        })
    }
    /// Revalidate before consuming a scoped result. Byte-identical registries
    /// still represent distinct immutable snapshots and reject each other's targets.
    pub fn target(&self, target: &Target) -> Result<&Value, Error> {
        if !Arc::ptr_eq(&self.snapshot, &target.snapshot) {
            return Err(Error::StaleSnapshot);
        }
        self.entries
            .get(&target.resource)
            .and_then(|e| e.document.element(&target.element_id))
            .ok_or(Error::StaleSnapshot)
    }
}
