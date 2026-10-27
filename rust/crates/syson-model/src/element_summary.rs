// SPDX-License-Identifier: EPL-2.0
//! Borrowed views and the single traversal behind both summary APIs.
use super::{Document, ElementSummary, class, id};
use serde_json::Value;

/// A read-only view into this document's existing JSON strings. Collecting these
/// records allocates only the row vector, never copies a name, identity or owner.
/// Its lifetime prevents mutation/replacement of the borrowed document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElementSummaryRef<'a> {
    pub resource_id: &'a str,
    pub element_id: &'a str,
    pub class: &'a str,
    pub name: Option<&'a str>,
    pub owner: Option<&'a str>,
}
impl ElementSummaryRef<'_> {
    pub(super) fn into_owned(self) -> ElementSummary {
        ElementSummary {
            resource_id: self.resource_id.into(),
            element_id: self.element_id.into(),
            class: self.class.into(),
            name: self.name.map(str::to_owned),
            owner: self.owner.map(str::to_owned),
        }
    }
}
impl Document {
    /// Same order/class/name/semantic-owner rules as `elements`, with borrowed
    /// string fields. A Vec of fixed-size records is the only result ownership;
    /// this is not a bound on other consumers, rendering, or wall-clock work.
    pub fn element_summaries(&self) -> Vec<ElementSummaryRef<'_>> {
        let mut rows = Vec::new();
        visit(
            self.value["content"].as_array().unwrap(),
            None,
            false,
            &mut |row| rows.push(row),
        );
        rows
    }
}
// Preserve the original traversal exactly: direct related elements inherit the
// incoming owner; relationship rows have no semantic owner; their related
// elements receive the owner passed through the relationship's container.
pub(super) fn visit<'a>(
    nodes: &'a [Value],
    owner: Option<&'a str>,
    relationships: bool,
    output: &mut impl FnMut(ElementSummaryRef<'a>),
) {
    for node in nodes {
        output(ElementSummaryRef {
            resource_id: node["id"].as_str().unwrap(),
            element_id: id(node),
            class: class(node),
            name: node["data"]["declaredName"].as_str(),
            owner: if relationships { None } else { owner },
        });
        if let Some(related) = node["data"]["ownedRelatedElement"].as_array() {
            visit(related, owner, false, output);
        }
        if let Some(relations) = node["data"]["ownedRelationship"].as_array() {
            visit(relations, Some(id(node)), true, output);
        }
    }
}
