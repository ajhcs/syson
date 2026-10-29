// Copyright (c) 2022-2025 Sensmetry UAB and others.
// SPDX-License-Identifier: EPL-2.0 OR GPL-2.0 WITH Classpath-exception-2.0
//! Immutable, owner-qualified reflection of the 189 retained Syside AST names.
//! Ordered tables use an explicit source-derived short-sort policy. They have
//! not been qualified against the original JavaScript runtime. This crate does
//! not construct ASTs, apply defaults, link references or validate semantics.
#![forbid(unsafe_code)]
use std::cmp::Ordering;
use std::sync::Arc;
mod generated;

#[derive(Debug, Clone)]
pub struct TypeHandle {
    owner: Arc<()>,
    index: usize,
}
impl PartialEq for TypeHandle {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && Arc::ptr_eq(&self.owner, &other.owner)
    }
}
impl Eq for TypeHandle {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MandatoryKind {
    Array,
    Boolean,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MandatoryProperty {
    pub name: &'static str,
    pub kind: MandatoryKind,
    /// None means the override's final `$children` property.
    pub declared_by: Option<TypeHandle>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReflectionError {
    UnknownType(String),
    ForeignTable,
    InvalidReferencePosition,
    UnknownReferenceProperty(String),
}
/// This is an explicit candidate policy, not a runtime-parity assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderingQualification {
    SourceDerivedStableShortSortCandidate,
}
#[derive(Debug, Clone)]
pub struct ReferencePosition {
    container: TypeHandle,
    index: usize,
    length: usize,
}
struct Row {
    name: &'static str,
    parents: &'static [usize],
    non_type: bool,
    mandatory: &'static [(&'static str, MandatoryKind)],
}

/// A database and every handle share a unique allocation. Retaining a handle
/// retains that owner, preventing its identity from being recycled underneath it.
/// All publicly reachable table values are immutable after construction.
#[derive(Debug)]
pub struct ReflectionDb {
    owner: Arc<()>,
    all: Vec<TypeHandle>,
    ast: Vec<TypeHandle>,
    ancestors: Vec<Vec<TypeHandle>>,
    descendants: Vec<Vec<TypeHandle>>,
    mandatory: Vec<Vec<MandatoryProperty>>,
}
impl Default for ReflectionDb {
    fn default() -> Self {
        Self::pinned()
    }
}
impl ReflectionDb {
    pub const SOURCE_BASELINE: &str = "6ba2ac64e6b16ee2b05752ad55395eebb87d4910";
    pub const SOURCE_CANDIDATE: &str = "1326f234cae23b0e4777d5a48a069b17e9e332f1";
    pub const MAX_PINNED_SORT_PARTITION: usize = 13;

