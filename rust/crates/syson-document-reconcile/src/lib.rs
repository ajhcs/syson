// Copyright (c) 2024, 2025, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
// Contributors: Obeo - original Java aggregate/entity; SysON Rust port contributors - pure planner.
//! Source-derived plans for SemanticData updates, without clocks, UUID allocation or I/O.
//! Inputs are caller-ordered observations of Java sets/lists, not a reconstruction of HashSet order.
#![forbid(unsafe_code)]
use std::collections::{BTreeSet, HashMap, HashSet};

/// UUID bytes; malformed/null Java identities are outside this typed boundary.
pub type Uuid = [u8; 16];

/// A nonnull, fully built document observation. Timestamps are opaque caller values.
/// Object identity is the address of this observation, never its UUID or field equality.
#[derive(Debug)]
pub struct Document<T> {
    pub id: Uuid,
    pub name: String,
    pub content: String,
    pub is_read_only: bool,
    pub created_on: T,
    pub last_modified_on: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Existing,
    Incoming,
}

/// Keep preserves the complete original instance, including timestamps and readOnly.
/// Rebuild requires a fresh instance and one fresh timestamp for both timestamp fields.
#[derive(Debug)]
pub enum DocumentAction<'a, T> {
    Keep {
        document: &'a Document<T>,
        origin: Origin,
    },
    Rebuild {
        id: Uuid,
        name: &'a str,
        content: &'a str,
        is_read_only: bool,
    },
}

/// An Unchanged plan leaves the original document collection and domains untouched.
#[derive(Debug)]
pub enum DocumentPlan<'a, T> {
    Unchanged,
    Replace {
        documents: Vec<DocumentAction<'a, T>>,
        domain_uris: BTreeSet<&'a str>,
    },
}

/// Ordered caller effects. UpdatedEvent needs a fresh event UUID, the aggregate
/// timestamp and caller cause; this library neither allocates nor publishes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    RebuiltDocumentTimestamp { output_index: usize },
    AggregateTimestamp,
    UpdatedEvent,
}

impl<T> DocumentPlan<'_, T> {
    pub fn effects(&self) -> Vec<Effect> {
        match self {
            Self::Unchanged => Vec::new(),
            Self::Replace { documents, .. } => {
                let mut effects = documents
                    .iter()
                    .enumerate()
                    .filter_map(|(i, item)| {
                        matches!(item, DocumentAction::Rebuild { .. })
                            .then_some(Effect::RebuiltDocumentTimestamp { output_index: i })
                    })
                    .collect::<Vec<_>>();
                effects.extend([Effect::AggregateTimestamp, Effect::UpdatedEvent]);
                effects
            }
        }
    }
}

fn distinct_instances<'a, T>(documents: &[&'a Document<T>]) -> Vec<&'a Document<T>> {
    let mut identities = HashSet::with_capacity(documents.len());
    documents
        .iter()
        .copied()
        .filter(|document| identities.insert(std::ptr::from_ref(*document)))
        .collect()
}

