// Copyright (c) 2024, 2026 Obeo.
// This program and accompanying materials are made available under the terms
// of the Eclipse Public License v2.0, https://www.eclipse.org/legal/epl-2.0/.
// SPDX-License-Identifier: EPL-2.0
// Contributors: Obeo - initial API and implementation
// Rust adapter design and translation: SysON Rust port contributors.

//! EMF JSON value-preserving adapter and proposed local edit transactions.
//! Not a complete EMF implementation, database, REST API, or SysML semantic engine.
#![forbid(unsafe_code)]
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use syson_domain::model::{Element, ElementId, Model};
mod containment;
mod scalar;
pub use containment::ContainmentFeature;
pub use scalar::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    Attribute(AttributeError),
    Json(String),
    JsonAdmission(syson_json_value::Error),
    JsonOutputIo {
        kind: std::io::ErrorKind,
        written: usize,
    },
    Shape(String),
    InvalidId(String),
    DuplicateId(String),
    MissingElement(String),
    UnsupportedClass(String),
    InvalidName,
    WrongType {
        element: String,
        expected: &'static str,
    },
    StaleTransaction,
    FailedTransaction,
    ForeignTransaction,
    MissingMembership,
    UnexpectedMembership,
}
impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ModelError {}
type Result<T> = std::result::Result<T, ModelError>;
const JSON_BYTES_LIMIT: usize = 4 * 1024 * 1024;
const JSON_DEPTH_LIMIT: usize = 128;

