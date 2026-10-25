// Copyright (c) 2022-2025 Sensmetry UAB and others.
// SPDX-License-Identifier: EPL-2.0 OR GPL-2.0 WITH Classpath-exception-2.0
//! Finite local reference chains over immutable F11 syntax and explicit F04 bindings.
#![forbid(unsafe_code)]
use serde_json::Value as Json;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use syson_ast::{Arena, NodeId, Slot, Value};
use syson_metamodel::{self as mm, ClassifierId as C};
use syson_references::{Registry, Target};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Unsupported,
    Limit,
    Ambiguous,
    ForeignResult,
    AliasCycle,
    Unresolved(String),
    TypeMismatch { expected: C, actual: C },
    MissingBinding(NodeId),
    Reference(syson_references::Error),
}
/// Alias syntax is not parsed by F11. These are caller-supplied local observations.
pub struct Alias {
    pub scope: NodeId,
    pub name: String,
    pub parts: Vec<String>,
}
/// A declaration-to-JSON-node UUID association, never an inferred semantic ID.
pub struct Binding {
    pub declaration: NodeId,
    pub wire: Json,
}
#[derive(Clone)]
enum Entry {
    Declaration(NodeId),
    Alias(usize),
}
pub struct Link {
    snapshot: Arc<()>,
    reference: NodeId,
    found: Vec<NodeId>,
    error: Option<(usize, Error)>,
}
impl Link {
    pub fn reference(&self) -> NodeId {
        self.reference
    }
    pub fn found(&self) -> &[NodeId] {
        &self.found
    }
    pub fn error(&self) -> Option<&(usize, Error)> {
        self.error.as_ref()
    }
    pub fn target(&self) -> Option<NodeId> {
        if self.error.is_some() {
            None
        } else {
            self.found.last().copied()
        }
    }
}
pub struct Snapshot {
    identity: Arc<()>,
    arena: Arena,
    registry: Registry,
    resource: String,
    scopes: BTreeMap<NodeId, BTreeMap<String, Entry>>,
    parents: BTreeMap<NodeId, NodeId>,
    aliases: Vec<Alias>,
    bindings: BTreeMap<NodeId, Json>,
}
fn slot<'a>(arena: &'a Arena, id: NodeId, key: &str) -> Result<&'a Value, Error> {
    match arena.node(id).map(|n| n.slot(key)) {
        Some(Slot::Present(v)) => Ok(&v.value),
        _ => Err(Error::Unsupported),
    }
}
fn kind(arena: &Arena, id: NodeId) -> Result<C, Error> {
    arena
        .node(id)
        .and_then(|n| n.kind())
        .and_then(|n| mm::lookup(mm::Package::Domain, n))
        .ok_or(Error::Unsupported)
}
fn name(raw: &str) -> Result<String, Error> {
    if raw.is_empty() || raw.len() > 2048 {
        return Err(Error::Unsupported);
    }
    if !raw.starts_with('\'') {
        return Ok(raw.into());
    }
    let inner = raw
        .strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''))
        .ok_or(Error::Unsupported)?;
    let mut chars = inner.chars();
    let mut out = String::new();
    while let Some(c) = chars.next() {
        out.push(if c == '\\' {
            match chars.next().ok_or(Error::Unsupported)? {
                'b' => '\x08',
                't' => '\t',
                'n' => '\n',
                'f' => '\x0c',
                'r' => '\r',
                c @ ('\'' | '"' | '\\') => c,
                _ => return Err(Error::Unsupported),
            }
        } else {
            c
        });
    }
    Ok(out)
}
impl Snapshot {
    pub fn new(
        arena: Arena,
        registry: Registry,
        resource: String,
        aliases: Vec<Alias>,
        bindings: Vec<Binding>,
    ) -> Result<Self, Error> {
        if arena.nodes().len() > 4096 || aliases.len() > 256 || bindings.len() > 256 {
            return Err(Error::Limit);
        }
        let mut result = Self {
            identity: Arc::new(()),
            arena,
            registry,
            resource,
            scopes: BTreeMap::new(),
            parents: BTreeMap::new(),
            aliases,
            bindings: BTreeMap::new(),
        };
        result.index(result.arena.root(), 0)?;
        for i in 0..result.aliases.len() {
            let alias = &result.aliases[i];
            if alias.parts.is_empty() || alias.parts.len() > 64 {
                return Err(Error::Limit);
            }
            let key = name(&alias.name)?;
            for part in &alias.parts {
                name(part)?;
            }
            let scope = result
                .scopes
                .get_mut(&alias.scope)
                .ok_or(Error::Unsupported)?;
            if scope.insert(key, Entry::Alias(i)).is_some() {
                return Err(Error::Ambiguous);
            }
        }
        for b in bindings {
            if !result.parents.contains_key(&b.declaration)
                || !matches!(
                    syson_references::decode(Some(&b.wire)),
                    Ok(syson_references::Decoded::LocalNode(_))
                )
                || result.bindings.insert(b.declaration, b.wire).is_some()
            {
                return Err(Error::Unsupported);
            }
        }
        Ok(result)
    }
    pub fn arena(&self) -> &Arena {
        &self.arena
    }
    fn index(&mut self, id: NodeId, depth: usize) -> Result<(), Error> {
        if depth > 64 || self.scopes.len() == 256 {
            return Err(Error::Limit);
        }
        if !matches!(
            kind(&self.arena, id)?,
            C::Namespace | C::Package | C::PartDefinition | C::PartUsage
        ) || self.scopes.insert(id, BTreeMap::new()).is_some()
        {
            return Err(Error::Unsupported);
        }
        let Value::Array(children) = slot(&self.arena, id, "children")? else {
            return Err(Error::Unsupported);
        };
        let children: Vec<_> = children
            .iter()
            .map(|d| match d.value {
                Value::Node(n) => Ok(n),
                _ => Err(Error::Unsupported),
            })
            .collect::<Result<_, _>>()?;
        for membership in children {
            if !matches!(
                kind(&self.arena, membership)?,
                C::OwningMembership | C::FeatureMembership
            ) {
                return Err(Error::Unsupported);
            }
            let Value::Node(child) = slot(&self.arena, membership, "target")? else {
                return Err(Error::Unsupported);
            };
            let child = *child;
            self.parents.insert(child, id);
            if let Some(Slot::Present(d)) = self.arena.node(child).map(|n| n.slot("declaredName")) {
                let Value::String(raw) = &d.value else {
                    return Err(Error::Unsupported);
                };
                let key = name(raw)?;
                if self
                    .scopes
                    .get_mut(&id)
                    .unwrap()
                    .insert(key, Entry::Declaration(child))
                    .is_some()
                {
                    return Err(Error::Ambiguous);
                }
            }
            self.index(child, depth + 1)?;
        }
        Ok(())
    }
    fn lookup(
        &self,
        mut scope: NodeId,
        text: &str,
        lexical: bool,
        work: &mut usize,
    ) -> Result<Entry, Error> {
        loop {
            *work += 1;
            if *work > 4096 {
                return Err(Error::Limit);
            }
            if let Some(entry) = self.scopes.get(&scope).and_then(|s| s.get(text)) {
                return Ok(entry.clone());
            }
            if lexical && let Some(parent) = self.parents.get(&scope) {
                scope = *parent;
                continue;
            }
            return Err(Error::Unresolved(text.into()));
        }
    }
    fn chain(
        &self,
        scope: NodeId,
        parts: &[String],
        aliases: &mut BTreeSet<usize>,
        work: &mut usize,
    ) -> Result<NodeId, Error> {
        let mut scope = scope;
        for (index, part) in parts.iter().enumerate() {
            let entry = self.lookup(scope, &name(part)?, index == 0, work)?;
            scope = self.follow(entry, aliases, work)?;
        }
        Ok(scope)
    }
    fn follow(
        &self,
        entry: Entry,
        seen: &mut BTreeSet<usize>,
        work: &mut usize,
    ) -> Result<NodeId, Error> {
        match entry {
            Entry::Declaration(id) => Ok(id),
            Entry::Alias(i) => {
                if !seen.insert(i) {
                    return Err(Error::AliasCycle);
                }
                let alias = &self.aliases[i];
                let result = self.chain(alias.scope, &alias.parts, seen, work);
                seen.remove(&i);
                result
            }
        }
    }
    /// Link one F11 TypeReference (an ElementReference subtype) without changing syntax.
    pub fn link(&self, reference: NodeId) -> Result<Link, Error> {
        if !matches!(
            self.arena.node(reference).and_then(|n| n.kind()),
            Some("TypeReference" | "ElementReference")
        ) {
            return Err(Error::Unsupported);
        }
        let Value::Array(parts) = slot(&self.arena, reference, "parts")? else {
            return Err(Error::Unsupported);
        };
        if parts.is_empty() || parts.len() > 64 {
            return Err(Error::Limit);
        }
        let mut owner = reference;
        let mut scope = loop {
            let parent = self
                .arena
                .node(owner)
                .and_then(|n| n.container())
                .ok_or(Error::Unsupported)?
                .node;
            if self.scopes.contains_key(&parent) {
                break parent;
            }
            owner = parent;
        };
        let mut link = Link {
            snapshot: self.identity.clone(),
            reference,
            found: vec![],
            error: None,
        };
        let mut work = 0;
        for (index, part) in parts.iter().enumerate() {
            let wanted = if self.arena.node(reference).unwrap().kind() == Some("TypeReference")
                && index + 1 == parts.len()
            {
                C::Type
            } else {
                C::Element
            };
            let Value::UnresolvedReference { text, target_type } = &part.value else {
                return Err(Error::Unsupported);
            };
            if mm::lookup(mm::Package::Domain, target_type) != Some(wanted) {
                return Err(Error::Unsupported);
            }
            let result = name(text)
                .and_then(|text| self.lookup(scope, &text, index == 0, &mut work))
                .and_then(|entry| self.follow(entry, &mut BTreeSet::new(), &mut work));
            match result {
                Ok(id) => {
                    link.found.push(id);
                    let actual = kind(&self.arena, id)?;
                    if !mm::is_subtype(actual, wanted) {
                        link.error = Some((
                            index,
                            Error::TypeMismatch {
                                expected: wanted,
                                actual,
                            },
                        ));
                        break;
                    }
                    scope = id;
                }
                Err(error) => {
                    link.error = Some((index, error));
                    break;
                }
            }
        }
        Ok(link)
    }
    /// Binding is explicit and lazy. Successful AST lookup never invents a semantic ID.
    pub fn semantic(&self, link: &Link) -> Result<Target, Error> {
        if !Arc::ptr_eq(&self.identity, &link.snapshot) {
            return Err(Error::ForeignResult);
        }
        if let Some((_, error)) = &link.error {
            return Err(error.clone());
        }
        let id = link.target().ok_or(Error::Unsupported)?;
        let wire = self.bindings.get(&id).ok_or(Error::MissingBinding(id))?;
        self.registry
            .resolve(&self.resource, Some(wire), kind(&self.arena, id)?)
            .map_err(Error::Reference)?
            .ok_or(Error::Unsupported)
    }
    pub fn value(&self, target: &Target) -> Result<&Json, Error> {
        self.registry.target(target).map_err(Error::Reference)
    }
}
