// SPDX-License-Identifier: EPL-2.0
//! Canonical flat-graph admission; not ELK or full application qualification.
use crate::{DirectedEdge, LayoutError, Limit, Limits, MeasuredNode, Options, Unsupported};
use std::fmt;
use std::ops::Deref;

#[derive(Debug)]
pub struct PreparedGraph<'g, 'a> {
    pub(crate) nodes: &'g [MeasuredNode<'a>],
    pub(crate) edges: &'g [DirectedEdge<'a>],
    pub(crate) options: Options,
    pub(crate) limits: Limits,
    pub(crate) order: Vec<usize>,
    pub(crate) forward: Vec<Vec<usize>>,
    pub(crate) reverse: Vec<Vec<usize>>,
    pub(crate) indexed: Vec<(usize, usize, usize)>,
    pub(crate) maximum_id: usize,
    pub(crate) memory: usize,
}
impl<'g, 'a> PreparedGraph<'g, 'a> {
    pub fn nodes(&self) -> &'g [MeasuredNode<'a>] {
        self.nodes
    }
    pub fn edges(&self) -> &'g [DirectedEdge<'a>] {
        self.edges
    }
    pub fn options(&self) -> Options {
        self.options
    }
    pub fn limits(&self) -> Limits {
        self.limits
    }
    pub fn node_order(&self) -> &[usize] {
        &self.order
    }
    pub fn forward(&self) -> &[Vec<usize>] {
        &self.forward
    }
    pub fn reverse(&self) -> &[Vec<usize>] {
        &self.reverse
    }
    /// Canonical source/target node indices and the original input edge index.
    pub fn indexed_edges(&self) -> &[(usize, usize, usize)] {
        &self.indexed
    }
    pub fn maximum_id_bytes(&self) -> usize {
        self.maximum_id
    }
    pub fn estimated_workspace_bytes(&self) -> usize {
        self.memory
    }
}
/// Immutable borrowed graph plus its original callback and spent-work state.
pub struct AdmittedGraph<'g, 'a, F> {
    graph: PreparedGraph<'g, 'a>,
    guard: Guard<F>,
}
impl<F: FnMut() -> bool> AdmittedGraph<'_, '_, F> {
    pub fn charged_work(&self) -> usize {
        self.guard.used()
    }
}
impl<'g, 'a, F> AdmittedGraph<'g, 'a, F> {
    /// Consume admission once; the opaque guard has no constructor or reset API.
    pub fn into_parts(self) -> (PreparedGraph<'g, 'a>, Guard<F>) {
        (self.graph, self.guard)
    }
}
impl<'g, 'a, F> Deref for AdmittedGraph<'g, 'a, F> {
    type Target = PreparedGraph<'g, 'a>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl<F: FnMut() -> bool> fmt::Debug for AdmittedGraph<'_, '_, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AdmittedGraph")
            .field("nodes", &self.graph.nodes.len())
            .field("edges", &self.graph.edges.len())
            .field("charged_work", &self.guard.used())
            .finish_non_exhaustive()
    }
}
pub struct Guard<F> {
    cancelled: F,
    used: usize,
    maximum: usize,
}
impl<F: FnMut() -> bool> Guard<F> {
    pub fn used(&self) -> usize {
        self.used
    }
    /// Charge logical work and poll the original cancellation callback.
    pub fn tick(&mut self, units: usize) -> Result<(), LayoutError> {
        if (self.cancelled)() {
            return Err(LayoutError::Cancelled);
        }
        self.used = self
            .used
            .checked_add(units)
            .filter(|n| *n <= self.maximum)
            .ok_or(LayoutError::Limit(Limit::Work))?;
        Ok(())
    }
    pub(crate) fn sort<T>(
        &mut self,
        values: &mut [T],
        key_bytes: usize,
        compare: impl Fn(&T, &T) -> std::cmp::Ordering,
    ) -> Result<(), LayoutError> {
        let charge = values
            .len()
            .checked_mul(values.len())
            .and_then(|n| n.checked_mul(key_bytes.max(1)))
            .ok_or(LayoutError::Limit(Limit::Work))?;
        self.tick(charge)?;
        values.sort_unstable_by(compare);
        self.tick(0)
    }
}
pub(crate) fn filled<T: Clone>(length: usize, value: T) -> Result<Vec<T>, LayoutError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(length)
        .map_err(|_| LayoutError::Allocation)?;
    result.resize(length, value);
    Ok(result)
}
pub(crate) fn push<T>(values: &mut Vec<T>, value: T) -> Result<(), LayoutError> {
    values.try_reserve(1).map_err(|_| LayoutError::Allocation)?;
    values.push(value);
    Ok(())
}
fn valid(number: f64) -> bool {
    number.is_finite() && number >= 0.0
}