fn json_cancelled(cancelled: &mut impl FnMut() -> bool) -> Result<()> {
    if cancelled() {
        Err(ModelError::JsonAdmission(
            syson_json_value::Error::Cancelled,
        ))
    } else {
        Ok(())
    }
}
struct JsonWriter<'a, W: ?Sized, F> {
    target: &'a mut W,
    cancelled: &'a mut F,
    limit: usize,
    written: usize,
    failure: Option<ModelError>,
}
impl<W: std::io::Write + ?Sized, F: FnMut() -> bool> std::io::Write for JsonWriter<'_, W, F> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.failure.is_some() {
            return Err(std::io::Error::other("JSON writer already refused"));
        }
        if let Err(error) = json_cancelled(self.cancelled) {
            self.failure = Some(error);
            return Err(std::io::Error::other("JSON output cancelled"));
        }
        if bytes.len() > self.limit.saturating_sub(self.written) {
            self.failure = Some(ModelError::JsonAdmission(syson_json_value::Error::Limit(
                syson_json_value::Limit::Bytes,
            )));
            return Err(std::io::Error::other("JSON output byte limit"));
        }
        match self.target.write(bytes) {
            Ok(count) if count <= bytes.len() && (count != 0 || bytes.is_empty()) => {
                self.written += count;
                Ok(count)
            }
            Ok(count) => {
                let kind = if count == 0 {
                    std::io::ErrorKind::WriteZero
                } else {
                    std::io::ErrorKind::InvalidData
                };
                self.failure = Some(ModelError::JsonOutputIo {
                    kind,
                    written: self.written,
                });
                Err(std::io::Error::new(kind, "invalid JSON sink write count"))
            }
            // serde's write_all retries Interrupted. The retry re-enters this
            // wrapper and polls cancellation again instead of hiding the loop.
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => Err(error),
            Err(error) => {
                self.failure = Some(ModelError::JsonOutputIo {
                    kind: error.kind(),
                    written: self.written,
                });
                Err(error)
            }
        }
    }
    // This API never flushes the caller's writer or promises durability.
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Separate UUIDs: EMF resource identity must not overwrite SysML elementId.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeIds {
    pub resource: String,
    pub element: String,
}
impl NodeIds {
    pub fn new(resource: &str, element: &str) -> Result<Self> {
        for id in [resource, element] {
            if id.len() != 36
                || !id.bytes().enumerate().all(|(i, b)| {
                    if [8, 13, 18, 23].contains(&i) {
                        b == b'-'
                    } else {
                        b.is_ascii_hexdigit()
                    }
                })
            {
                return Err(ModelError::InvalidId(id.to_owned()));
            }
        }
        Ok(Self {
            resource: resource.to_owned(),
            element: element.to_owned(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    value: Value,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementSummary {
    pub resource_id: String,
    pub element_id: String,
    pub class: String,
    pub name: Option<String>,
    pub owner: Option<String>,
}
impl Default for Document {
    fn default() -> Self {
        Self::empty()
    }
}
impl Document {
    pub fn empty() -> Self {
        Self {
            value: json!({"json":{"version":"1.0","encoding":"utf-8"},
            "ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[]}),
        }
    }
    /// Parse literal JSON before model validation. Decoded duplicate keys refuse;
    /// opaque objects and exact numeric tokens are never serde metadata.
    /// This selected model admits at most 4 MiB, value-depth 128, two million
    /// values/keys and 8 MiB scalar-copy charge. Bridge byte limits remain lower.
    /// The Value view is canonical; upload whitespace/member order is not retained.
    pub fn from_json(input: &str) -> Result<Self> {
        let limits = syson_json_value::Limits {
            bytes: JSON_BYTES_LIMIT,
            depth: JSON_DEPTH_LIMIT,
            values: 2_000_000,
            keys: 2_000_000,
            scalar_bytes: 8 * 1024 * 1024,
        };
        let (value, _) = syson_json_value::parse(input, limits, || false)
            .and_then(|literal| literal.into_value(|| false))
            .map_err(|error| match error {
                syson_json_value::Error::Syntax { .. }
                | syson_json_value::Error::StringEncoding => ModelError::Json(error.to_string()),
                other => ModelError::JsonAdmission(other),
            })?;
        let document = Self { value };
        document.validate()?;
        document.check_json_budget()?;
        Ok(document)
    }
    fn check_json_budget(&self) -> Result<()> {
        self.json_size(JSON_BYTES_LIMIT, || false).map(|_| ())
    }
    /// Count canonical pretty JSON without allocating its output. Limits above
    /// 4 MiB refuse; provisional values must also fit the publication depth128.
    /// This is serialization admission, not validation of provisional semantics.
    pub fn json_size(
        &self,
        max_bytes: usize,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<usize> {
        if max_bytes > JSON_BYTES_LIMIT {
            return Err(ModelError::JsonAdmission(
                syson_json_value::Error::InvalidLimits,
            ));
        }
        fn depth(value: &Value, level: usize, cancelled: &mut impl FnMut() -> bool) -> Result<()> {
            json_cancelled(cancelled)?;
            if level > JSON_DEPTH_LIMIT {
                return Err(ModelError::JsonAdmission(syson_json_value::Error::Limit(
                    syson_json_value::Limit::Depth,
                )));
            }
            match value {
                Value::Array(rows) => {
                    for row in rows {
                        depth(row, level + 1, cancelled)?;
                    }
                }
                Value::Object(rows) => {
                    for row in rows.values() {
                        depth(row, level + 1, cancelled)?;
                    }
                }
                _ => (),
            }
            Ok(())
        }
        depth(&self.value, 1, &mut cancelled)?;
        self.emit_json(&mut std::io::sink(), max_bytes, &mut cancelled)
    }
    // One serializer and bounded/cancellable writer serve counting and emission.
    // Callers admit/count first; the exact admitted size bounds the second pass.
    fn emit_json<W: std::io::Write + ?Sized>(
        &self,
        target: &mut W,
        limit: usize,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<usize> {
        let mut writer = JsonWriter {
            target,
            cancelled,
            limit,
            written: 0,
            failure: None,
        };
        let encoded = serde_json::to_writer_pretty(&mut writer, &self.value);
        if let Some(error) = writer.failure.take() {
            return Err(error);
        }
        encoded.map_err(|error| ModelError::Json(error.to_string()))?;
        json_cancelled(writer.cancelled)?;
        Ok(writer.written)
    }
    /// Write after a complete counting/depth pass, with no owned output buffer.
    /// Admission/early cancellation touches no writer. Later I/O/cancellation can
    /// leave a prefix or complete output: the caller owns transport rollback.
    /// JsonOutputIo.written counts acknowledged bytes, not actual side effects
    /// of a writer that mutates before returning Err. No flush/durability promise.
    /// Cancellation is cooperative between value/write steps; library and caller
    /// writer calls are not internally preemptible or wall-clock bounded.
    pub fn write_json<W: std::io::Write + ?Sized>(
        &self,
        target: &mut W,
        max_bytes: usize,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<usize> {
        let size = self.json_size(max_bytes, &mut cancelled)?;
        self.emit_json(target, size, &mut cancelled)
    }
    /// Return complete canonical output or an error; no partial String escapes.
    /// Count/depth admission precedes the fallible exact-size buffer reservation.
    /// Provisional staged values do not bypass the same 4MiB/depth128 ceilings.
    pub fn to_json_bounded(
        &self,
        max_bytes: usize,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<String> {
        let size = self.json_size(max_bytes, &mut cancelled)?;
        json_cancelled(&mut cancelled)?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(size)
            .map_err(|_| ModelError::JsonAdmission(syson_json_value::Error::Allocation))?;
        self.emit_json(&mut output, size, &mut cancelled)?;
        String::from_utf8(output).map_err(|error| ModelError::Json(error.to_string()))
    }
    pub fn to_json(&self) -> Result<String> {
        self.to_json_bounded(JSON_BYTES_LIMIT, || false)
    }
    pub fn value(&self) -> &Value {
        &self.value
    }
    pub fn element(&self, id: &str) -> Option<&Value> {
        find_node(self.value.get("content")?.as_array()?, id)
    }
    /// Only containment, stable names and semantic owners; relationships included
    /// in this listing carry no semantic owner just as base ElementImpl.getOwner.
    pub fn elements(&self) -> Vec<ElementSummary> {
        let mut out = Vec::new();
        collect_summaries(
            self.value["content"].as_array().unwrap(),
            None,
            false,
            &mut out,
        );
        out
    }
    /// Source-class check for API capability decisions. Unknown classes and
    /// prefixes are preserved on import but never become positive type matches.
    pub fn is_instance_of(&self, stable_id: &str, expected: syson_metamodel::ClassifierId) -> bool {
        self.element(stable_id)
            .and_then(|node| scalar::node_classifier(node).ok())
            .is_some_and(|actual| syson_metamodel::is_subtype(actual, expected))
    }
    /// addChildInParent accepts any Element, selecting membership by Type/Feature.
    /// This reports command support, not cross-feature semantic validity.
    pub fn can_create_in(&self, stable_id: &str) -> bool {
        self.is_instance_of(stable_id, syson_metamodel::ClassifierId::Element)
    }
    pub fn projection(&self) -> Result<Projection> {
        let mut model = Model::default();
        let mut stable_ids = Vec::new();
        let mut handles = BTreeMap::new();
        for root in self.value["content"].as_array().unwrap() {
            project_node(root, None, &mut model, &mut stable_ids, &mut handles)?;
        }
        Ok(Projection {
            model,
            stable_ids,
            handles,
        })
    }
    fn validate(&self) -> Result<()> {
        if !self.value.is_object()
            || !self.value["json"].is_object()
            || self.value["json"]["version"] != "1.0"
            || self.value["ns"]["sysml"] != "http://www.eclipse.org/syson/sysml"
        {
            return Err(ModelError::Shape(
                "expected SysON EMF JSON 1.0 envelope".into(),
            ));
        }
        let content = self.value["content"]
            .as_array()
            .ok_or_else(|| ModelError::Shape("content must be array".into()))?;
        let mut resources = BTreeSet::new();
        let mut elements = BTreeSet::new();
        validate_nodes(content, &mut resources, &mut elements)
    }
}
/// Explicit mapping from process-local query handles to stable elementId strings.
pub struct Projection {
    pub model: Model,
    stable_ids: Vec<String>,
    handles: BTreeMap<String, ElementId>,
}
impl Projection {
    pub fn stable_id(&self, id: ElementId) -> Option<&str> {
        self.stable_ids.get(id.0).map(String::as_str)
    }
    pub fn handle(&self, stable_id: &str) -> Option<ElementId> {
        self.handles.get(stable_id).copied()
    }
}
fn validate_nodes(
    nodes: &[Value],
    resources: &mut BTreeSet<String>,
    elements: &mut BTreeSet<String>,
) -> Result<()> {
    for node in nodes {
        for key in ["id", "eClass"] {
            if !node[key].is_string() || node[key].as_str() == Some("") {
                return Err(ModelError::Shape(format!(
                    "node.{key} must be nonempty string"
                )));
            }
        }
        if !node["data"].is_object()
            || !node["data"]["elementId"].is_string()
            || node["data"]["elementId"].as_str() == Some("")
        {
            return Err(ModelError::Shape(
                "node.data.elementId must be nonempty string".into(),
            ));
        }
        for (id, seen) in [
            (node["id"].as_str().unwrap(), &mut *resources),
            (node["data"]["elementId"].as_str().unwrap(), &mut *elements),
        ] {
            if !seen.insert(id.to_ascii_lowercase()) {
                return Err(ModelError::DuplicateId(id.to_owned()));
            }
        }
        for key in ["ownedRelationship", "ownedRelatedElement"] {
            if let Some(value) = node["data"].get(key) {
                let children = value
                    .as_array()
                    .ok_or_else(|| ModelError::Shape(format!("data.{key} must be array")))?;
                validate_nodes(children, resources, elements)?;
            }
        }
    }
    Ok(())
}
fn find_node<'a>(nodes: &'a [Value], id: &str) -> Option<&'a Value> {
    for node in nodes {
        if node["data"]["elementId"].as_str() == Some(id) {
            return Some(node);
        }
        for key in ["ownedRelationship", "ownedRelatedElement"] {
            if let Some(children) = node["data"][key].as_array()
                && let Some(found) = find_node(children, id)
            {
                return Some(found);
            }
        }
    }
    None
}
fn find_node_mut<'a>(nodes: &'a mut [Value], id: &str) -> Option<&'a mut Value> {
    for node in nodes {
        if node["data"]["elementId"].as_str() == Some(id) {
            return Some(node);
        }
        // Search only the containment branch holding the ID to keep borrows disjoint.
        let branch = ["ownedRelationship", "ownedRelatedElement"]
            .into_iter()
            .find(|key| {
                node["data"][*key]
                    .as_array()
                    .is_some_and(|a| find_node(a, id).is_some())
            });
        if let Some(key) = branch {
            return find_node_mut(node["data"][key].as_array_mut().unwrap(), id);
        }
    }
    None
}
fn class(node: &Value) -> &str {
    node["eClass"]
        .as_str()
        .unwrap()
        .strip_prefix("sysml:")
        .unwrap_or(node["eClass"].as_str().unwrap())
}
fn id(node: &Value) -> &str {
    node["data"]["elementId"].as_str().unwrap()
}
fn collect_summaries(
    nodes: &[Value],
    owner: Option<&str>,
    relationships: bool,
    out: &mut Vec<ElementSummary>,
) {
    for node in nodes {
        out.push(ElementSummary {
            resource_id: node["id"].as_str().unwrap().into(),
            element_id: id(node).into(),
            class: class(node).into(),
            name: node["data"]["declaredName"].as_str().map(str::to_owned),
            owner: if relationships {
                None
            } else {
                owner.map(str::to_owned)
            },
        });
        if let Some(related) = node["data"]["ownedRelatedElement"].as_array() {
            collect_summaries(related, owner, false, out);
        }
        if let Some(relations) = node["data"]["ownedRelationship"].as_array() {
            collect_summaries(relations, Some(id(node)), true, out);
        }
    }
}
fn project_node(
    node: &Value,
    owner: Option<ElementId>,
    model: &mut Model,
    stable: &mut Vec<String>,
    handles: &mut BTreeMap<String, ElementId>,
) -> Result<()> {
    let actual_classifier = scalar::node_classifier(node).ok();
    let projected_class = actual_classifier.map_or_else(
        || format!("<unresolved:{}>", node["eClass"].as_str().unwrap()),
        |id| id.metadata().name().to_owned(),
    );
    let mut e = Element::new(projected_class, node["data"]["declaredName"].as_str());
    e.declared_short_name = node["data"]["declaredShortName"]
        .as_str()
        .map(str::to_owned);
    e.is_namespace = actual_classifier.is_some_and(|id| {
        syson_metamodel::is_subtype(id, syson_metamodel::ClassifierId::Namespace)
    });
    let handle = model
        .insert(e, owner)
        .map_err(|e| ModelError::Shape(format!("projection: {e:?}")))?;
    stable.push(id(node).to_owned());
    handles.insert(id(node).to_owned(), handle);
    // Flatten every owned Relationship, not only Membership, as getOwnedElement.
    if let Some(relationships) = node["data"]["ownedRelationship"].as_array() {
        for relationship in relationships {
            if let Some(related) = relationship["data"]["ownedRelatedElement"].as_array() {
                for child in related {
                    project_node(child, Some(handle), model, stable, handles)?;
                }
            }
        }
    }
    Ok(())
}

/// Proposed local snapshot editor. No persistence/network/concurrency claim.
#[derive(Debug)]
pub struct Editor {
    document: Document,
    undo: Vec<Document>,
    redo: Vec<Document>,
    revision: u64,
    identity: std::sync::Arc<()>,
}
impl Editor {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            undo: Vec::new(),
            redo: Vec::new(),
            revision: 0,
            identity: std::sync::Arc::new(()),
        }
    }
    pub fn document(&self) -> &Document {
        &self.document
    }
    pub fn begin(&self) -> Transaction {
        Transaction {
            base: self.document.clone(),
            staged: self.document.clone(),
            revision: self.revision,
            identity: self.identity.clone(),
            failed: false,
        }
    }
    /// Failed transactions are poisoned; no partial edits or history changes.
    pub fn commit(&mut self, transaction: Transaction) -> Result<bool> {
        if !std::sync::Arc::ptr_eq(&self.identity, &transaction.identity) {
            return Err(ModelError::ForeignTransaction);
        }
        if transaction.failed {
            return Err(ModelError::FailedTransaction);
        }
        if transaction.revision != self.revision || transaction.base != self.document {
            return Err(ModelError::StaleTransaction);
        }
        transaction.staged.validate()?;
        if transaction.staged == self.document {
            return Ok(false);
        }
        transaction.staged.check_json_budget()?;
        self.undo
            .push(std::mem::replace(&mut self.document, transaction.staged));
        self.redo.clear();
        self.revision += 1;
        Ok(true)
    }
    pub fn undo(&mut self) -> bool {
        if let Some(doc) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.document, doc));
            self.revision += 1;
            true
        } else {
            false
        }
    }
    pub fn redo(&mut self) -> bool {
        if let Some(doc) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.document, doc));
            self.revision += 1;
            true
        } else {
            false
        }
    }
}
#[derive(Debug)]
pub struct Transaction {
    base: Document,
    staged: Document,
    revision: u64,
    identity: std::sync::Arc<()>,
    failed: bool,
}
impl Transaction {
    pub fn document(&self) -> &Document {
        &self.staged
    }
    pub fn cancel(self) {}
    pub fn create_package(
        &mut self,
        parent: Option<&str>,
        ids: NodeIds,
        membership: Option<NodeIds>,
        name: &str,
    ) -> Result<()> {
        self.create(parent, ids, membership, "Package", name)
    }
    pub fn create_part_definition(
        &mut self,
        parent: &str,
        ids: NodeIds,
        membership: NodeIds,
        name: &str,
    ) -> Result<()> {
        self.create(Some(parent), ids, Some(membership), "PartDefinition", name)
    }
    pub fn create_part_usage(
        &mut self,
        parent: &str,
        ids: NodeIds,
        membership: NodeIds,
        name: &str,
    ) -> Result<()> {
        self.create(Some(parent), ids, Some(membership), "PartUsage", name)
    }
    fn create(
        &mut self,
        parent: Option<&str>,
        ids: NodeIds,
        membership: Option<NodeIds>,
        kind: &str,
        name: &str,
    ) -> Result<()> {
        self.apply(|doc| {
            if name.is_empty() { return Err(ModelError::InvalidName); }
            // NodeIds fields are public for ergonomic use, so validate again here.
            NodeIds::new(&ids.resource,&ids.element)?;
            let mut node=json!({"id":ids.resource,"eClass":format!("sysml:{kind}"),
                "data":{"elementId":ids.element,"declaredName":name}});
            if kind=="PartUsage" { node["data"]["isComposite"]=json!(true); }
            let content=doc.value["content"].as_array_mut().unwrap();
            if let Some(parent)=parent {
                let membership=membership.ok_or(ModelError::MissingMembership)?;
                NodeIds::new(&membership.resource,&membership.element)?;
                let parent=find_node_mut(content,parent).ok_or_else(||ModelError::MissingElement(parent.into()))?;
                let parent_class=scalar::node_classifier(parent)?;
                if !syson_metamodel::is_subtype(parent_class,syson_metamodel::ClassifierId::Element) {
                    return Err(ModelError::UnsupportedClass(class(parent).into()));
                }
                // Exact addChildInParent rule: Feature child in Type parent,
                // otherwise OwningMembership. These are still only three
                // supported constructors, not a complete classifier factory.
                let membership_kind=if kind=="PartUsage" && syson_metamodel::is_subtype(parent_class,syson_metamodel::ClassifierId::Type) {
                    "FeatureMembership"
                } else { "OwningMembership" };
                let relation=json!({"id":membership.resource,"eClass":format!("sysml:{membership_kind}"),
                    "data":{"elementId":membership.element,"ownedRelatedElement":[node]}});
                let data=parent["data"].as_object_mut().unwrap();
                data.entry("ownedRelationship").or_insert_with(||json!([])).as_array_mut().unwrap().push(relation);
            } else {
                if membership.is_some() { return Err(ModelError::UnexpectedMembership); }
                content.push(node);
            }
            doc.validate()
        })
    }
    pub fn rename(&mut self, element: &str, name: &str) -> Result<()> {
        self.apply(|doc| {
            if name.is_empty() {
                return Err(ModelError::InvalidName);
            }
            let node = find_node_mut(doc.value["content"].as_array_mut().unwrap(), element)
                .ok_or_else(|| ModelError::MissingElement(element.into()))?;
            if !matches!(class(node), "Package" | "PartDefinition" | "PartUsage") {
                return Err(ModelError::UnsupportedClass(class(node).into()));
            }
            node["data"]["declaredName"] = json!(name);
            Ok(())
        })
    }
    pub fn type_part_usage(
        &mut self,
        usage: &str,
        definition: &str,
        typing: NodeIds,
    ) -> Result<()> {
        self.apply(|doc| {
            NodeIds::new(&typing.resource, &typing.element)?;
            let target = doc
                .element(definition)
                .ok_or_else(|| ModelError::MissingElement(definition.into()))?;
            if !scalar::node_classifier(target).is_ok_and(|id| {
                syson_metamodel::is_subtype(id, syson_metamodel::ClassifierId::PartDefinition)
            }) {
                return Err(ModelError::WrongType {
                    element: definition.into(),
                    expected: "PartDefinition",
                });
            }
            let node = find_node_mut(doc.value["content"].as_array_mut().unwrap(), usage)
                .ok_or_else(|| ModelError::MissingElement(usage.into()))?;
            if !scalar::node_classifier(node).is_ok_and(|id| {
                syson_metamodel::is_subtype(id, syson_metamodel::ClassifierId::PartUsage)
            }) {
                return Err(ModelError::WrongType {
                    element: usage.into(),
                    expected: "PartUsage",
                });
            }
            let data = node["data"].as_object_mut().unwrap();
            let relations = data
                .entry("ownedRelationship")
                .or_insert_with(|| json!([]))
                .as_array_mut()
                .unwrap();
            relations.push(json!({"id":typing.resource,"eClass":"sysml:FeatureTyping",
                "data":{"elementId":typing.element,"type":definition,"typedFeature":usage}}));
            doc.validate()
        })
    }
    fn apply(&mut self, operation: impl FnOnce(&mut Document) -> Result<()>) -> Result<()> {
        if self.failed {
            return Err(ModelError::FailedTransaction);
        }
        let mut next = self.staged.clone();
        match operation(&mut next) {
            Ok(()) => {
                self.staged = next;
                Ok(())
            }
            Err(e) => {
                self.failed = true;
                Err(e)
            }
        }
    }
}
