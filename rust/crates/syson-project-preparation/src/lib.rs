// SPDX-License-Identifier: EPL-2.0
//! Consuming, private selected-project preparation; no SQL or native history host.
#![forbid(unsafe_code)]
use std::collections::HashMap;

use serde_json::Value;
use syson_migration_pipeline::{self as library, Resolver};
use syson_migrations::{self as migrations, b25a, b25b, b25c, b25d};
use syson_model::Document;
use syson_persistence::{DIAGRAM_KIND, DIAGRAM_VERSION, REFERENCES_VERSION};

pub const MAX_ITEMS: usize = 64;
pub const MAX_PROJECT_BYTES: usize = 8 * migrations::MAX_BYTES;
const MAX_METADATA: usize = 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SemanticVersions {
    pub connector: Option<String>,
    pub annotation: Option<String>,
    pub view: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticInput {
    pub json: String,
    pub versions: SemanticVersions,
    pub references_version: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepresentationInput {
    pub kind: String,
    pub json: String,
    pub control_version: Option<String>,
    pub diagram_version: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectInput {
    pub documents: Vec<SemanticInput>,
    pub representations: Vec<RepresentationInput>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Limit,
    Version {
        participant: &'static str,
        found: String,
    },
    Semantic {
        index: usize,
        cause: migrations::Error,
    },
    Representation {
        index: usize,
        cause: b25d::Error,
    },
    Admission {
        kind: &'static str,
        index: usize,
        detail: String,
    },
    Library(library::Error),
}
/// Matching wire and typed snapshots are private until complete preparation.
/// Publish both snapshots and all their independent watermarks atomically in a
/// later persistence layer. No durable transaction is implemented by this type.
#[derive(Debug, PartialEq, Eq)]
pub struct PreparedProject {
    wire: ProjectInput,
    project: library::Project,
}
impl PreparedProject {
    pub fn wire(&self) -> &ProjectInput {
        &self.wire
    }
    pub fn project(&self) -> &library::Project {
        &self.project
    }
    pub fn into_parts(self) -> (ProjectInput, library::Project) {
        (self.wire, self.project)
    }
}
fn metadata(value: &Option<String>) -> Result<(), Error> {
    if value.as_ref().is_some_and(|v| v.len() > MAX_METADATA) {
        Err(Error::Limit)
    } else {
        Ok(())
    }
}
fn version(value: &Option<String>, expected: &'static str) -> Result<(), Error> {
    metadata(value)?;
    if let Some(found) = value.as_ref().filter(|v| v.as_str() != expected) {
        return Err(Error::Version {
            participant: expected,
            found: found.clone(),
        });
    }
    Ok(())
}
fn value_budget(value: &Value, depth: usize, count: &mut usize) -> Result<(), Error> {
    *count += 1;
    if depth > migrations::MAX_DEPTH || *count > migrations::MAX_VALUES {
        return Err(Error::Limit);
    }
    match value {
        Value::Object(o) => {
            for v in o.values() {
                value_budget(v, depth + 1, count)?;
            }
        }
        Value::Array(a) => {
            for v in a {
                value_budget(v, depth + 1, count)?;
            }
        }
        _ => (),
    }
    Ok(())
}
fn parse(text: &str, kind: &'static str, index: usize) -> Result<Value, Error> {
    let value = serde_json::from_str(text).map_err(|e| Error::Admission {
        kind,
        index,
        detail: format!("JSON input: {e}"),
    })?;
    value_budget(&value, 0, &mut 0)?;
    Ok(value)
}
fn preflight(input: &ProjectInput, ids: &HashMap<String, String>) -> Result<(), Error> {
    if input
        .documents
        .len()
        .saturating_add(input.representations.len())
        > MAX_ITEMS
        || ids.len() > migrations::MAX_VALUES
    {
        return Err(Error::Limit);
    }
    let mut size = 0usize;
    for (kind, index, text) in input
        .documents
        .iter()
        .enumerate()
        .map(|(i, d)| ("document", i, &d.json))
        .chain(
            input
                .representations
                .iter()
                .enumerate()
                .map(|(i, r)| ("representation", i, &r.json)),
        )
    {
        if text.len() > migrations::MAX_BYTES {
            return Err(Error::Limit);
        }
        size = size
            .checked_add(text.len())
            .filter(|n| *n <= MAX_PROJECT_BYTES)
            .ok_or(Error::Limit)?;
        parse(text, kind, index)?;
    }
    let mut map_size = 0usize;
    for (old, new) in ids {
        if old.len() > migrations::MAX_BYTES || new.len() > migrations::MAX_BYTES {
            return Err(Error::Limit);
        }
        map_size = map_size
            .checked_add(old.len())
            .and_then(|n| n.checked_add(new.len()))
            .filter(|n| *n <= MAX_PROJECT_BYTES)
            .ok_or(Error::Limit)?;
    }
    for doc in &input.documents {
        version(&doc.versions.connector, b25b::VERSION)?;
        version(&doc.versions.annotation, b25a::VERSION)?;
        version(&doc.versions.view, b25c::VERSION)?;
        version(&doc.references_version, REFERENCES_VERSION)?;
    }
    for rep in &input.representations {
        if rep.kind.len() > MAX_METADATA {
            return Err(Error::Limit);
        }
        metadata(&rep.control_version)?;
        metadata(&rep.diagram_version)?;
        if rep.kind == DIAGRAM_KIND {
            version(&rep.diagram_version, DIAGRAM_VERSION)?;
        }
        if rep.kind == b25d::KIND {
            version(&rep.control_version, b25d::VERSION)?;
        }
    }
    Ok(())
}
fn semantic(doc: &mut SemanticInput, index: usize) -> Result<migrations::Prepared, Error> {
    let error = |cause| Error::Semantic { index, cause };
    // A zero-removal call captures bounded namespace transport only. Its version
    // field is never copied into a selected participant watermark.
    let mut carrier = migrations::prepare(&doc.json, b25c::VERSION, &[]).map_err(error)?;
    type Pass = fn(&str) -> Result<migrations::Prepared, migrations::Error>;
    for (stamp, expected, run) in [
        (
            &mut doc.versions.connector,
            b25b::VERSION,
            b25b::prepare as Pass,
        ),
        (
            &mut doc.versions.annotation,
            b25a::VERSION,
            b25a::prepare as Pass,
        ),
        (&mut doc.versions.view, b25c::VERSION, b25c::prepare as Pass),
    ] {
        if stamp.is_none() {
            carrier = run(&carrier.to_json().map_err(error)?).map_err(error)?;
            if carrier.matched_resource {
                *stamp = Some(expected.into());
            }
        }
    }
    Ok(carrier)
}
/// Execute the explicit local recipe: b25b -> b25a -> b25c -> Document admission
/// -> B01 semantic/diagram library IDs -> b25d. Selected watermarks are independent
/// and do not encode native framework activation/history. Semantic stamps must
/// be absent or exact; unmatched semantic resource predicates leave them pending.
/// Foreign representation kinds retain both supplied representation watermarks.
///
/// Bounds and selected metadata are checked before cloning/resolver work. Every
/// failure drops private staging; the input and a previously published output
/// stay untouched. Resolver effects remain caller-owned: use a read-only/no-load
/// snapshot. This API cannot roll back an arbitrary resolver's external effects.
pub fn prepare(
    input: &ProjectInput,
    ids: &HashMap<String, String>,
    resolver: &mut impl Resolver,
) -> Result<PreparedProject, Error> {
    preflight(input, ids)?;
    let mut wire = input.clone();
    let mut carriers = Vec::with_capacity(wire.documents.len());
    let mut project = library::Project::default();
    for (index, doc) in wire.documents.iter_mut().enumerate() {
        let carrier = semantic(doc, index)?;
        let text = carrier
            .to_json()
            .map_err(|cause| Error::Semantic { index, cause })?;
        let document = Document::from_json(&text).map_err(|e| Error::Admission {
            kind: "document",
            index,
            detail: e.to_string(),
        })?;
        project.documents.push(library::SemanticDocument {
            document,
            references_version: doc.references_version.clone(),
        });
        carriers.push(carrier);
    }
    for (index, rep) in wire.representations.iter().enumerate() {
        let content = parse(&rep.json, "representation", index)?;
        if rep.kind == b25d::KIND && !content.is_object() {
            return Err(Error::Representation {
                index,
                cause: b25d::Error::Input("diagram root must be an object".into()),
            });
        }
        project.representations.push(library::Representation {
            kind: rep.kind.clone(),
            content,
            diagram_version: rep.diagram_version.clone(),
        });
    }
    // C03b preflight above validates every B01 watermark before resolver work.
    // B01 then handles semantic1600 and diagram1700 via its no-load API.
    let mut project = library::prepare(&project, ids, resolver).map_err(Error::Library)?;
    for (index, ((doc, carrier), typed)) in wire
        .documents
        .iter_mut()
        .zip(carriers.iter_mut())
        .zip(&project.documents)
        .enumerate()
    {
        carrier.json = typed.document.value().clone();
        doc.json = carrier
            .to_json()
            .map_err(|cause| Error::Semantic { index, cause })?;
        doc.references_version = typed.references_version.clone();
    }
    for (index, (rep, typed)) in wire
        .representations
        .iter_mut()
        .zip(&mut project.representations)
        .enumerate()
    {
        let text = serde_json::to_string(&typed.content).map_err(|e| Error::Admission {
            kind: "representation",
            index,
            detail: e.to_string(),
        })?;
        let prepared = b25d::prepare(&rep.kind, &text, rep.control_version.as_deref())
            .map_err(|cause| Error::Representation { index, cause })?;
        rep.json = prepared
            .to_json()
            .map_err(|cause| Error::Representation { index, cause })?;
        rep.control_version = prepared.participant_version;
        typed.content = prepared.json;
        rep.diagram_version = typed.diagram_version.clone();
    }
    preflight(&wire, ids)?;
    Ok(PreparedProject { wire, project })
}
