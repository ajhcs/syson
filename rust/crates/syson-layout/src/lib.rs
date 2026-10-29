// SPDX-License-Identifier: EPL-2.0
#![forbid(unsafe_code)]
mod admission;
mod placement;
mod types;
pub use admission::{AdmittedGraph, Guard, PreparedGraph, admit};
pub use placement::{ClassifiedEdge, Layout, Placement, place};
pub use types::*;

/// Admit once, then place using the same canonical buffers and continued guard.
pub fn layout<'a>(
    nodes: &[MeasuredNode<'a>],
    edges: &[DirectedEdge<'a>],
    options: Options,
    limits: Limits,
    cancelled: impl FnMut() -> bool,
) -> Result<Layout<'a>, LayoutError> {
    admit(nodes, edges, options, limits, cancelled).and_then(place)
}
