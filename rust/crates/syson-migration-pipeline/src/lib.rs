// SPDX-License-Identifier: EPL-2.0
//! Bounded, pure preparation of two pinned library-ID migrations.
//! Not native migration-history compatibility or a durable transaction. The
//! caller must commit all output documents, representations and watermarks
//! together. No resource loading or subprocess fallback is performed here.
#![forbid(unsafe_code)]
use serde_json::Value;
use std::{collections::HashMap, io::Write};
use syson_metamodel::{FeatureKind, Package, inherited_feature, lookup};
use syson_model::Document;
use syson_persistence::{
    DIAGRAM_KIND, DIAGRAM_VERSION, REFERENCES_VERSION, ResolvedElement, migrate_diagram_target,
    migrate_reference_uri,
};

const DOCUMENT_BYTES: usize = 1024 * 1024;
const PROJECT_BYTES: usize = 8 * DOCUMENT_BYTES;
const ITEMS: usize = 64;
const DEPTH: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticDocument {
    pub document: Document,
    /// None means this selected participant has not run. The sole accepted
    /// nonempty value is REFERENCES_VERSION, not a native global version.
    pub references_version: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Representation {
    pub kind: String,
    pub content: Value,
    /// Selected participant watermark; unrelated representation kinds retain it.
    pub diagram_version: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Project {
    pub documents: Vec<SemanticDocument>,
    pub representations: Vec<Representation>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Limit,
    UnsupportedVersion(String),
    Unclassified(String),
    ReferenceShape(String),
    InvalidDocument(String),
    Resolver(String),
}
/// The caller supplies a no-load snapshot resolver. Some is an actual SysML
/// Element; its resource URI distinguishes a library hit from a nonlibrary hit.
/// None is unresolved/non-Element. Errors abort preparation without output.
/// Resolver side effects, if any, cannot be rolled back by this pure API.
pub trait Resolver {
    fn resolve(&mut self, id_or_uri: &str) -> Result<Option<ResolvedElement>, String>;
}
struct Counter(usize);
impl Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .filter(|n| *n <= DOCUMENT_BYTES)
            .ok_or_else(|| std::io::Error::other("document byte budget"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn depth(value: &Value, level: usize) -> Result<(), Error> {
    if level > DEPTH {
        return Err(Error::Limit);
    }
    match value {
        Value::Object(o) => {
            for v in o.values() {
                depth(v, level + 1)?;
            }
        }
        Value::Array(a) => {
            for v in a {
                depth(v, level + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn size(value: &Value) -> Result<usize, Error> {
    let mut counter = Counter(0);
    serde_json::to_writer(&mut counter, value).map_err(|_| Error::Limit)?;
    Ok(counter.0)
}
fn budget(project: &Project) -> Result<usize, Error> {
    if project
        .documents
        .len()
        .saturating_add(project.representations.len())
        > ITEMS
    {
        return Err(Error::Limit);
    }
    if project.documents.iter().any(|d| {
        d.references_version
            .as_ref()
            .is_some_and(|v| v.len() > 1024)
    }) || project
        .representations
        .iter()
        .any(|r| r.kind.len() > 1024 || r.diagram_version.as_ref().is_some_and(|v| v.len() > 1024))
    {
        return Err(Error::Limit);
    }
    let mut total = 0usize;
    for value in project
        .documents
        .iter()
        .map(|d| d.document.value())
        .chain(project.representations.iter().map(|r| &r.content))
    {
        depth(value, 0)?;
        total = total
            .checked_add(size(value)?)
            .filter(|n| *n <= PROJECT_BYTES)
            .ok_or(Error::Limit)?;
    }
    Ok(total)
}
fn version(current: &Option<String>, expected: &str) -> Result<bool, Error> {
    match current.as_deref() {
        None => Ok(true),
        Some(v) if v == expected => Ok(false),
        Some(v) => Err(Error::UnsupportedVersion(v.into())),
    }
}
// Count escaped JSON bytes before cloning/formatting a replacement. Growth is
// cumulative (shrinks do not refund it), bounded per document and whole project.
struct Growth<'a> {
    document: usize,
    project: &'a mut usize,
}
impl Growth<'_> {
    fn reserve(&mut self, old: &str, prefix: &str, id: &str) -> Result<(), Error> {
        let encoded = |s: &str| -> Result<usize, Error> {
            let mut counter = Counter(0);
            serde_json::to_writer(&mut counter, s).map_err(|_| Error::Limit)?;
            Ok(counter.0)
        };
        let new = encoded(prefix)?
            .checked_add(encoded(id)?)
            .and_then(|n| n.checked_sub(2))
            .ok_or(Error::Limit)?;
        let growth = new.saturating_sub(encoded(old)?);
        if growth > self.document || growth > *self.project {
            return Err(Error::Limit);
        }
        self.document -= growth;
        *self.project -= growth;
        Ok(())
    }
}
fn reference(
    value: &mut Value,
    field: &str,
    ids: &HashMap<String, String>,
    resolver: &mut impl Resolver,
    growth: &mut Growth<'_>,
) -> Result<(), Error> {
    match value {
        Value::Null => Ok(()),
        Value::String(uri) => {
            if !(uri.contains("sysmllibrary") || uri.contains("kermllibrary")) {
                return Ok(());
            }
            // Same no-load lookup normalization as the pinned pure participant.
            let query = if uri.starts_with("sysml:") {
                &uri[uri
                    .find("sysmllibrary")
                    .or_else(|| uri.find("kermllibrary"))
                    .unwrap()..]
            } else {
                uri.as_str()
            };
            let mut resolved = resolver.resolve(query).map_err(Error::Resolver)?;
            let (prefix, old_id) = uri
                .find('#')
                .map_or(("", uri.as_str()), |i| (&uri[..=i], &uri[i + 1..]));
            if let Some(id) = resolved
                .as_ref()
                .map(|e| &e.element_id)
                .or_else(|| ids.get(old_id))
            {
                growth.reserve(uri, prefix, id)?;
                *uri = migrate_reference_uri(uri, false, ids, |_| {
                    resolved.take().map(|e| e.element_id)
                });
            }
            Ok(())
        }
        Value::Array(items) => {
            for item in items {
                if !item.is_string() && !item.is_null() {
                    return Err(Error::ReferenceShape(field.into()));
                }
                reference(item, field, ids, resolver, growth)?;
            }
            Ok(())
        }
        _ => Err(Error::ReferenceShape(field.into())),
    }
}
fn has_library_uri(value: &Value) -> bool {
    match value {
        Value::String(s) => s.contains("sysmllibrary") || s.contains("kermllibrary"),
        Value::Array(a) => a.iter().any(has_library_uri),
        Value::Object(o) => o.values().any(has_library_uri),
        _ => false,
    }
}
fn semantic(
    value: &mut Value,
    ids: &HashMap<String, String>,
    resolver: &mut impl Resolver,
    growth: &mut Growth<'_>,
) -> Result<(), Error> {
    match value {
        Value::Object(object) => {
            if let Some(class) = object.get("eClass") {
                let classifier = class
                    .as_str()
                    .and_then(|s| s.strip_prefix("sysml:"))
                    .and_then(|s| lookup(Package::Domain, s))
                    .ok_or_else(|| Error::Unclassified(class.to_string()))?;
                let data = object
                    .get_mut("data")
                    .and_then(Value::as_object_mut)
                    .ok_or_else(|| {
                        Error::InvalidDocument("EObject data is not an object".into())
                    })?;
                for (name, value) in data {
                    let feature = inherited_feature(classifier, name);
                    if let Some(f) = feature
                        .unique()
                        .filter(|_| feature.unresolved_supertypes.is_empty())
                    {
                        if f.kind() == FeatureKind::Reference && !f.containment() {
                            reference(value, name, ids, resolver, growth)?;
                        }
                    } else if has_library_uri(value) {
                        return Err(Error::Unclassified(name.clone()));
                    }
                }
            }
            for child in object.values_mut() {
                semantic(child, ids, resolver, growth)?;
            }
        }
        Value::Array(items) => {
            for child in items {
                semantic(child, ids, resolver, growth)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn diagram(
    value: &mut Value,
    ids: &HashMap<String, String>,
    resolver: &mut impl Resolver,
    growth: &mut Growth<'_>,
) -> Result<(), Error> {
    match value {
        Value::Object(object) => {
            if let Some(old) = object.get("targetObjectId").and_then(Value::as_str) {
                let mut resolved = resolver.resolve(old).map_err(Error::Resolver)?;
                let replacement = match resolved.as_ref() {
                    Some(e)
                        if syson_persistence::is_standard_library_resource(
                            e.resource_uri.as_deref(),
                        ) =>
                    {
                        Some(&e.element_id)
                    }
                    Some(_) => None,
                    None => ids.get(old),
                };
                if let Some(id) = replacement {
                    growth.reserve(old, "", id)?;
                    if let Some(new) =
                        migrate_diagram_target("targetObjectId", Some(old), ids, |_| {
                            resolved.take()
                        })
                    {
                        object.insert("targetObjectId".into(), Value::String(new));
                    }
                }
            }
            for child in object.values_mut() {
                diagram(child, ids, resolver, growth)?;
            }
        }
        Value::Array(items) => {
            for child in items {
                diagram(child, ids, resolver, growth)?;
            }
        }
        _ => {}
    }
    Ok(())
}
/// Apply semantic participant 1600 across all documents, then representation
/// participant 1700 across matching diagrams. Already stamped inputs are skipped,
/// even when a map's new ID is another old ID. Unknown participant watermarks
/// refuse; they are not interpreted as native application migration history.
/// Inputs stay untouched on any error, including output admission/budget errors.
pub fn prepare(
    input: &Project,
    ids: &HashMap<String, String>,
    resolver: &mut impl Resolver,
) -> Result<Project, Error> {
    let mut project_remaining = PROJECT_BYTES - budget(input)?;
    for doc in &input.documents {
        version(&doc.references_version, REFERENCES_VERSION)?;
    }
    for rep in &input.representations {
        if rep.kind == DIAGRAM_KIND {
            version(&rep.diagram_version, DIAGRAM_VERSION)?;
        }
    }
    let mut output = input.clone();
    for doc in &mut output.documents {
        if version(&doc.references_version, REFERENCES_VERSION)? {
            let mut value = doc.document.value().clone();
            let mut growth = Growth {
                document: DOCUMENT_BYTES - size(&value)?,
                project: &mut project_remaining,
            };
            semantic(&mut value, ids, resolver, &mut growth)?;
            let bytes =
                serde_json::to_string(&value).map_err(|e| Error::InvalidDocument(e.to_string()))?;
            doc.document =
                Document::from_json(&bytes).map_err(|e| Error::InvalidDocument(e.to_string()))?;
            doc.references_version = Some(REFERENCES_VERSION.into());
        }
    }
    for rep in &mut output.representations {
        if rep.kind == DIAGRAM_KIND && version(&rep.diagram_version, DIAGRAM_VERSION)? {
            let mut growth = Growth {
                document: DOCUMENT_BYTES - size(&rep.content)?,
                project: &mut project_remaining,
            };
            diagram(&mut rep.content, ids, resolver, &mut growth)?;
            rep.diagram_version = Some(DIAGRAM_VERSION.into());
        }
    }
    budget(&output)?;
    Ok(output)
}