/// Plan updateDocuments in the supplied source iteration order. Duplicate UUIDs
/// are admitted; repeated pointers model one set entry. Domains have set values,
/// not an asserted source HashSet iteration order. Inputs are never modified.
pub fn plan_documents<'a, T>(
    existing: &[&'a Document<T>],
    incoming: &[&'a Document<T>],
    requested_domains: &'a [String],
) -> DocumentPlan<'a, T> {
    let existing = distinct_instances(existing);
    let incoming = distinct_instances(incoming);
    // Indexes are used only for membership/lookup. Source observation order
    // remains in the vectors; duplicate UUIDs retain the first existing object.
    let mut by_id = HashMap::with_capacity(existing.len());
    for document in &existing {
        by_id.entry(document.id).or_insert(*document);
    }
    let mut kept = HashSet::with_capacity(incoming.len());
    let mut output = Vec::new();
    let mut changed = false;
    for document in incoming {
        let action = match by_id.get(&document.id).copied() {
            Some(prior) if prior.name == document.name && prior.content == document.content => {
                DocumentAction::Keep {
                    document: prior,
                    origin: Origin::Existing,
                }
            }
            Some(prior) => {
                changed = true;
                DocumentAction::Rebuild {
                    id: prior.id,
                    name: &document.name,
                    content: &document.content,
                    is_read_only: false, // The source builder deliberately does not copy this flag.
                }
            }
            None => {
                changed = true;
                DocumentAction::Keep {
                    document,
                    origin: Origin::Incoming,
                }
            }
        };
        // LinkedHashSet<Document> deduplicates kept instances by object identity.
        // Each Rebuild is a fresh object even when all its values match another.
        let duplicate = match &action {
            DocumentAction::Keep { document, .. } => !kept.insert(std::ptr::from_ref(*document)),
            DocumentAction::Rebuild { .. } => false,
        };
        if !duplicate {
            output.push(action);
        }
    }
    let old_ids = existing
        .iter()
        .map(|document| document.id)
        .collect::<BTreeSet<_>>();
    let new_ids = output
        .iter()
        .map(|item| match item {
            DocumentAction::Keep { document, .. } => document.id,
            DocumentAction::Rebuild { id, .. } => *id,
        })
        .collect::<BTreeSet<_>>();
    changed |= old_ids != new_ids;
    if changed {
        DocumentPlan::Replace {
            documents: output,
            domain_uris: requested_domains.iter().map(String::as_str).collect(),
        }
    } else {
        DocumentPlan::Unchanged
    }
}

/// Admission for the exact list operation, supplied from the caller's observed
/// collection implementation. It must not be inferred from an immutable slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationAdmission {
    Allowed,
    ThrowsUnsupportedOperation,
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyError {
    UnsupportedMutation,
    UnresolvedMutationCapability,
}

impl MutationAdmission {
    fn require(self) -> Result<(), DependencyError> {
        match self {
            Self::Allowed => Ok(()),
            Self::ThrowsUnsupportedOperation => Err(DependencyError::UnsupportedMutation),
            Self::Unresolved => Err(DependencyError::UnresolvedMutationCapability),
        }
    }
}

/// References preserve original wrappers and list order. Equality of reference
/// wrappers is caller supplied for removal; it is not inferred from UUIDs.
#[derive(Debug)]
pub struct DependencyPlan<'a, R> {
    pub references: Vec<&'a R>,
    pub request_updated_event: bool,
}

impl<R> DependencyPlan<'_, R> {
    pub fn effects(&self) -> Vec<Effect> {
        if self.request_updated_event {
            vec![Effect::AggregateTimestamp, Effect::UpdatedEvent]
        } else {
            Vec::new()
        }
    }
}

/// The filter checks only the original list. New duplicates in the same batch
/// remain duplicates. No addition means no addAll call or mutation admission.
pub fn plan_add_dependencies<'a, R>(
    existing: &[&'a R],
    incoming: &[&'a R],
    id: impl Fn(&R) -> Uuid,
    add_all: MutationAdmission,
) -> Result<DependencyPlan<'a, R>, DependencyError> {
    let mut references = existing.to_vec();
    for candidate in incoming {
        if !existing.iter().any(|current| id(current) == id(candidate)) {
            references.push(*candidate);
        }
    }
    let request_updated_event = references.len() != existing.len();
    if request_updated_event {
        add_all.require()?;
    }
    Ok(DependencyPlan {
        references,
        request_updated_event,
    })
}

/// same_reference(existing, removal) supplies the source contains direction.
/// removeIf is called even for no-op input. Unknown/rejected mutation yields no
/// successful plan or timestamp/event request, including for an empty list.
pub fn plan_remove_dependencies<'a, R>(
    existing: &[&'a R],
    removals: &[&R],
    same_reference: impl Fn(&R, &R) -> bool,
    remove_if: MutationAdmission,
) -> Result<DependencyPlan<'a, R>, DependencyError> {
    remove_if.require()?;
    Ok(DependencyPlan {
        references: existing
            .iter()
            .copied()
            .filter(|current| {
                !removals
                    .iter()
                    .any(|removal| same_reference(current, removal))
            })
            .collect(),
        request_updated_event: true,
    })
}
