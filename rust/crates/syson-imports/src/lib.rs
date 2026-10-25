// SPDX-License-Identifier: EPL-2.0
//! Public, resolved non-Type namespace evaluation from immutable observations.
//! Object tokens are caller-supplied identities, never inferred from UUIDs.
//! No EMF getter, proxy resolution, native binding or resource I/O executes here.
#![forbid(unsafe_code)]
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NamespaceId(pub String);
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberId(pub String);
#[derive(Clone, Debug)]
pub enum Observation<T> {
    Known(T),
    Unresolved,
}
impl<T> Observation<T> {
    fn read(&self) -> Result<&T, Error> {
        match self {
            Self::Known(value) => Ok(value),
            Self::Unresolved => Err(Error::Unresolved),
        }
    }
}
#[derive(Clone, Debug)]
pub enum Target {
    Absent,
    Element,
    Namespace(NamespaceId),
}
/// Already-observed Java String reference identity, not textual comparison.
#[derive(Clone, Debug)]
pub enum Alias {
    None,
    SameReference,
    Distinct(String),
}
#[derive(Clone, Debug)]
pub struct Member {
    pub target: Observation<Target>,
    pub alias: Observation<Alias>,
    pub effective_name: Observation<Option<String>>,
    pub effective_short_name: Observation<Option<String>>,
}
#[derive(Clone, Copy, Debug)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Unresolved,
}
#[derive(Clone, Debug)]
pub enum Import {
    Namespace {
        target: Observation<Option<NamespaceId>>,
        recursive: bool,
        include_all: bool,
    },
    Membership {
        target: Observation<Option<MemberId>>,
        recursive: bool,
        include_all: bool,
    },
    BaseNull,
}
#[derive(Clone, Debug)]
pub struct Namespace {
    pub is_type: bool,
    pub owned: Vec<(MemberId, Visibility)>,
    pub imports: Vec<(Import, Visibility)>,
}
#[derive(Clone, Debug)]
struct Data {
    namespaces: BTreeMap<NamespaceId, Observation<Namespace>>,
    members: BTreeMap<MemberId, Observation<Member>>,
}
#[derive(Clone, Debug)]
pub struct Snapshot(Arc<Data>);
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    DuplicateIdentity,
    MissingNamespace(NamespaceId),
    MissingMember(MemberId),
    Unresolved,
    UnsupportedType,
    UnsupportedIncludeAll,
    NullImport,
    Budget,
    InvalidLimits,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Memberships(Vec<MemberId>),
    NullUnimplemented,
}
#[derive(Clone, Copy)]
pub struct Limits {
    pub steps: usize,
    pub depth: usize,
}
/// Modes remain distinct: supplied import results are never called graph evaluation.
pub enum Query {
    Visible {
        namespace: NamespaceId,
        recursive: bool,
        include_all: bool,
    },
    Import(Import),
    Imported(NamespaceId),
    Supplied {
        owned: Vec<MemberId>,
        calls: Vec<Observation<Option<Vec<MemberId>>>>,
    },
    Filter {
        seed: Vec<String>,
        members: Vec<MemberId>,
    },
    MembershipsOfVisibility,
}
impl Snapshot {
    pub fn new(
        namespaces: Vec<(NamespaceId, Observation<Namespace>)>,
        members: Vec<(MemberId, Observation<Member>)>,
    ) -> Result<Self, Error> {
        let mut data = Data {
            namespaces: BTreeMap::new(),
            members: BTreeMap::new(),
        };
        for (id, value) in namespaces {
            if data.namespaces.insert(id, value).is_some() {
                return Err(Error::DuplicateIdentity);
            }
        }
        for (id, value) in members {
            if data.members.insert(id, value).is_some() {
                return Err(Error::DuplicateIdentity);
            }
        }
        Ok(Self(Arc::new(data)))
    }
    pub fn evaluate(
        &self,
        query: Query,
        excluded: &[NamespaceId],
        limits: Limits,
    ) -> Result<Outcome, Error> {
        // Bounded recursion is a proposed Rust admission policy, not a native limit.
        if limits.depth == 0 || limits.depth > 128 {
            return Err(Error::InvalidLimits);
        }
        let mut context = Context {
            snapshot: self,
            remaining: limits.steps,
            max_depth: limits.depth,
        };
        let mut visited = excluded.iter().cloned().collect();
        let result = match query {
            Query::Visible {
                namespace,
                recursive,
                include_all,
            } => context.visible(&namespace, recursive, include_all, &mut visited, 0)?,
            Query::Import(Import::BaseNull) => return Err(Error::NullImport),
            Query::Import(import) => context.import(&import, &mut visited, 0)?,
            Query::Imported(namespace) => context.imported(&namespace, excluded)?,
            Query::Supplied { owned, calls } => {
                let mut filter = context.seed(&owned)?;
                let mut result = Vec::new();
                for call in calls {
                    context.tick()?;
                    for id in call.read()?.as_ref().ok_or(Error::NullImport)? {
                        if context.accept(id, &mut filter)? {
                            result.push(id.clone());
                        }
                    }
                }
                result
            }
            Query::Filter { seed, members } => {
                let mut filter = seed.into_iter().collect();
                let mut result = Vec::new();
                for id in members {
                    if context.accept(&id, &mut filter)? {
                        result.push(id);
                    }
                }
                result
            }
            Query::MembershipsOfVisibility => return Ok(Outcome::NullUnimplemented),
        };
        Ok(Outcome::Memberships(result))
    }
}
struct Context<'a> {
    snapshot: &'a Snapshot,
    remaining: usize,
    max_depth: usize,
}
fn check(name: &Option<String>, used: &mut BTreeSet<String>) -> bool {
    name.as_ref().is_none_or(|name| used.insert(name.clone()))
}
fn public(visibility: Visibility) -> Result<bool, Error> {
    match visibility {
        Visibility::Public => Ok(true),
        Visibility::Private | Visibility::Protected => Ok(false),
        Visibility::Unresolved => Err(Error::Unresolved),
    }
}
impl Context<'_> {
    fn tick(&mut self) -> Result<(), Error> {
        self.remaining = self.remaining.checked_sub(1).ok_or(Error::Budget)?;
        Ok(())
    }
    fn namespace(&self, id: &NamespaceId) -> Result<&Namespace, Error> {
        self.snapshot
            .0
            .namespaces
            .get(id)
            .ok_or_else(|| Error::MissingNamespace(id.clone()))?
            .read()
    }
    fn member(&self, id: &MemberId) -> Result<&Member, Error> {
        self.snapshot
            .0
            .members
            .get(id)
            .ok_or_else(|| Error::MissingMember(id.clone()))?
            .read()
    }
    fn accept(&mut self, id: &MemberId, used: &mut BTreeSet<String>) -> Result<bool, Error> {
        self.tick()?;
        let member = self.member(id)?;
        let alias = member.alias.read()?;
        if matches!(member.target.read()?, Target::Absent) {
            return Ok(true);
        }
        if let Alias::Distinct(name) = alias {
            return Ok(used.insert(name.clone()));
        }
        if !check(member.effective_name.read()?, used) {
            return Ok(false);
        }
        Ok(check(member.effective_short_name.read()?, used))
    }
    fn seed(&mut self, owned: &[MemberId]) -> Result<BTreeSet<String>, Error> {
        let mut used = BTreeSet::new();
        for id in owned {
            self.tick()?;
            let member = self.member(id)?;
            if !matches!(member.target.read()?, Target::Absent) {
                for name in [
                    member.effective_name.read()?,
                    member.effective_short_name.read()?,
                ]
                .into_iter()
                .flatten()
                {
                    used.insert(name.clone());
                }
            }
        }
        Ok(used)
    }
    fn imported(
        &mut self,
        id: &NamespaceId,
        excluded: &[NamespaceId],
    ) -> Result<Vec<MemberId>, Error> {
        self.tick()?;
        let namespace = self.namespace(id)?;
        if namespace.is_type {
            return Err(Error::UnsupportedType);
        }
        let owned = namespace
            .owned
            .iter()
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        let imports = namespace.imports.clone();
        let mut used = self.seed(&owned)?;
        let mut excluded_and_self: BTreeSet<_> = excluded.iter().cloned().collect();
        excluded_and_self.insert(id.clone());
        let mut result = Vec::new();
        // Native Namespace.importedMemberships does not filter ownedImport visibility.
        for (import, _) in imports {
            self.tick()?;
            if matches!(import, Import::BaseNull) {
                return Err(Error::NullImport);
            }
            let mut visited = excluded_and_self.clone();
            for member in self.import(&import, &mut visited, 0)? {
                if self.accept(&member, &mut used)? {
                    result.push(member);
                }
            }
        }
        Ok(result)
    }
    fn visible(
        &mut self,
        id: &NamespaceId,
        recursive: bool,
        include_all: bool,
        visited: &mut BTreeSet<NamespaceId>,
        depth: usize,
    ) -> Result<Vec<MemberId>, Error> {
        self.tick()?;
        if visited.contains(id) {
            return Ok(Vec::new());
        }
        if depth >= self.max_depth {
            return Err(Error::Budget);
        }
        visited.insert(id.clone());
        let namespace = self.namespace(id)?;
        if include_all {
            return Err(Error::UnsupportedIncludeAll);
        }
        if namespace.is_type {
            return Err(Error::UnsupportedType);
        }
        let (owned, imports) = (namespace.owned.clone(), namespace.imports.clone());
        let mut used = BTreeSet::new();
        let mut result = Vec::new();
        for (member, visibility) in owned {
            self.tick()?;
            if public(visibility)? && self.accept(&member, &mut used)? {
                result.push(member);
            }
        }
        for (import, visibility) in imports {
            self.tick()?;
            if public(visibility)? {
                for member in self.import(&import, visited, depth + 1)? {
                    if self.accept(&member, &mut used)? {
                        result.push(member);
                    }
                }
            }
        }
        if recursive {
            let mut descendants = Vec::new();
            for member in &result {
                self.tick()?;
                if let Target::Namespace(namespace) = self.member(member)?.target.read()?.clone() {
                    descendants.extend(self.visible(
                        &namespace,
                        true,
                        false,
                        visited,
                        depth + 1,
                    )?);
                }
            }
            result.extend(descendants);
        }
        Ok(result)
    }
    fn import(
        &mut self,
        import: &Import,
        visited: &mut BTreeSet<NamespaceId>,
        depth: usize,
    ) -> Result<Vec<MemberId>, Error> {
        self.tick()?;
        match import {
            Import::Namespace {
                target,
                recursive,
                include_all,
            } => match target.read()? {
                Some(id) => self.visible(id, *recursive, *include_all, visited, depth + 1),
                None => Ok(Vec::new()),
            },
            Import::Membership {
                target,
                recursive,
                include_all,
            } => {
                let Some(id) = target.read()? else {
                    return Ok(Vec::new());
                };
                match self.member(id)?.target.read()?.clone() {
                    Target::Absent => Ok(Vec::new()),
                    Target::Namespace(namespace) if *recursive => {
                        if visited.contains(&namespace) {
                            return Ok(Vec::new());
                        }
                        let mut result = vec![id.clone()];
                        result.extend(self.visible(
                            &namespace,
                            true,
                            *include_all,
                            visited,
                            depth + 1,
                        )?);
                        Ok(result)
                    }
                    _ => Ok(vec![id.clone()]),
                }
            }
            // MembershipComputer dispatch defaults to an empty list, unlike the
            // polymorphic Import API used by Namespace.importedMemberships.
            Import::BaseNull => Ok(Vec::new()),
        }
    }
}
