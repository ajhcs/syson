// SPDX-License-Identifier: EPL-2.0
// Source behavior attribution: see ../NOTICE.md and porting/ast/README.md.
//! Source-traced AST construction operations; not a parser, linker or Syside service port.
//!
//! Behavioral references and limits are recorded in porting/ast/README.md.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(u64);
impl NodeId {
    pub fn value(self) -> u64 {
        self.0
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
#[derive(Clone, Debug, Default)]
struct State {
    nodes: BTreeMap<NodeId, Node>,
    frames: Vec<Frame>,
    cst: BTreeMap<u64, NodeId>,
}

/// Only this builder mutates objects. Complete arenas expose no mutable references.
pub struct Builder {
    reflection: Reflection,
    limits: Limits,
    cancel: Arc<AtomicBool>,
    work: usize,
    next_id: u64,
    state: State,
}
impl Builder {
    pub fn new(reflection: Reflection, limits: Limits, cancel: Arc<AtomicBool>) -> Self {
        Self {
            reflection,
            limits,
            cancel,
            work: 0,
            next_id: 0,
            state: State::default(),
        }
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
        let saved = self.state.clone();
        match f(self) {
            Ok(v) => Ok(v),
            Err(e) => {
                self.state = saved;
                Err(e)
            }
        }
    }
    pub fn node(&self, id: NodeId) -> Result<&Node, Error> {
        self.state.nodes.get(&id).ok_or(Error::UnknownNode(id))
    }
    fn node_mut(&mut self, id: NodeId) -> Result<&mut Node, Error> {
        self.state.nodes.get_mut(&id).ok_or(Error::UnknownNode(id))
    }
    pub fn allocate(
        &mut self,
        kind: Option<&str>,
        origin: Option<Lexeme>,
    ) -> Result<NodeId, Error> {
        self.charge(1 + origin.as_ref().map_or(0, |x| x.text.len()))?;
        if let Some(k) = kind {
            if !self.reflection.types.contains_key(k) {
                return Err(Error::UnknownType(k.to_owned()));
            }
        }
        if self.state.nodes.len() >= self.limits.max_nodes {
            return Err(Error::Budget);
        }
        let id = NodeId(self.next_id);
        self.next_id = self.next_id.checked_add(1).ok_or(Error::Budget)?;
        self.state.nodes.insert(
            id,
            Node {
                kind: kind.map(str::to_owned),
                fields: BTreeMap::new(),
                origin,
                container: None,
            },
        );
        Ok(id)
    }
    fn push(&mut self, frame: Frame) -> Result<(), Error> {
        self.charge(1)?;
        if self.state.frames.len() >= self.limits.max_frames {
            return Err(Error::Budget);
        }
        self.state.frames.push(frame);
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
        self.charge(1 + converted.len())?;
        match self.state.frames.last_mut() {
            Some(Frame::Datatype(s)) => {
                s.push_str(converted);
                Ok(())
            }
            _ => Err(Error::WrongFrame),
        }
    }
    /// Unassigned, non-datatype subrule result becomes the active frame after merging.
    pub fn adopt_subrule(&mut self, target: NodeId) -> Result<(), Error> {
        self.transaction(|b| {
            let source = b.current_node()?;
            b.merge(target, source)?;
            *b.state.frames.last_mut().ok_or(Error::NoFrame)? = Frame::Node(target);
            Ok(())
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
        self.charge(1 + feature.len())?;
        self.node(id)?;
        self.validate_datum(&value)?;
        let fields = &mut self.node_mut(id)?.fields;
        match op {
            Assignment::Set => {
                fields.insert(feature.to_owned(), value);
            }
            Assignment::Presence => {
                fields.insert(
                    feature.to_owned(),
                    Datum {
                        value: Value::Boolean(true),
                        lexeme: value.lexeme,
                    },
                );
            }
            Assignment::Append => {
                let slot = fields
                    .entry(feature.to_owned())
                    .or_insert_with(|| Datum::new(Value::Array(Vec::new())));
                if !matches!(slot.value, Value::Array(_)) {
                    *slot = Datum::new(Value::Array(Vec::new()));
                }
                if let Value::Array(a) = &mut slot.value {
                    a.push(value);
                }
            }
        }
        Ok(())
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
            text: lexeme.text.clone(),
            target_type: target_type.to_owned(),
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
        self.charge(1)?;
        let kind = self.node(id)?.kind.clone().ok_or(Error::MissingType(id))?;
        let props = self
            .reflection
            .types
            .get(&kind)
            .ok_or(Error::UnknownType(kind))?
            .clone();
        self.charge(props.len())?;
        for p in props {
            let fields = &mut self.node_mut(id)?.fields;
            match p {
                DefaultValue::Array(name) => {
                    if !matches!(fields.get(&name).map(|x| &x.value), Some(Value::Array(_))) {
                        fields.insert(name, Datum::new(Value::Array(Vec::new())));
                    }
                }
                DefaultValue::Boolean(name) => {
                    fields
                        .entry(name)
                        .or_insert_with(|| Datum::new(Value::Boolean(false)));
                }
            }
        }
        Ok(())
    }
    /// Adopts source type only when target lacks one; source array entries come first.
    /// CST retargeting is explicit through associate_cst, never guessed from node spans.
    pub fn merge(&mut self, target: NodeId, source: NodeId) -> Result<(), Error> {
        self.transaction(|b| {
            if target == source {
                return Err(Error::SelfMerge);
            }
            let src = b.node(source)?.clone();
            b.node(target)?;
            b.charge(1 + src.fields.len())?;
            let adopted = b.node(target)?.kind.is_none() && src.kind.is_some();
            if adopted {
                b.node_mut(target)?.kind = src.kind;
            }
            for (key, old) in src.fields {
                b.validate_datum(&old)?;
                let fields = &mut b.node_mut(target)?.fields;
                match fields.get_mut(&key) {
                    None => {
                        fields.insert(key, old);
                    }
                    Some(new) => {
                        if let (Value::Array(a), Value::Array(z)) = (&old.value, &mut new.value) {
                            let mut joined = a.clone();
                            joined.append(z);
                            *z = joined;
                        }
                    }
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
            *b.state.frames.last_mut().ok_or(Error::NoFrame)? = Frame::Node(new);
            Ok(new)
        })
    }
    /// Associate a known CST identity with its AST element, including explicit retargets.
    pub fn associate_cst(&mut self, cst_id: u64, node: NodeId) -> Result<(), Error> {
        self.charge(1)?;
        self.node(node)?;
        self.state.cst.insert(cst_id, node);
        Ok(())
    }
    pub fn end_frame(&mut self) -> Result<Constructed, Error> {
        self.transaction(
            |b| match b.state.frames.last().cloned().ok_or(Error::NoFrame)? {
                Frame::Node(id) => {
                    b.defaults(id)?;
                    b.state.frames.pop();
                    Ok(Constructed::Node(id))
                }
                Frame::Datatype(s) => {
                    b.state.frames.pop();
                    Ok(Constructed::Text(s))
                }
            },
        )
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
            let n = self.node_mut(id)?;
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
                            property: key.clone(),
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
                                        property: key.clone(),
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
            root,
            nodes: self.state.nodes,
            cst: self.state.cst,
        })
    }
}
#[derive(Clone, Debug)]
pub struct Arena {
    root: NodeId,
    nodes: BTreeMap<NodeId, Node>,
    cst: BTreeMap<u64, NodeId>,
}
impl Arena {
    pub fn root(&self) -> NodeId {
        self.root
    }
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }
    pub fn nodes(&self) -> &BTreeMap<NodeId, Node> {
        &self.nodes
    }
    pub fn cst_element(&self, cst_id: u64) -> Option<NodeId> {
        self.cst.get(&cst_id).copied()
    }
}