    pub fn pinned() -> Self {
        let rows = generated::ROWS;
        let owner = Arc::new(());
        let all: Vec<_> = (0..rows.len())
            .map(|index| TypeHandle {
                owner: owner.clone(),
                index,
            })
            .collect();
        let mut closure = vec![vec![false; rows.len()]; rows.len()];
        for (i, row) in rows.iter().enumerate() {
            for parent in row.parents {
                closure[i][*parent] = true;
            }
        }
        for through in 0..rows.len() {
            let indirect = closure[through].clone();
            for row in &mut closure {
                if row[through] {
                    for (edge, reachable) in row.iter_mut().zip(&indirect) {
                        *edge |= reachable;
                    }
                }
            }
        }
        let subtype = |a: usize, b: usize| a == b || closure[a][b];
        let namespace = rows.iter().position(|r| r.name == "Namespace").unwrap();
        let mut ancestor_indexes = Vec::with_capacity(rows.len());
        for (i, reachable) in closure.iter().enumerate() {
            let (mut unions, mut types, mut relationships) = (Vec::new(), Vec::new(), Vec::new());
            for (parent, row) in rows.iter().enumerate() {
                if i != parent && reachable[parent] {
                    if row.non_type {
                        unions.push(parent);
                    } else if subtype(parent, namespace) {
                        types.push(parent);
                    } else {
                        relationships.push(parent);
                    }
                }
            }
            let compare = |a: usize, b: usize| {
                if subtype(a, b) {
                    Ordering::Less
                } else if subtype(b, a) {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            };
            for partition in [&mut unions, &mut types, &mut relationships] {
                assert!(
                    partition.len() <= Self::MAX_PINNED_SORT_PARTITION,
                    "pinned partition drift"
                );
                stable_short_sort(partition, &compare);
            }
            for other in [relationships, unions] {
                let mut positions: Vec<_> = other
                    .into_iter()
                    .map(|item| {
                        (
                            types
                                .iter()
                                .position(|existing| !subtype(*existing, item))
                                .unwrap_or(types.len()),
                            item,
                        )
                    })
                    .collect();
                // Numeric insertion positions form a total order, unlike the
                // subtype comparator. Stable ties retain `other` order.
                positions.sort_by_key(|(position, _)| *position);
                for (inserted, (position, item)) in positions.into_iter().enumerate() {
                    types.insert(position + inserted, item);
                }
            }
            ancestor_indexes.push(types);
        }
        let ancestors = ancestor_indexes
            .iter()
            .map(|indexes| indexes.iter().map(|i| all[*i].clone()).collect())
            .collect();
        // Reverse subtype sets receive names in generated outer-loop order.
        let descendants = (0..rows.len())
            .map(|parent| {
                (0..rows.len())
                    .filter(|child| *child != parent && closure[*child][parent])
                    .map(|child| all[child].clone())
                    .collect()
            })
            .collect();
        let mandatory = ancestor_indexes
            .iter()
            .enumerate()
            .map(|(index, bases)| {
                let mut result: Vec<MandatoryProperty> = Vec::new();
                for declaring in std::iter::once(&index).chain(bases) {
                    for &(name, kind) in rows[*declaring].mandatory {
                        if !result.iter().any(|p| p.name == name) {
                            result.push(MandatoryProperty {
                                name,
                                kind,
                                declared_by: Some(all[*declaring].clone()),
                            });
                        }
                    }
                }
                result.push(MandatoryProperty {
                    name: "$children",
                    kind: MandatoryKind::Array,
                    declared_by: None,
                });
                result
            })
            .collect();
        let ast = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.non_type)
            .map(|(i, _)| all[i].clone())
            .collect();
        Self {
            owner,
            all,
            ast,
            ancestors,
            descendants,
            mandatory,
        }
    }
    pub fn ordering_qualification(&self) -> OrderingQualification {
        OrderingQualification::SourceDerivedStableShortSortCandidate
    }
    pub fn resolve_type(&self, name: &str) -> Result<TypeHandle, ReflectionError> {
        generated::ROWS
            .iter()
            .position(|r| r.name == name)
            .map(|i| self.all[i].clone())
            .ok_or_else(|| ReflectionError::UnknownType(name.into()))
    }
    fn index(&self, ty: &TypeHandle) -> Result<usize, ReflectionError> {
        if !Arc::ptr_eq(&self.owner, &ty.owner) {
            return Err(ReflectionError::ForeignTable);
        }
        Ok(ty.index)
    }
    pub fn name(&self, ty: &TypeHandle) -> Result<&'static str, ReflectionError> {
        Ok(generated::ROWS[self.index(ty)?].name)
    }
    pub fn all_generated_types(&self) -> &[TypeHandle] {
        &self.all
    }
    pub fn ast_types_filtered(&self) -> &[TypeHandle] {
        &self.ast
    }
    pub fn proper_supertypes(&self, ty: &TypeHandle) -> Result<&[TypeHandle], ReflectionError> {
        Ok(&self.ancestors[self.index(ty)?])
    }
    pub fn proper_subtypes(&self, ty: &TypeHandle) -> Result<&[TypeHandle], ReflectionError> {
        Ok(&self.descendants[self.index(ty)?])
    }
    pub fn is_subtype(&self, sub: &TypeHandle, sup: &TypeHandle) -> Result<bool, ReflectionError> {
        let a = self.index(sub)?;
        let b = self.index(sup)?;
        Ok(a == b || self.ancestors[a].iter().any(|ty| ty.index == b))
    }
    pub fn effective_mandatory(
        &self,
        ty: &TypeHandle,
    ) -> Result<&[MandatoryProperty], ReflectionError> {
        Ok(&self.mandatory[self.index(ty)?])
    }
    /// Generated-base property checking is distinct from the positional override.
    pub fn generated_reference_target(
        &self,
        property_id: &str,
    ) -> Result<TypeHandle, ReflectionError> {
        if !generated::REFERENCE_IDS.contains(&property_id) {
            return Err(ReflectionError::UnknownReferenceProperty(
                property_id.into(),
            ));
        }
        self.resolve_type("Element")
    }
    /// usize plus explicit nonempty bounds is deliberately narrower than the
    /// original dynamic index domain. The override ignores the property name.
    pub fn reference_position(
        &self,
        container: &TypeHandle,
        index: usize,
        length: usize,
    ) -> Result<ReferencePosition, ReflectionError> {
        self.index(container)?;
        if index >= length {
            return Err(ReflectionError::InvalidReferencePosition);
        }
        Ok(ReferencePosition {
            container: container.clone(),
            index,
            length,
        })
    }
    pub fn reference_target_type(
        &self,
        position: &ReferencePosition,
    ) -> Result<TypeHandle, ReflectionError> {
        let name = self.name(&position.container)?;
        let target = if position.index == position.length - 1 {
            match name {
                "TypeReference" => "Type",
                "ClassifierReference" => "Classifier",
                "MetaclassReference" => "Metaclass",
                "MembershipReference" => "Membership",
                "NamespaceReference" => "Namespace",
                "FeatureReference" => "Feature",
                "ConjugatedPortReference" => "ConjugatedPortDefinition",
                _ => "Element",
            }
        } else {
            "Element"
        };
        self.resolve_type(target)
    }
}

/// Frozen stable short-sort policy: detect/reverse the first strict descending
/// run (or retain a nondecreasing run), then insert remaining items using an
/// upper-bound binary search. Do not substitute generic sort with this partial
/// comparator, and do not describe this policy as proven JavaScript ordering.
fn stable_short_sort(values: &mut [usize], compare: &impl Fn(usize, usize) -> Ordering) {
    if values.len() < 2 {
        return;
    }
    let descending = compare(values[1], values[0]) == Ordering::Less;
    let mut sorted = 2;
    while sorted < values.len()
        && if descending {
            compare(values[sorted], values[sorted - 1]) == Ordering::Less
        } else {
            compare(values[sorted], values[sorted - 1]) != Ordering::Less
        }
    {
        sorted += 1;
    }
    if descending {
        values[..sorted].reverse();
    }
    for end in sorted..values.len() {
        let pivot = values[end];
        let (mut low, mut high) = (0, end);
        while low < high {
            let middle = low + (high - low) / 2;
            if compare(pivot, values[middle]) == Ordering::Less {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        values[low..=end].rotate_right(1);
    }
}
