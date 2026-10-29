// SPDX-License-Identifier: EPL-2.0
//! Finite package forests; native/UI acceptance remains open.
mod admission;
pub use admission::{
    AdmittedForest, Error, PackageLayout, PackageNode, PackagePlacement, Position, Size,
    admit_packages,
};
mod placement;
pub use placement::place_packages;

pub fn layout_packages<'a, F: FnMut() -> bool>(
    nodes: &[PackageNode<'a>],
    edges: &[crate::DirectedEdge<'a>],
    options: crate::Options,
    limits: crate::Limits,
    cancelled: F,
) -> Result<PackageLayout<'a>, Error> {
    place_packages(admit_packages(nodes, edges, options, limits, cancelled)?)
}
