// SPDX-License-Identifier: EPL-2.0
#![forbid(unsafe_code)]
mod admission;
mod search;
pub use admission::{
    AdmittedRouting, Edge, Error, Limit, Limits, Point, Route, Routing, Side, Unsupported, admit,
};
pub use search::route_admitted;

/// Admit once and search with the same spent-work and cancellation guard.
pub fn route<'a, F: FnMut() -> bool>(
    nodes: &[syson_layout::Placement<'a>],
    edges: &[Edge<'a>],
    limits: Limits,
    cancelled: F,
) -> Result<Routing<'a>, Error> {
    route_admitted(admit(nodes, edges, limits, cancelled)?)
}
