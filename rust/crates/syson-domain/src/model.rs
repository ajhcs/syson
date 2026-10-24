// Copyright (c) 2024, 2026 Obeo.
// This program and accompanying materials are made available under the terms
// of the Eclipse Public License v2.0, https://www.eclipse.org/legal/epl-2.0/.
// SPDX-License-Identifier: EPL-2.0
// Contributors: Obeo - initial API and implementation
// Rust translation and compatibility tests: SysON Rust port contributors.

//! Explicit in-memory ownership projection for ElementImpl / ElementUtil queries.
//! This projects ownership, names, and imported memberships only. Class identities
//! are supplied by adapters: it does not pretend to implement EMF inheritance.
//! Inputs must preflatten non-Membership Relationship wrappers into semantic
//! owner/owned-element links. Only Membership wrappers are recognized here.
//! Raw EMF containment is NOT an accepted lossless input format.
use crate::names::{name_matches, printable_name};
use crate::query::{self, QueryGraph};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementId(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
}
#[derive(Debug, Clone)]
pub struct ImportedMembership {
    pub visibility: Visibility,
    pub related: Vec<ElementId>,
}
#[derive(Debug, Clone)]
pub struct Element {
    pub class: String,
    pub declared_name: Option<String>,
    pub declared_short_name: Option<String>,
    pub is_namespace: bool,
    pub is_membership: bool,
    pub imports: Vec<ImportedMembership>,
    owner: Option<ElementId>,
    children: Vec<ElementId>,
}
impl Element {
    pub fn new(class: impl Into<String>, name: Option<&str>) -> Self {
        Self {
            class: class.into(),
            declared_name: name.map(str::to_owned),
            declared_short_name: None,
            is_namespace: false,
            is_membership: false,
            imports: Vec::new(),
            owner: None,
            children: Vec::new(),
        }
    }
    pub fn owner(&self) -> Option<ElementId> {
        self.owner
    }
    pub fn projected_children(&self) -> &[ElementId] {
        &self.children
    }
    pub fn effective_name(&self) -> Option<&str> {
        self.declared_name.as_deref()
    }
    pub fn effective_short_name(&self) -> Option<&str> {
        self.declared_short_name.as_deref()
    }
    pub fn escaped_name(&self) -> String {
        printable_name(self.effective_name().or(self.effective_short_name()), true)
    }
    fn matches(&self, query: &str) -> bool {
        name_matches(self.effective_name(), Some(query))
            || name_matches(self.effective_short_name(), Some(query))
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    MissingElement(ElementId),
    OwnershipCycle,
}
#[derive(Debug, Default)]
pub struct Model {
    elements: Vec<Element>,
    roots: Vec<ElementId>,
}
impl Model {
    /// Adapter safety boundary: owner must already exist; IDs are process-local.
    pub fn insert(
        &mut self,
        mut element: Element,
        owner: Option<ElementId>,
    ) -> Result<ElementId, ModelError> {
        if let Some(owner) = owner {
            self.get(owner)?;
        }
        let id = ElementId(self.elements.len());
        element.owner = owner;
        element.children.clear();
        self.elements.push(element);
        match owner {
            Some(owner) => self.elements[owner.0].children.push(id),
            None => self.roots.push(id),
        }
        Ok(id)
    }
    pub fn get(&self, id: ElementId) -> Result<&Element, ModelError> {
        self.elements
            .get(id.0)
            .ok_or(ModelError::MissingElement(id))
    }
    /// Compatibility output follows actual ElementImpl.getQualifiedName, not
    /// the commented specification implementation. Skip one Membership owner.
    pub fn qualified_name(&self, id: ElementId) -> Result<Option<String>, ModelError> {
        let mut path = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut cursor = Some(id);
        while let Some(id) = cursor {
            if !visited.insert(id) {
                return Err(ModelError::OwnershipCycle);
            }
            let element = self.get(id)?;
            let name = printable_name(element.effective_name(), true);
            if name.is_empty() {
                break;
            }
            path.push(name);
            cursor = element.owner;
            if let Some(owner) = cursor {
                let parent = self.get(owner)?;
                if parent.is_membership {
                    cursor = parent.owner;
                }
            }
        }
        path.reverse();
        Ok(if path.is_empty() {
            None
        } else {
            Some(path.join("::"))
        })
    }
    /// Upstream type.isInstance is supplied by the adapter. No Rust-side
    /// metamodel subtyping is silently inferred from class name strings.
    pub fn find_by_name_and_type(
        &self,
        name: &str,
        accepts: impl Fn(&Element) -> bool,
    ) -> Result<Option<ElementId>, ModelError> {
        query::find_by_name_and_type(self, &self.roots, name, |id| Ok(accepts(self.get(id)?)))
    }
}

/// Historical projection adapter. Native callers provide QueryGraph facts
/// directly; they must not reinterpret these projected children as raw EMF.
impl QueryGraph for Model {
    type Error = ModelError;

    fn raw_children(&self, id: ElementId) -> Result<Vec<ElementId>, ModelError> {
        Ok(self.get(id)?.children.clone())
    }

    fn is_namespace(&self, id: ElementId) -> Result<bool, ModelError> {
        Ok(self.get(id)?.is_namespace)
    }

    fn has_raw_container(&self, id: ElementId) -> Result<bool, ModelError> {
        Ok(self.get(id)?.owner.is_some())
    }

    fn is_unnamed(&self, id: ElementId) -> Result<bool, ModelError> {
        Ok(self.get(id)?.effective_name().is_none())
    }

    fn name_matches(&self, id: ElementId, name: &str) -> Result<bool, ModelError> {
        Ok(self.get(id)?.matches(name))
    }

    fn owned_elements(&self, parent: ElementId) -> Result<Vec<ElementId>, ModelError> {
        let e = self.get(parent)?;
        // getOwnedElement is an EMF derived relation, not literal eContents:
        // membership wrappers are flattened by the projection adapter below.
        let mut candidates = Vec::new();
        for id in &e.children {
            let child = self.get(*id)?;
            if child.is_membership {
                candidates.extend(child.children.iter().copied());
            } else {
                candidates.push(*id);
            }
        }
        Ok(candidates)
    }

    fn public_imported_elements(&self, id: ElementId) -> Result<Vec<ElementId>, ModelError> {
        Ok(self
            .get(id)?
            .imports
            .iter()
            .filter(|import| import.visibility == Visibility::Public)
            .flat_map(|import| import.related.iter().copied())
            .collect())
    }
}
