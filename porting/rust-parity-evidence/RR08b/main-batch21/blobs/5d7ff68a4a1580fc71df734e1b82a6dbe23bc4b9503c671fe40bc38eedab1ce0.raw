// Copyright (c) 2022-2025 Sensmetry UAB and others.
// SPDX-License-Identifier: EPL-2.0 OR GPL-2.0 WITH Classpath-exception-2.0
//! Immutable, source-derived dispatch plans; no callback or parser executes.
//! Ancestor order retains F08's unqualified short-sort candidate policy.
#![forbid(unsafe_code)]
use std::sync::Arc;
use syson_reflection::{OrderingQualification, ReflectionDb, ReflectionError, TypeHandle};

/// The falsey source domain is classified explicitly, never by Rust truthiness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Falsey {
    Undefined,
    Null,
    False,
    Zero,
    NaN,
    EmptyString,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Entry<T> {
    Falsey(Falsey),
    Truthy(T),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchError {
    Reflection(ReflectionError),
    DuplicateRegistration,
}
impl From<ReflectionError> for DispatchError {
    fn from(value: ReflectionError) -> Self {
        Self::Reflection(value)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChainOrder {
    SubtypeFirst,
    SupertypeFirst,
}
/// Source construct accepts strings, symbols or undefined; unknown strings skip.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConstructInput<'a> {
    String(&'a str),
    Symbol,
    Undefined,
}

/// Owns an immutable reflection owner and registrations. Duplicate registration
/// keys are rejected as a Rust input policy, unlike repeated payload identities.
/// JS inherited object properties and dynamic registry mutation are not admitted.
#[derive(Debug)]
pub struct Registry<T> {
    db: Arc<ReflectionDb>,
    entries: Vec<(TypeHandle, Entry<T>)>,
    default: Option<Entry<T>>,
}
impl<T> Registry<T> {
    pub fn new(
        db: Arc<ReflectionDb>,
        entries: Vec<(TypeHandle, Entry<T>)>,
        default: Option<Entry<T>>,
    ) -> Result<Self, DispatchError> {
        for (i, (ty, _)) in entries.iter().enumerate() {
            db.name(ty)?;
            if entries[..i].iter().any(|(other, _)| other == ty) {
                return Err(DispatchError::DuplicateRegistration);
            }
        }
        Ok(Self {
            db,
            entries,
            default,
        })
    }
    pub fn ordering_qualification(&self) -> OrderingQualification {
        self.db.ordering_qualification()
    }
    fn truthy(entry: &Entry<T>) -> Option<&T> {
        match entry {
            Entry::Truthy(value) => Some(value),
            Entry::Falsey(_) => None,
        }
    }
    fn value(&self, ty: &TypeHandle) -> Option<&T> {
        self.entries
            .iter()
            .find(|(key, _)| key == ty)
            .and_then(|(_, entry)| Self::truthy(entry))
    }
    fn direct_and_ancestors<'a>(
        &'a self,
        ty: &'a TypeHandle,
    ) -> Result<impl Iterator<Item = &'a TypeHandle>, DispatchError> {
        let ancestors = self.db.proper_supertypes(ty)?;
        Ok(std::iter::once(ty).chain(ancestors))
    }
    /// Source expandToDerivedTypes: first truthy own/ancestor value, then default.
    pub fn most_derived(&self, ty: &TypeHandle) -> Result<Option<&T>, DispatchError> {
        let found = self
            .direct_and_ancestors(ty)?
            .find_map(|key| self.value(key));
        Ok(found.or_else(|| self.default.as_ref().and_then(Self::truthy)))
    }
    /// Complete generated-type map, omitting types without a truthy selection.
    pub fn expand_most_derived(&self) -> Result<Vec<(TypeHandle, &T)>, DispatchError> {
        let mut result = Vec::new();
        for ty in self.db.all_generated_types() {
            if let Some(value) = self.most_derived(ty)? {
                result.push((ty.clone(), value));
            }
        }
        Ok(result)
    }
    /// Source chain: own, ancestors, default; reverse the entire list if asked.
    /// Repeated payloads are retained. This does not invoke any payload.
    pub fn chain(&self, ty: &TypeHandle, order: ChainOrder) -> Result<Vec<&T>, DispatchError> {
        let mut result: Vec<_> = self
            .direct_and_ancestors(ty)?
            .filter_map(|key| self.value(key))
            .collect();
        if let Some(value) = self.default.as_ref().and_then(Self::truthy) {
            result.push(value);
        }
        if order == ChainOrder::SupertypeFirst {
            result.reverse();
        }
        Ok(result)
    }
    /// The parser's expanded map contains only known generated names. Symbols,
    /// undefined and unknown strings have no entry, even with a truthy default.
    pub fn construct_plan(&self, input: ConstructInput<'_>) -> Result<Option<&T>, DispatchError> {
        let ConstructInput::String(name) = input else {
            return Ok(None);
        };
        match self.db.resolve_type(name) {
            Ok(ty) => self.most_derived(&ty),
            Err(ReflectionError::UnknownType(_)) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}
impl<T> Registry<Vec<T>> {
    /// Source expandAndMerge: default is ignored. Empty arrays are truthy.
    /// Reverse prepends WHOLE blocks, keeping each block's element order.
    pub fn merge_blocks(&self, ty: &TypeHandle, reverse: bool) -> Result<Vec<&T>, DispatchError> {
        let mut blocks: Vec<_> = self
            .direct_and_ancestors(ty)?
            .filter_map(|key| self.value(key))
            .collect();
        if reverse {
            blocks.reverse();
        }
        Ok(blocks.into_iter().flat_map(|block| block.iter()).collect())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Postprocessor {
    FixOperatorExpression,
    AddLoopMember,
    FinalizeImport,
    CreateEmptyTransitionParameters,
    CreateMissingSuccessionEnds,
}
/// The parser's five pinned registrations, as identifiers rather than callbacks.
pub fn parser_registry(db: Arc<ReflectionDb>) -> Result<Registry<Postprocessor>, DispatchError> {
    let entries = [
        ("OperatorExpression", Postprocessor::FixOperatorExpression),
        ("WhileLoopActionUsage", Postprocessor::AddLoopMember),
        ("Import", Postprocessor::FinalizeImport),
        (
            "TransitionUsage",
            Postprocessor::CreateEmptyTransitionParameters,
        ),
        (
            "SuccessionAsUsage",
            Postprocessor::CreateMissingSuccessionEnds,
        ),
    ]
    .into_iter()
    .map(|(name, processor)| Ok((db.resolve_type(name)?, Entry::Truthy(processor))))
    .collect::<Result<Vec<_>, DispatchError>>()?;
    Registry::new(db, entries, None)
}
