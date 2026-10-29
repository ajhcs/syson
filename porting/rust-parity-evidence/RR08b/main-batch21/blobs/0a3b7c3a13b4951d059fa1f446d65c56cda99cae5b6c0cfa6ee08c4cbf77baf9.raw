// SPDX-License-Identifier: EPL-2.0
// Source behavior attribution: see ../NOTICE.md and porting/ast/README.md.
//! Source-traced AST construction operations; not a parser, linker or Syside service port.
//!
//! Behavioral references and limits are recorded in porting/ast/README.md.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId {
    owner: u64,
    index: u64,
}
// Process-local identities never recycle, including after builder/arena destruction.
// Exhaustion fails closed; no allocation address is used as an ownership identity.
static NEXT_OWNER: AtomicU64 = AtomicU64::new(0);
fn reserve_owner(counter: &AtomicU64) -> Result<u64, Error> {
    let mut current = counter.load(Ordering::Relaxed);
    loop {
        let next = current
            .checked_add(1)
            .ok_or(Error::OwnerNamespaceExhausted)?;
        match counter.compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return Ok(current),
            Err(actual) => current = actual,
        }
    }
}
impl NodeId {
    pub fn value(self) -> u64 {
        self.index
    }
}

/// Half-open offsets in the same immutable input: UTF-8 bytes and UTF-16 code units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub bytes: std::ops::Range<usize>,
    pub utf16: std::ops::Range<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lexeme {
    pub text: String,
    pub span: Span,
}
impl Lexeme {
    pub fn from_source(source: &str, start: usize, end: usize) -> Result<Self, Error> {
        let text = source.get(start..end).ok_or(Error::InvalidSpan)?;
        let prefix = source.get(..start).ok_or(Error::InvalidSpan)?;
        let a = prefix.encode_utf16().count();
        Ok(Self {
            text: text.to_owned(),
            span: Span {
                bytes: start..end,
                utf16: a..a + text.encode_utf16().count(),
            },
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Datum {
    pub value: Value,
    pub lexeme: Option<Lexeme>,
}
impl Datum {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            lexeme: None,
        }
    }
    pub fn raw(lexeme: Lexeme) -> Self {
        Self {
            value: Value::String(lexeme.text.clone()),
            lexeme: Some(lexeme),
        }
    }
    pub fn node(id: NodeId) -> Self {
        Self::new(Value::Node(id))
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Node(NodeId),
    Array(Vec<Datum>),
    UnresolvedReference { text: String, target_type: String },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Slot<'a> {
    Absent,
    Present(&'a Datum),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Container {
    pub node: NodeId,
    pub property: String,
    pub index: Option<usize>,
}
#[derive(Clone, Debug)]
pub struct Node {
    kind: Option<String>,
    fields: BTreeMap<String, Datum>,
    origin: Option<Lexeme>,
    container: Option<Container>,
}
impl Node {
    pub fn kind(&self) -> Option<&str> {
        self.kind.as_deref()
    }
    pub fn slot(&self, name: &str) -> Slot<'_> {
        self.fields.get(name).map_or(Slot::Absent, Slot::Present)
    }
    pub fn fields(&self) -> &BTreeMap<String, Datum> {
        &self.fields
    }
    pub fn origin(&self) -> Option<&Lexeme> {
        self.origin.as_ref()
    }
    pub fn container(&self) -> Option<&Container> {
        self.container.as_ref()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefaultValue {
    Array(String),
    Boolean(String),
}
impl DefaultValue {
    fn name(&self) -> &str {
        match self {
            Self::Array(s) | Self::Boolean(s) => s,
        }
    }
}
/// Metadata must already have source-qualified inheritance/first-name-wins ordering.
#[derive(Clone, Debug, Default)]
pub struct Reflection {
    types: BTreeMap<String, Vec<DefaultValue>>,
}
impl Reflection {
    pub fn declare(&mut self, kind: String, properties: Vec<DefaultValue>) -> Result<(), Error> {
        if self.types.contains_key(&kind) {
            return Err(Error::DuplicateType(kind));
        }
        let mut seen = BTreeSet::new();
        self.types.insert(
            kind,
            properties
                .into_iter()
                .filter(|p| seen.insert(p.name().to_owned()))
                .collect(),
        );
        Ok(())
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_nodes: usize,
    pub max_frames: usize,
    /// Operation units and charged UTF8 copy bytes; spent work is not rolled
    /// back. This is not an allocator-byte cap or a cancellation deadline.
    pub max_work: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_nodes: 100_000,
            max_frames: 1024,
            max_work: 10_000_000,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Cancelled,
    Budget,
    InvalidSpan,
    UnknownNode(NodeId),
    ForeignOwner(NodeId),
    OwnerNamespaceExhausted,
    UnknownType(String),
    DuplicateType(String),
    NoFrame,
    WrongFrame,
    OpenFrames,
    SelfMerge,
    MissingType(NodeId),
    RepeatedContainment(NodeId),
    Cycle(NodeId),
    NestedContainmentArray,
    UnsupportedConstruct(String),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Assignment {
    Set,
    Presence,
    Append,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Constructed {
    Node(NodeId),
    Text(String),
}
#[derive(Clone, Debug)]
enum Frame {
    Node(NodeId),
    Datatype(String),
}
#[derive(Debug, Default)]
struct State {
    nodes: BTreeMap<NodeId, Node>,
    frames: Vec<Frame>,
    cst: BTreeMap<u64, NodeId>,
}

// Undo retains moved values rather than cloning untouched state. Inner commits
// remain reversible by their outer transaction; IDs and work are never journaled.
#[derive(Debug)]
enum Undo {
    Node(NodeId),
    Field(NodeId, String, Option<Datum>),
    Append(NodeId, String, usize),
    Kind(NodeId, Option<String>),
    PopFrame,
    PushFrame(Frame),
    Frame(Frame),
    Datatype(usize),
    Cst(u64, Option<NodeId>),
}
fn copied_text(
    text: &str,
    work: &mut usize,
    limit: usize,
    cancel: &AtomicBool,
) -> Result<String, Error> {
    if cancel.load(Ordering::Relaxed) {
        return Err(Error::Cancelled);
    }
    *work = work.checked_add(text.len()).ok_or(Error::Budget)?;
    if *work > limit {
        return Err(Error::Budget);
    }
    let mut result = String::with_capacity(text.len());
    let mut start = 0;
    while start < text.len() {
        if cancel.load(Ordering::Relaxed) {
            return Err(Error::Cancelled);
        }
        let mut end = (start + 4096).min(text.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        result.push_str(&text[start..end]);
        start = end;
    }
    if cancel.load(Ordering::Relaxed) {
        return Err(Error::Cancelled);
    }
    Ok(result)
}
fn copied_datum(
    d: &Datum,
    work: &mut usize,
    limit: usize,
    cancel: &AtomicBool,
) -> Result<Datum, Error> {
    if cancel.load(Ordering::Relaxed) {
        return Err(Error::Cancelled);
    }
    *work = work.checked_add(1).ok_or(Error::Budget)?;
    if *work > limit {
        return Err(Error::Budget);
    }
    let lexeme = d
        .lexeme
        .as_ref()
        .map(|x| -> Result<Lexeme, Error> {
            Ok(Lexeme {
                text: copied_text(&x.text, work, limit, cancel)?,
                span: x.span.clone(),
            })
        })
        .transpose()?;
    let value = match &d.value {
        Value::String(s) => Value::String(copied_text(s, work, limit, cancel)?),
        Value::UnresolvedReference {
            text: s,
            target_type,
        } => Value::UnresolvedReference {
            text: copied_text(s, work, limit, cancel)?,
            target_type: copied_text(target_type, work, limit, cancel)?,
        },
        Value::Array(values) => {
            // Validated builder arrays are flat. Copy entries individually and
            // charge each before vector growth; nested containment stays refused.
            let mut result = Vec::new();
            for value in values {
                if cancel.load(Ordering::Relaxed) {
                    return Err(Error::Cancelled);
                }
                result.push(copied_datum(value, work, limit, cancel)?);
            }
            Value::Array(result)
        }
        value => value.clone(),
    };
    Ok(Datum { value, lexeme })
}
/// Only this builder mutates objects. Complete arenas expose no mutable references.
pub struct Builder {
    owner: u64,
    reflection: Reflection,
    limits: Limits,
    cancel: Arc<AtomicBool>,
    work: usize,
    next_id: u64,
    state: State,
    undo: Vec<Undo>,
    transactions: usize,
}
impl Builder {
    pub fn new(
        reflection: Reflection,
        limits: Limits,
        cancel: Arc<AtomicBool>,
    ) -> Result<Self, Error> {
        Ok(Self {
            owner: reserve_owner(&NEXT_OWNER)?,
            reflection,
            limits,
            cancel,
            work: 0,
            next_id: 0,
            state: State::default(),
            undo: Vec::new(),
            transactions: 0,
        })
    }
    fn charge(&mut self, n: usize) -> Result<(), Error> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err(Error::Cancelled);
        }
        self.work = self.work.checked_add(n).ok_or(Error::Budget)?;
        if self.work > self.limits.max_work {
            return Err(Error::Budget);
        }
        Ok(())
    }
    /// Roll back nodes, frames and CST associations on Err. IDs/work are never recycled.
    /// Panics are not a rollback API; callers must return construction errors.
    pub fn transaction<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.charge(1)?;
        let mark = self.undo.len();
        self.transactions += 1;
        let result = f(self).and_then(|value| {
            self.charge(0)?;
            Ok(value)
        });
        self.transactions -= 1;
        if result.is_err() {
            while self.undo.len() > mark {
                match self.undo.pop().unwrap() {
                    Undo::Node(id) => {
                        self.state.nodes.remove(&id);
                    }
                    Undo::Field(id, key, old) => {
                        let fields = &mut self.state.nodes.get_mut(&id).unwrap().fields;
                        if let Some(old) = old {
                            fields.insert(key, old);
                        } else {
                            fields.remove(&key);
                        }
                    }
                    Undo::Append(id, key, len) => {
                        if let Value::Array(a) = &mut self
                            .state
                            .nodes
                            .get_mut(&id)
                            .unwrap()
                            .fields
                            .get_mut(&key)
                            .unwrap()
                            .value
                        {
                            a.truncate(len);
                        }
                    }
                    Undo::Kind(id, kind) => self.state.nodes.get_mut(&id).unwrap().kind = kind,
                    Undo::PopFrame => {
                        self.state.frames.pop();
                    }
                    Undo::PushFrame(frame) => self.state.frames.push(frame),
                    Undo::Frame(frame) => *self.state.frames.last_mut().unwrap() = frame,
                    Undo::Datatype(len) => {
                        if let Some(Frame::Datatype(s)) = self.state.frames.last_mut() {
                            s.truncate(len);
                        }
                    }
                    Undo::Cst(id, old) => {
                        if let Some(old) = old {
                            self.state.cst.insert(id, old);
                        } else {
                            self.state.cst.remove(&id);
                        }
                    }
                }
            }
        }
        if self.transactions == 0 {
            self.undo.clear();
        }
        result
    }
    fn record(&mut self, undo: Undo) {
        if self.transactions != 0 {
            self.undo.push(undo);
        }
    }
    fn text(&mut self, text: &str) -> Result<String, Error> {
        copied_text(text, &mut self.work, self.limits.max_work, &self.cancel)
    }
    fn put_field(&mut self, id: NodeId, key: String, value: Datum) -> Result<(), Error> {
        let journal_key = self.text(&key)?;
        let old = self.node_mut(id)?.fields.insert(key, value);
        self.record(Undo::Field(id, journal_key, old));
        Ok(())
    }
    fn replace_frame(&mut self, frame: Frame) -> Result<(), Error> {
        let old = std::mem::replace(self.state.frames.last_mut().ok_or(Error::NoFrame)?, frame);
        self.record(Undo::Frame(old));
        Ok(())
    }
    pub fn node(&self, id: NodeId) -> Result<&Node, Error> {
        if id.owner != self.owner {
            return Err(Error::ForeignOwner(id));
        }
        self.state.nodes.get(&id).ok_or(Error::UnknownNode(id))
    }
    fn node_mut(&mut self, id: NodeId) -> Result<&mut Node, Error> {
        if id.owner != self.owner {
            return Err(Error::ForeignOwner(id));
        }
        self.state.nodes.get_mut(&id).ok_or(Error::UnknownNode(id))
    }
    pub fn allocate(
        &mut self,
        kind: Option<&str>,
        origin: Option<Lexeme>,
    ) -> Result<NodeId, Error> {
        self.charge(1 + origin.as_ref().map_or(0, |x| x.text.len()))?;
        let kind = match kind {
            Some(k) => {
                let copied = self.text(k)?;
                if !self.reflection.types.contains_key(k) {
                    return Err(Error::UnknownType(copied));
                }
                Some(copied)
            }
            None => None,
        };
        if self.state.nodes.len() >= self.limits.max_nodes {
            return Err(Error::Budget);
        }
        let id = NodeId {
            owner: self.owner,
            index: self.next_id,
        };
        self.next_id = self.next_id.checked_add(1).ok_or(Error::Budget)?;
        self.state.nodes.insert(
            id,
            Node {
                kind,
                fields: BTreeMap::new(),
                origin,
                container: None,
            },
        );
        self.record(Undo::Node(id));
        Ok(id)
    }
    fn push(&mut self, frame: Frame) -> Result<(), Error> {
        self.charge(1)?;
        if self.state.frames.len() >= self.limits.max_frames {
            return Err(Error::Budget);
        }
        self.state.frames.push(frame);
        self.record(Undo::PopFrame);
        Ok(())
    }
    pub fn begin_node(
        &mut self,
        kind: Option<&str>,
        origin: Option<Lexeme>,
    ) -> Result<NodeId, Error> {
        self.transaction(|b| {
            let id = b.allocate(kind, origin)?;
            b.push(Frame::Node(id))?;
            Ok(id)
        })
    }
    pub fn begin_datatype(&mut self) -> Result<(), Error> {
        self.push(Frame::Datatype(String::new()))
    }
    /// Adapter supplies already converted pieces. No hidden whitespace is inserted.
    pub fn append_datatype(&mut self, converted: &str) -> Result<(), Error> {
        self.transaction(|b| {
            let (len, capacity) = match b.state.frames.last() {
                Some(Frame::Datatype(s)) => (s.len(), s.capacity()),
                _ => return Err(Error::WrongFrame),
            };
            let required = len.checked_add(converted.len()).ok_or(Error::Budget)?;
            // Reserve may copy the old buffer as well as the appended bytes.
            b.charge(
                converted
                    .len()
                    .checked_add(if required > capacity { len } else { 0 })
                    .ok_or(Error::Budget)?,
            )?;
            b.record(Undo::Datatype(len));
            if let Some(Frame::Datatype(s)) = b.state.frames.last_mut() {
                s.try_reserve_exact(converted.len())
                    .map_err(|_| Error::Budget)?;
                let mut start = 0;
                while start < converted.len() {
                    if b.cancel.load(Ordering::Relaxed) {
                        return Err(Error::Cancelled);
                    }
                    let mut end = (start + 4096).min(converted.len());
                    while !converted.is_char_boundary(end) {
                        end -= 1;
                    }
                    s.push_str(&converted[start..end]);
                    start = end;
                }
            }
            b.charge(0)
        })
    }

    /// Unassigned, non-datatype subrule result becomes the active frame after merging.
    pub fn adopt_subrule(&mut self, target: NodeId) -> Result<(), Error> {
        self.transaction(|b| {
            let source = b.current_node()?;
            b.merge(target, source)?;
            b.replace_frame(Frame::Node(target))
        })
    }
    /// Langium suppresses assigned subrule results whose composite CST is empty.
    /// The adapter must supply the actual composite-CST length, not token-count guesses.
    pub fn assign_subrule(
        &mut self,
        feature: &str,
        op: Assignment,
        child: NodeId,
        cst_length: usize,
    ) -> Result<bool, Error> {
        self.charge(1)?;
        let parent = self.current_node()?;
        self.node(child)?;
        if cst_length == 0 {
            return Ok(false);
        }
        self.assign(parent, feature, op, Datum::node(child))?;
        Ok(true)
    }
    pub fn current_node(&self) -> Result<NodeId, Error> {
        match self.state.frames.last() {
            Some(Frame::Node(id)) => Ok(*id),
            Some(_) => Err(Error::WrongFrame),
            None => Err(Error::NoFrame),
        }
    }
    fn validate_datum(&mut self, datum: &Datum) -> Result<(), Error> {
        // Iterative traversal avoids attacker-controlled recursive validation depth.
        let mut pending = vec![datum];
        while let Some(d) = pending.pop() {
            self.charge(1 + d.lexeme.as_ref().map_or(0, |x| x.text.len()))?;
            match &d.value {
                Value::Node(n) => {
                    self.node(*n)?;
                }
                Value::Array(a) => {
                    if a.iter().any(|v| matches!(v.value, Value::Array(_))) {
                        return Err(Error::NestedContainmentArray);
                    }
                    pending.extend(a);
                }
                Value::String(s) => self.charge(s.len())?,
                Value::UnresolvedReference { text, target_type } => {
                    self.charge(text.len() + target_type.len())?
                }
                _ => (),
            }
        }
        Ok(())
    }
    pub fn assign(
        &mut self,
        id: NodeId,
        feature: &str,
        op: Assignment,
        value: Datum,
    ) -> Result<(), Error> {
        self.transaction(|b| {
            b.charge(1 + feature.len())?;
            b.node(id)?;
            if op == Assignment::Append && matches!(value.value, Value::Array(_)) {
                return Err(Error::NestedContainmentArray);
            }
            b.validate_datum(&value)?;
            let key = b.text(feature)?;
            match op {
                Assignment::Set => b.put_field(id, key, value)?,
                Assignment::Presence => b.put_field(
                    id,
                    key,
                    Datum {
                        value: Value::Boolean(true),
                        lexeme: value.lexeme,
                    },
                )?,
                Assignment::Append => {
                    if let Some(Datum {
                        value: Value::Array(a),
                        ..
                    }) = b.node(id)?.fields.get(feature)
                    {
                        let copied = if a.len() == a.capacity() { a.len() } else { 0 };
                        b.charge(copied)?;
                        let Datum {
                            value: Value::Array(a),
                            ..
                        } = b.node_mut(id)?.fields.get_mut(feature).unwrap()
                        else {
                            unreachable!()
                        };
                        let len = a.len();
                        a.try_reserve(1).map_err(|_| Error::Budget)?;
                        a.push(value);
                        b.record(Undo::Append(id, key, len));
                    } else {
                        b.put_field(id, key, Datum::new(Value::Array(vec![value])))?;
                    }
                }
            }
            Ok(())
        })
    }

    /// Plain unresolved reference construction; resolution and scope are outside this crate.
    pub fn reference(
        &mut self,
        id: NodeId,
        feature: &str,
        op: Assignment,
        lexeme: Lexeme,
        target_type: &str,
    ) -> Result<(), Error> {
        let value = Value::UnresolvedReference {
            text: self.text(&lexeme.text)?,
            target_type: self.text(target_type)?,
        };
        self.assign(
            id,
            feature,
            op,
            Datum {
                value,
                lexeme: Some(lexeme),
            },
        )
    }
    pub fn defaults(&mut self, id: NodeId) -> Result<(), Error> {
        self.transaction(|b| {
            b.charge(1)?;
            let kind = b.node(id)?.kind.as_deref().ok_or(Error::MissingType(id))?;
            let props = b
                .reflection
                .types
                .get(kind)
                .ok_or_else(|| Error::UnknownType(kind.into()))?;
            let mut copied = Vec::new();
            for p in props {
                let name = copied_text(p.name(), &mut b.work, b.limits.max_work, &b.cancel)?;
                copied.push(match p {
                    DefaultValue::Array(_) => DefaultValue::Array(name),
                    DefaultValue::Boolean(_) => DefaultValue::Boolean(name),
                });
            }
            b.charge(copied.len())?;
            for p in copied {
                let replace = match &p {
                    DefaultValue::Array(name) => !matches!(
                        b.node(id)?.fields.get(name).map(|x| &x.value),
                        Some(Value::Array(_))
                    ),
                    DefaultValue::Boolean(name) => !b.node(id)?.fields.contains_key(name),
                };
                if replace {
                    let (name, value) = match p {
                        DefaultValue::Array(name) => (name, Value::Array(Vec::new())),
                        DefaultValue::Boolean(name) => (name, Value::Boolean(false)),
                    };
                    b.put_field(id, name, Datum::new(value))?;
                }
            }
            Ok(())
        })
    }

    /// Adopts source type only when target lacks one; source array entries come first.
    /// CST retargeting is explicit through associate_cst, never guessed from node spans.
    pub fn merge(&mut self, target: NodeId, source: NodeId) -> Result<(), Error> {
        self.transaction(|b| {
            if target == source {
                return Err(Error::SelfMerge);
            }
            b.node(source)?;
            b.node(target)?;
            let src = &b.state.nodes[&source];
            let kind = src
                .kind
                .as_ref()
                .map(|s| copied_text(s, &mut b.work, b.limits.max_work, &b.cancel))
                .transpose()?;
            let mut fields = BTreeMap::new();
            for (key, value) in &src.fields {
                let key = copied_text(key, &mut b.work, b.limits.max_work, &b.cancel)?;
                let value = copied_datum(value, &mut b.work, b.limits.max_work, &b.cancel)?;
                fields.insert(key, value);
            }
            b.charge(1 + fields.len())?;
            let adopted = b.node(target)?.kind.is_none() && kind.is_some();
            if adopted {
                let old = std::mem::replace(&mut b.node_mut(target)?.kind, kind);
                b.record(Undo::Kind(target, old));
            }
            for (key, old) in fields {
                b.validate_datum(&old)?;
                match b.node(target)?.fields.get(&key) {
                    None => b.put_field(target, key, old)?,
                    Some(new)
                        if matches!(
                            (&old.value, &new.value),
                            (Value::Array(_), Value::Array(_))
                        ) =>
                    {
                        // Retain the target by moving it into undo. Only its
                        // entries are copied, with scalar work charged/polled.
                        let mut joined = if let Value::Array(a) = old.value {
                            a
                        } else {
                            unreachable!()
                        };
                        let copied = copied_datum(
                            &b.state.nodes[&target].fields[&key],
                            &mut b.work,
                            b.limits.max_work,
                            &b.cancel,
                        )?;
                        if let Value::Array(mut a) = copied.value {
                            joined.append(&mut a);
                        }
                        b.put_field(
                            target,
                            key,
                            Datum {
                                value: Value::Array(joined),
                                lexeme: copied.lexeme,
                            },
                        )?;
                    }
                    _ => (),
                }
            }
            if adopted {
                b.defaults(target)?;
            }
            Ok(())
        })
    }
    pub fn action(
        &mut self,
        kind: &str,
        feature: Option<(&str, Assignment)>,
        origin: Option<Lexeme>,
    ) -> Result<NodeId, Error> {
        self.transaction(|b| {
            let old = b.current_node()?;
            // This boundary models an already chosen action; adapter owns construct(false)/CST sequencing.
            if feature.is_some() {
                b.defaults(old)?;
            }
            let new = b.allocate(Some(kind), origin)?;
            if let Some((name, op)) = feature {
                b.assign(new, name, op, Datum::node(old))?;
            }
            b.replace_frame(Frame::Node(new))?;
            Ok(new)
        })
    }
    /// Associate a known CST identity with its AST element, including explicit retargets.
    pub fn associate_cst(&mut self, cst_id: u64, node: NodeId) -> Result<(), Error> {
        self.charge(1)?;
        self.node(node)?;
        let old = self.state.cst.insert(cst_id, node);
        self.record(Undo::Cst(cst_id, old));
        Ok(())
    }
    pub fn end_frame(&mut self) -> Result<Constructed, Error> {
        self.transaction(|b| {
            let result = match b.state.frames.last().ok_or(Error::NoFrame)? {
                Frame::Node(id) => {
                    let id = *id;
                    b.defaults(id)?;
                    Constructed::Node(id)
                }
                Frame::Datatype(s) => {
                    Constructed::Text(copied_text(s, &mut b.work, b.limits.max_work, &b.cancel)?)
                }
            };
            let frame = b.state.frames.pop().unwrap();
            b.record(Undo::PushFrame(frame));
            Ok(result)
        })
    }

    pub fn unsupported<T>(&self, construct: &str) -> Result<T, Error> {
        Err(Error::UnsupportedConstruct(construct.to_owned()))
    }
    /// Freeze only reachable nodes, deriving containment and rejecting cycles/shared children.
    /// No parse, linking, validation, CLI serialization or import outcome is implied.
    pub fn finish(mut self, root: NodeId) -> Result<Arena, Error> {
        self.charge(1)?;
        if !self.state.frames.is_empty() {
            return Err(Error::OpenFrames);
        }
        self.node(root)?;
        let mut seen = BTreeSet::new();
        let mut active = BTreeSet::new();
        let mut pending = vec![(root, None, false)];
        while let Some((id, parent, exit)) = pending.pop() {
            self.charge(1)?;
            if exit {
                active.remove(&id);
                continue;
            }
            if active.contains(&id) {
                return Err(Error::Cycle(id));
            }
            if !seen.insert(id) {
                return Err(Error::RepeatedContainment(id));
            }
            active.insert(id);
            self.defaults(id)?;
            self.node(id)?;
            let n = self
                .state
                .nodes
                .get_mut(&id)
                .ok_or(Error::UnknownNode(id))?;
            if n.kind.is_none() {
                return Err(Error::MissingType(id));
            }
            n.container = parent;
            let mut children = Vec::new();
            for (key, d) in &n.fields {
                match &d.value {
                    Value::Node(child) => children.push((
                        *child,
                        Some(Container {
                            node: id,
                            property: copied_text(
                                key,
                                &mut self.work,
                                self.limits.max_work,
                                &self.cancel,
                            )?,
                            index: None,
                        }),
                        false,
                    )),
                    Value::Array(a) => {
                        for (i, v) in a.iter().enumerate() {
                            match v.value {
                                Value::Node(child) => children.push((
                                    child,
                                    Some(Container {
                                        node: id,
                                        property: copied_text(
                                            key,
                                            &mut self.work,
                                            self.limits.max_work,
                                            &self.cancel,
                                        )?,
                                        index: Some(i),
                                    }),
                                    false,
                                )),
                                Value::Array(_) => return Err(Error::NestedContainmentArray),
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                }
            }
            pending.push((id, None, true));
            pending.extend(children.into_iter().rev());
        }
        self.state.nodes.retain(|id, _| seen.contains(id));
        self.state.cst.retain(|_, id| seen.contains(id));
        Ok(Arena {
            owner: self.owner,
            root,
            nodes: self.state.nodes,
            cst: self.state.cst,
        })
    }
}
#[derive(Clone, Debug)]
pub struct Arena {
    owner: u64,
    root: NodeId,
    nodes: BTreeMap<NodeId, Node>,
    cst: BTreeMap<u64, NodeId>,
}
impl Arena {
    pub fn root(&self) -> NodeId {
        self.root
    }
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        if id.owner != self.owner {
            return None;
        }
        self.nodes.get(&id)
    }
    pub fn nodes(&self) -> &BTreeMap<NodeId, Node> {
        &self.nodes
    }
    pub fn cst_element(&self, cst_id: u64) -> Option<NodeId> {
        self.cst.get(&cst_id).copied()
    }
}

#[cfg(test)]
mod owner_namespace_tests {
    use super::*;
    #[test]
    fn namespace_exhaustion_never_wraps_or_reuses() {
        let counter = AtomicU64::new(u64::MAX - 1);
        assert_eq!(reserve_owner(&counter), Ok(u64::MAX - 1));
        assert_eq!(reserve_owner(&counter), Err(Error::OwnerNamespaceExhausted));
        assert_eq!(reserve_owner(&counter), Err(Error::OwnerNamespaceExhausted));
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    }
}
