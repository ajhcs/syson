/*******************************************************************************
 * Copyright (c) 2024, 2026 Obeo.
 * This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License v2.0
 * which accompanies this distribution, and is available at
 * https://www.eclipse.org/legal/epl-2.0/
 *
 * SPDX-License-Identifier: EPL-2.0
 *
 * Contributors:
 *     Obeo - initial API and implementation
 *******************************************************************************/
//! One lookup algorithm from pinned ElementUtil, with independently supplied
//! containment, derived ownership, names and import facts. Adapters retain their
//! own error type so missing facts are not silently treated as absent elements.
use crate::{model::ElementId, names::parse_qualified_name};

/// Lazy facts consumed by ElementUtil's ordered first-match algorithm.
///
/// `raw_children` must describe an admitted finite acyclic traversal. Native
/// adapters supply actual eAllContents preorder; the historical Model adapter
/// retains its explicitly projected traversal contract. `owned_elements` is
/// the distinct derived getOwnedElement relation, including source ordering and
/// flattening of all relevant Relationships in native adapters. These lists
/// must be supplied independently when their native meanings differ.
///
/// Names are effective getters, not necessarily declaredName/declaredShortName.
/// `is_unnamed` tests getName()==null, including for the top-level Namespace
/// special case. `has_raw_container` tests eContainer rather than semantic owner.
pub trait QueryGraph {
    type Error;

    fn raw_children(&self, id: ElementId) -> Result<Vec<ElementId>, Self::Error>;
    fn owned_elements(&self, id: ElementId) -> Result<Vec<ElementId>, Self::Error>;
    fn is_namespace(&self, id: ElementId) -> Result<bool, Self::Error>;
    fn has_raw_container(&self, id: ElementId) -> Result<bool, Self::Error>;
    fn is_unnamed(&self, id: ElementId) -> Result<bool, Self::Error>;
    fn name_matches(&self, id: ElementId, name: &str) -> Result<bool, Self::Error>;
    /// Already ordered PUBLIC imported Membership.relatedElement facts only.
    /// Called only when no directly owned element matches the current segment.
    fn public_imported_elements(&self, id: ElementId) -> Result<Vec<ElementId>, Self::Error>;

    /// Adapters may override this when resolving related elements itself has
    /// effects or failures: later imported targets must remain unevaluated once
    /// the first matching target is found, as with the upstream Java stream.
    fn find_public_imported_match(
        &self,
        id: ElementId,
        name: &str,
    ) -> Result<Option<ElementId>, Self::Error> {
        for imported in self.public_imported_elements(id)? {
            if self.name_matches(imported, name)? {
                return Ok(Some(imported));
            }
        }
        Ok(None)
    }
}

/// Search caller-supplied resource roots in order. Empty roots mean no search;
/// there is no implicit fallback to the context object's own resource/root.
/// Java-compatible split and optional-name-quote behavior stay in the shared
/// names module and the adapter's name_matches getter.
pub fn find_by_name_and_type<G: QueryGraph>(
    graph: &G,
    roots: &[ElementId],
    name: &str,
    accepts: impl Fn(ElementId) -> Result<bool, G::Error>,
) -> Result<Option<ElementId>, G::Error> {
    let parts = parse_qualified_name(name);
    for &root in roots {
        let found = if parts.len() > 1 {
            find_qualified_root(graph, root, &parts, &accepts)?
        } else {
            find_unqualified(graph, root, name, &accepts)?
        };
        if found.is_some() {
            return Ok(found);
        }
    }
    Ok(None)
}

fn find_unqualified<G: QueryGraph>(
    graph: &G,
    root: ElementId,
    name: &str,
    accepts: &impl Fn(ElementId) -> Result<bool, G::Error>,
) -> Result<Option<ElementId>, G::Error> {
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        // Source type.isInstance short-circuits both effective-name getters.
        if accepts(id)? && graph.name_matches(id, name)? {
            return Ok(Some(id));
        }
        stack.extend(graph.raw_children(id)?.into_iter().rev());
    }
    Ok(None)
}

fn find_qualified_root<G: QueryGraph>(
    graph: &G,
    root: ElementId,
    parts: &[&str],
    accepts: &impl Fn(ElementId) -> Result<bool, G::Error>,
) -> Result<Option<ElementId>, G::Error> {
    if graph.is_namespace(root)? && !graph.has_raw_container(root)? && graph.is_unnamed(root)? {
        find_qualified_child(graph, root, parts, accepts)
    } else if graph.name_matches(root, parts[0])? {
        find_qualified_child(graph, root, &parts[1..], accepts)
    } else {
        Ok(None)
    }
}

fn find_qualified_child<G: QueryGraph>(
    graph: &G,
    parent: ElementId,
    parts: &[&str],
    accepts: &impl Fn(ElementId) -> Result<bool, G::Error>,
) -> Result<Option<ElementId>, G::Error> {
    let Some((head, tail)) = parts.split_first() else {
        return Ok(None);
    };
    let mut selected = None;
    for child in graph.owned_elements(parent)? {
        if graph.name_matches(child, head)? {
            selected = Some(child);
            break;
        }
    }
    if selected.is_none() && graph.is_namespace(parent)? {
        selected = graph.find_public_imported_match(parent, head)?;
    }
    match selected {
        Some(id) if !tail.is_empty() => find_qualified_child(graph, id, tail, accepts),
        Some(id) if accepts(id)? => Ok(Some(id)),
        _ => Ok(None),
    }
}