/// Borrow inputs unchanged, preserving canonical indices and all donor charges.
/// Workspace is the logical 1024*N+640*E estimate, not heap/RSS or a CPU deadline.
/// Containers, ports and manual pins remain typed unsupported branches.
pub fn admit<'g, 'a, F: FnMut() -> bool>(
    nodes: &'g [MeasuredNode<'a>],
    edges: &'g [DirectedEdge<'a>],
    options: Options,
    limits: Limits,
    cancelled: F,
) -> Result<AdmittedGraph<'g, 'a, F>, LayoutError> {
    let mut guard = Guard {
        cancelled,
        used: 0,
        maximum: limits.max_work,
    };
    guard.tick(0)?;
    let (n, m) = (nodes.len(), edges.len());
    if n > limits.max_nodes {
        return Err(LayoutError::Limit(Limit::Nodes));
    }
    if m > limits.max_edges {
        return Err(LayoutError::Limit(Limit::Edges));
    }
    let memory = n
        .checked_mul(1024)
        .and_then(|v| m.checked_mul(640).and_then(|e| v.checked_add(e)))
        .ok_or(LayoutError::Limit(Limit::Memory))?;
    if memory > limits.max_workspace_bytes {
        return Err(LayoutError::Limit(Limit::Memory));
    }
    if ![
        options.layer_gap,
        options.node_gap,
        options.component_gap,
        options.padding,
    ]
    .into_iter()
    .all(valid)
    {
        return Err(LayoutError::InvalidNumber);
    }
    // Precharge fixed buffer initialization/bookkeeping before allocations.
    guard.tick(
        n.checked_mul(32)
            .and_then(|v| m.checked_mul(8).and_then(|e| v.checked_add(e)))
            .ok_or(LayoutError::Limit(Limit::Work))?,
    )?;
    let mut maximum_id = 1;
    for id in nodes
        .iter()
        .map(|n| n.id)
        .chain(edges.iter().flat_map(|e| [e.id, e.source, e.target]))
    {
        guard.tick(1)?;
        if id.is_empty() {
            return Err(LayoutError::EmptyId);
        }
        if id.len() > limits.max_id_bytes {
            return Err(LayoutError::Limit(Limit::IdBytes));
        }
        maximum_id = maximum_id.max(id.len());
    }
    let mut order = filled(n, 0)?;
    for (i, value) in order.iter_mut().enumerate() {
        *value = i;
    }
    guard.sort(&mut order, maximum_id, |a, b| {
        nodes[*a].id.cmp(nodes[*b].id)
    })?;
    for (position, index) in order.iter().enumerate() {
        guard.tick(1)?;
        let node = nodes[*index];
        if position > 0 && node.id == nodes[order[position - 1]].id {
            return Err(LayoutError::DuplicateNode);
        }
        if !valid(node.width) || !valid(node.height) {
            return Err(LayoutError::InvalidNumber);
        }
        if node.parent.is_some() {
            return Err(LayoutError::Unsupported(Unsupported::Container));
        }
        if node.port_count > 0 {
            return Err(LayoutError::Unsupported(Unsupported::Ports));
        }
        if node.pinned {
            return Err(LayoutError::Unsupported(Unsupported::Pinned));
        }
    }
    let mut edge_order = filled(m, 0)?;
    for (i, value) in edge_order.iter_mut().enumerate() {
        *value = i;
    }
    guard.sort(&mut edge_order, maximum_id, |a, b| {
        edges[*a].id.cmp(edges[*b].id)
    })?;
    let mut forward = filled(n, Vec::<usize>::new())?;
    let mut reverse = filled(n, Vec::<usize>::new())?;
    let mut indexed = Vec::new();
    for (position, index) in edge_order.iter().enumerate() {
        guard.tick(n.saturating_mul(maximum_id).saturating_mul(2).max(1))?;
        let edge = edges[*index];
        if position > 0 && edge.id == edges[edge_order[position - 1]].id {
            return Err(LayoutError::DuplicateEdge);
        }
        let source = order
            .binary_search_by_key(&edge.source, |i| nodes[*i].id)
            .map_err(|_| LayoutError::UnknownEndpoint)?;
        let target = order
            .binary_search_by_key(&edge.target, |i| nodes[*i].id)
            .map_err(|_| LayoutError::UnknownEndpoint)?;
        push(&mut forward[source], target)?;
        push(&mut reverse[target], source)?;
        push(&mut indexed, (source, target, *index))?;
    }
    // One extra zero-charge poll fences the new standalone admission publication.
    guard.tick(0)?;
    Ok(AdmittedGraph {
        graph: PreparedGraph {
            nodes,
            edges,
            options,
            limits,
            order,
            forward,
            reverse,
            indexed,
            maximum_id,
            memory,
        },
        guard,
    })
}
