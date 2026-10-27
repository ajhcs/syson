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
use std::collections::{BTreeMap, BTreeSet, VecDeque};
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
    RevisionExhausted,
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
/// Selected in-memory retention limits, separate from publication JSON limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryLimits {
    entries: usize,
    charge_bytes: usize,
}
impl Default for HistoryLimits {
    fn default() -> Self {
        Self {
            entries: 64,
            charge_bytes: 128 * 1024 * 1024,
        }
    }
}
impl HistoryLimits {
    /// Zero disables retention. Larger-than-default policies explicitly refuse.
    pub fn new(entries: usize, charge_bytes: usize) -> Option<Self> {
        let maximum = Self::default();
        (entries <= maximum.entries && charge_bytes <= maximum.charge_bytes).then_some(Self {
            entries,
            charge_bytes,
        })
    }
    pub fn entries(self) -> usize {
        self.entries
    }
    pub fn charge_bytes(self) -> usize {
        self.charge_bytes
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryAvailability {
    Available,
    Empty,
    RetentionBoundary,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryTruncation {
    Disabled,
    EntryBudget,
    ByteBudget,
    SnapshotTooLarge,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryState {
    pub undo: HistoryAvailability,
    pub redo: HistoryAvailability,
    pub undo_entries: usize,
    pub redo_entries: usize,
    /// Capacity charge plus conservative container allowance, NOT measured heap
    /// or RSS. Current document and open transactions are outside this budget.
    pub retained_charge_bytes: usize,
    pub limits: HistoryLimits,
    pub discarded_undo: u64,
    pub discarded_redo: u64,
    pub last_truncation: Option<HistoryTruncation>,
    pub revision_exhausted: bool,
}
#[derive(Debug)]
struct HistoryEntry {
    document: Document,
    charge: usize,
}
// Charge actual String/key/Vec capacities; include Value headers and a generous
// map base/per-slot allowance for the qualified BTreeMap backend. This is a
// retention policy, not a portable allocator/RSS theorem. Saturation refuses
// retaining oversized data. Number's private capacity is compacted to a fresh
// exact-token String before caching; no generic Value parsing or normalization.
fn history_charge(value: &mut Value, level: usize) -> usize {
    // Editor may receive a provisional Document clone; never recurse past the
    // publication depth even when that provisional value has not been admitted.
    if level > JSON_DEPTH_LIMIT {
        return usize::MAX;
    }
    let mut charge = std::mem::size_of::<Value>();
    let mut add = |bytes| {
        charge = charge.saturating_add(bytes);
    };
    match value {
        Value::String(text) => add(text.capacity()),
        Value::Number(number) => {
            let token = number.as_str().to_owned();
            add(token.capacity());
            *number = serde_json::Number::from_string_unchecked(token);
        }
        Value::Array(rows) => {
            add(rows.capacity().saturating_mul(std::mem::size_of::<Value>()));
            for row in rows {
                add(history_charge(row, level + 1));
            }
        }
        Value::Object(rows) => {
            add(1024usize.saturating_add(rows.len().saturating_mul(
                std::mem::size_of::<String>() + std::mem::size_of::<Value>() + 128,
            )));
            for (key, value) in rows {
                add(key.capacity());
                add(history_charge(value, level + 1));
            }
        }
        _ => (),
    }
    charge
}
#[derive(Debug)]
pub struct Editor {
    document: Document,
    current_charge: usize,
    undo: VecDeque<HistoryEntry>,
    redo: VecDeque<HistoryEntry>,
    history_limits: HistoryLimits,
    discarded_undo: u64,
    discarded_redo: u64,
    last_truncation: Option<HistoryTruncation>,
    revision: u64,
    identity: std::sync::Arc<()>,
}
impl Editor {
    pub fn new(document: Document) -> Self {
        Self::with_history_limits(document, HistoryLimits::default())
    }
    pub fn with_history_limits(mut document: Document, limits: HistoryLimits) -> Self {
        let current_charge = history_charge(&mut document.value, 0);
        Self {
            document,
            current_charge,
            undo: VecDeque::new(),
            redo: VecDeque::new(),
            history_limits: limits,
            discarded_undo: 0,
            discarded_redo: 0,
            last_truncation: None,
            revision: 0,
            identity: std::sync::Arc::new(()),
        }
    }
    pub fn document(&self) -> &Document {
        &self.document
    }
    fn retained_charge(&self) -> usize {
        self.undo
            .iter()
            .chain(&self.redo)
            .fold(0usize, |sum, entry| sum.saturating_add(entry.charge))
            .saturating_add(
                (self.undo.capacity().saturating_add(self.redo.capacity()))
                    .saturating_mul(std::mem::size_of::<HistoryEntry>()),
            )
    }
    pub fn history_state(&self) -> HistoryState {
        let availability = |length, discarded| {
            if length != 0 {
                HistoryAvailability::Available
            } else if discarded != 0 {
                HistoryAvailability::RetentionBoundary
            } else {
                HistoryAvailability::Empty
            }
        };
        HistoryState {
            undo: availability(self.undo.len(), self.discarded_undo),
            redo: availability(self.redo.len(), self.discarded_redo),
            undo_entries: self.undo.len(),
            redo_entries: self.redo.len(),
            retained_charge_bytes: self.retained_charge(),
            limits: self.history_limits,
            discarded_undo: self.discarded_undo,
            discarded_redo: self.discarded_redo,
            last_truncation: self.last_truncation,
            revision_exhausted: self.revision == u64::MAX,
        }
    }
    /// Metadata policy changes do not change document revisions. Retain nearest
    /// undo first; discard oldest undo then farthest redo until BOTH bounds fit.
    pub fn set_history_limits(&mut self, limits: HistoryLimits) {
        self.history_limits = limits;
        self.trim_history();
    }
    fn trim_history(&mut self) {
        loop {
            let length = self.undo.len().saturating_add(self.redo.len());
            let entries = length > self.history_limits.entries;
            let bytes = self.retained_charge() > self.history_limits.charge_bytes;
            if !entries && !bytes {
                break;
            }
            let reason =
                if self.history_limits.entries == 0 || self.history_limits.charge_bytes == 0 {
                    HistoryTruncation::Disabled
                } else if bytes && length == 1 {
                    HistoryTruncation::SnapshotTooLarge
                } else if entries {
                    HistoryTruncation::EntryBudget
                } else {
                    HistoryTruncation::ByteBudget
                };
            if self.undo.pop_front().is_some() {
                self.discarded_undo = self.discarded_undo.saturating_add(1);
            } else if self.redo.pop_front().is_some() {
                self.discarded_redo = self.discarded_redo.saturating_add(1);
            } else {
                self.undo = VecDeque::new();
                self.redo = VecDeque::new();
                break;
            }
            self.last_truncation = Some(reason);
            if self.undo.is_empty() {
                self.undo.shrink_to_fit();
            }
            if self.redo.is_empty() {
                self.redo.shrink_to_fit();
            }
        }
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
    pub fn commit(&mut self, mut transaction: Transaction) -> Result<bool> {
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
        let next_revision = self
            .revision
            .checked_add(1)
            .ok_or(ModelError::RevisionExhausted)?;
        transaction.staged.check_json_budget()?;
        let charge = history_charge(&mut transaction.staged.value, 0);
        let old = HistoryEntry {
            document: std::mem::replace(&mut self.document, transaction.staged),
            charge: std::mem::replace(&mut self.current_charge, charge),
        };
        self.redo.clear();
        self.redo.shrink_to_fit();
        self.discarded_redo = 0;
        self.undo.push_back(old);
        self.trim_history();
        self.revision = next_revision;
        Ok(true)
    }
    pub fn undo(&mut self) -> bool {
        let Some(next_revision) = self.revision.checked_add(1) else {
            return false;
        };
        if let Some(entry) = self.undo.pop_back() {
            self.redo.push_back(HistoryEntry {
                document: std::mem::replace(&mut self.document, entry.document),
                charge: std::mem::replace(&mut self.current_charge, entry.charge),
            });
            self.trim_history();
            self.revision = next_revision;
            true
        } else {
            false
        }
    }
    pub fn redo(&mut self) -> bool {
        let Some(next_revision) = self.revision.checked_add(1) else {
            return false;
        };
        if let Some(entry) = self.redo.pop_back() {
            self.undo.push_back(HistoryEntry {
                document: std::mem::replace(&mut self.document, entry.document),
                charge: std::mem::replace(&mut self.current_charge, entry.charge),
            });
            self.trim_history();
            self.revision = next_revision;
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

#[cfg(feature = "raw-factory-candidate")]
pub mod factory;
#[cfg(feature = "raw-factory-candidate")]
mod raw_admission;
#[cfg(feature = "raw-factory-candidate")]
pub use raw_admission::{RawAdmissionError, RawAdmissionReport, attach_raw_selected};

#[cfg(feature = "raw-factory-candidate")]
mod port_definition_refs;
#[cfg(feature = "raw-factory-candidate")]
pub use port_definition_refs::PortDefinitionReference;

#[cfg(test)]
mod rr20_revision_tests {
    use super::*;
    #[test]
    fn exhausted_revision_refuses_before_history_or_document_changes() {
        let mut editor = Editor::new(Document::empty());
        let mut transaction = editor.begin();
        transaction
            .create_package(
                None,
                NodeIds::new(
                    "10000000-0000-4000-8000-000000000001",
                    "20000000-0000-4000-8000-000000000001",
                )
                .unwrap(),
                None,
                "one",
            )
            .unwrap();
        assert!(editor.commit(transaction).unwrap());
        editor.revision = u64::MAX;
        let mut transaction = editor.begin();
        transaction
            .rename("20000000-0000-4000-8000-000000000001", "two")
            .unwrap();
        let original = editor.document.clone();
        let state = editor.history_state();
        assert!(state.revision_exhausted);
        assert_eq!(
            editor.commit(transaction),
            Err(ModelError::RevisionExhausted)
        );
        assert!(!editor.undo());
        assert!(!editor.redo());
        assert_eq!(editor.document, original);
        assert_eq!(editor.history_state(), state);
        // Noop remains a noop even when another successful revision is impossible.
        let noop = editor.begin();
        assert!(!editor.commit(noop).unwrap());
    }
}
