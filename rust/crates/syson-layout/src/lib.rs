// SPDX-License-Identifier: EPL-2.0
//! Proposed Rust flat directed layout; not a port of or equivalent to ELK.
//! SCC condensation gives directed longest-path ranks. Weak components occupy
//! separate cross-axis bands; measured layer widths prevent rectangle overlap.
#![forbid(unsafe_code)]
pub mod containers;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasuredNode<'a> {
    pub id: &'a str,
    pub width: f64,
    pub height: f64,
    pub parent: Option<&'a str>,
    pub port_count: usize,
    pub pinned: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirectedEdge<'a> {
    pub id: &'a str,
    pub source: &'a str,
    pub target: &'a str,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Right,
    Down,
}
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub direction: Direction,
    pub layer_gap: f64,
    pub node_gap: f64,
    pub component_gap: f64,
    pub padding: f64,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            direction: Direction::Right,
            layer_gap: 40.0,
            node_gap: 20.0,
            component_gap: 60.0,
            padding: 16.0,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_nodes: usize,
    pub max_edges: usize,
    pub max_work: usize,
    /// Logical conservative buffer estimate, not allocator overhead or RSS.
    pub max_workspace_bytes: usize,
    pub max_id_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_nodes: 4096,
            max_edges: 16384,
            max_work: 100_000_000,
            max_workspace_bytes: 16 * 1024 * 1024,
            max_id_bytes: 1024,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    Nodes,
    Edges,
    Work,
    Memory,
    IdBytes,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unsupported {
    Container,
    Ports,
    Pinned,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutError {
    EmptyId,
    DuplicateNode,
    DuplicateEdge,
    UnknownEndpoint,
    InvalidNumber,
    ArithmeticOverflow,
    PrecisionLoss,
    Allocation,
    Cancelled,
    Limit(Limit),
    Unsupported(Unsupported),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement<'a> {
    pub id: &'a str,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub strong_component: usize,
    pub weak_component: usize,
    pub rank: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClassifiedEdge<'a> {
    pub edge: DirectedEdge<'a>,
    pub internal_strong_component: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Layout<'a> {
    /// Canonical ID order, independent of input permutations.
    pub nodes: Vec<Placement<'a>>,
    pub edges: Vec<ClassifiedEdge<'a>>,
    pub width: f64,
    pub height: f64,
    pub charged_work: usize,
    pub estimated_workspace_bytes: usize,
}
struct Guard<'a, F> {
    cancelled: &'a mut F,
    used: usize,
    maximum: usize,
}
impl<F: FnMut() -> bool> Guard<'_, F> {
    fn tick(&mut self, units: usize) -> Result<(), LayoutError> {
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
    fn sort<T>(
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
fn filled<T: Clone>(length: usize, value: T) -> Result<Vec<T>, LayoutError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(length)
        .map_err(|_| LayoutError::Allocation)?;
    result.resize(length, value);
    Ok(result)
}
fn push<T>(values: &mut Vec<T>, value: T) -> Result<(), LayoutError> {
    values.try_reserve(1).map_err(|_| LayoutError::Allocation)?;
    values.push(value);
    Ok(())
}
fn add(a: f64, b: f64) -> Result<f64, LayoutError> {
    let value = a + b;
    if !value.is_finite() {
        Err(LayoutError::ArithmeticOverflow)
    } else if (b > 0.0 && value <= a) || (a > 0.0 && value <= b) {
        Err(LayoutError::PrecisionLoss)
    } else {
        Ok(value)
    }
}
fn valid(number: f64) -> bool {
    number.is_finite() && number >= 0.0
}

/// Inputs are borrowed and never modified. Failures/cancellation return no
/// partial layout. Work is charged logical operations, not a CPU time bound;
/// sorts precharge n² times maximum key bytes and poll before/after. Memory is a
/// conservative logical admission estimate (1024*N+640*E), not an OS memory cap.
pub fn layout<'a>(
    nodes: &[MeasuredNode<'a>],
    edges: &[DirectedEdge<'a>],
    options: Options,
    limits: Limits,
    mut cancelled: impl FnMut() -> bool,
) -> Result<Layout<'a>, LayoutError> {
    let mut guard = Guard {
        cancelled: &mut cancelled,
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
    let mut seen = filled(n, false)?;
    let (mut finish, mut stack) = (Vec::new(), Vec::new());
    for start in 0..n {
        guard.tick(1)?;
        if seen[start] {
            continue;
        }
        seen[start] = true;
        push(&mut stack, (start, 0))?;
        while let Some(&(node, next)) = stack.last() {
            guard.tick(1)?;
            if next < forward[node].len() {
                let child = forward[node][next];
                stack.last_mut().unwrap().1 += 1;
                if !seen[child] {
                    seen[child] = true;
                    push(&mut stack, (child, 0))?;
                }
            } else {
                push(&mut finish, node)?;
                stack.pop();
            }
        }
    }
    let mut component = filled(n, usize::MAX)?;
    let (mut representatives, mut pending) = (Vec::new(), Vec::new());
    for start in finish.into_iter().rev() {
        guard.tick(1)?;
        if component[start] != usize::MAX {
            continue;
        }
        let id = representatives.len();
        push(&mut representatives, start)?;
        component[start] = id;
        push(&mut pending, start)?;
        while let Some(node) = pending.pop() {
            guard.tick(1)?;
            representatives[id] = representatives[id].min(node);
            for &parent in &reverse[node] {
                guard.tick(1)?;
                if component[parent] == usize::MAX {
                    component[parent] = id;
                    push(&mut pending, parent)?;
                }
            }
        }
    }
    let k = representatives.len();
    let mut canonical = filled(k, 0)?;
    for (i, slot) in canonical.iter_mut().enumerate() {
        *slot = i;
    }
    guard.sort(&mut canonical, 1, |a, b| {
        representatives[*a].cmp(&representatives[*b])
    })?;
    let mut remap = filled(k, 0)?;
    for (new, old) in canonical.into_iter().enumerate() {
        remap[old] = new;
    }
    for value in &mut component {
        guard.tick(1)?;
        *value = remap[*value];
    }
    let mut arcs = Vec::new();
    for &(source, target, _) in &indexed {
        guard.tick(1)?;
        if component[source] != component[target] {
            push(&mut arcs, (component[source], component[target]))?;
        }
    }
    guard.sort(&mut arcs, 1, |a, b| a.cmp(b))?;
    guard.tick(arcs.len())?;
    arcs.dedup();
    guard.tick(0)?;
    let mut indegree = filled(k, 0usize)?;
    let mut outgoing = filled(k, Vec::<usize>::new())?;
    for &(source, target) in &arcs {
        guard.tick(1)?;
        indegree[target] += 1;
        push(&mut outgoing[source], target)?;
    }
    let mut ready = BinaryHeap::new();
    ready.try_reserve(k).map_err(|_| LayoutError::Allocation)?;
    let heap_work = (usize::BITS - k.leading_zeros()).max(1) as usize;
    for (id, &incoming) in indegree.iter().enumerate() {
        guard.tick(heap_work)?;
        if incoming == 0 {
            ready.push(Reverse(id));
        }
    }
    let mut rank = filled(k, 0usize)?;
    while !ready.is_empty() {
        guard.tick(heap_work)?;
        let Reverse(source) = ready.pop().unwrap();
        for &target in &outgoing[source] {
            guard.tick(heap_work)?;
            rank[target] = rank[target].max(rank[source] + 1);
            indegree[target] -= 1;
            if indegree[target] == 0 {
                ready.push(Reverse(target));
            }
        }
    }
    let mut weak = filled(n, usize::MAX)?;
    let mut weak_count = 0;
    for start in 0..n {
        guard.tick(1)?;
        if weak[start] != usize::MAX {
            continue;
        }
        weak[start] = weak_count;
        push(&mut pending, start)?;
        while let Some(node) = pending.pop() {
            guard.tick(1)?;
            for &other in forward[node].iter().chain(&reverse[node]) {
                guard.tick(1)?;
                if weak[other] == usize::MAX {
                    weak[other] = weak_count;
                    push(&mut pending, other)?;
                }
            }
        }
        weak_count += 1;
    }
    let (mut placed, mut extent_primary, mut extent_cross, mut band) =
        (Vec::new(), 0.0f64, 0.0f64, options.padding);
    for group in 0..weak_count {
        guard.tick(k.checked_mul(4).ok_or(LayoutError::Limit(Limit::Work))?)?;
        let mut layer_count = 0;
        let mut widths = filled(k, 0.0f64)?;
        for (i, &index) in order.iter().enumerate() {
            guard.tick(1)?;
            if weak[i] == group {
                let node = nodes[index];
                let primary = if options.direction == Direction::Right {
                    node.width
                } else {
                    node.height
                };
                let layer = rank[component[i]];
                layer_count = layer_count.max(layer + 1);
                widths[layer] = widths[layer].max(primary);
            }
        }
        let mut origins = filled(layer_count, options.padding)?;
        for layer in 1..layer_count {
            guard.tick(1)?;
            origins[layer] = add(
                add(origins[layer - 1], widths[layer - 1])?,
                options.layer_gap,
            )?;
        }
        let mut cursors = filled(layer_count, band)?;
        let mut occupied = filled(layer_count, false)?;
        let mut bottom = band;
        for (i, &index) in order.iter().enumerate() {
            guard.tick(1)?;
            if weak[i] != group {
                continue;
            }
            let node = nodes[index];
            let layer = rank[component[i]];
            if occupied[layer] {
                cursors[layer] = add(cursors[layer], options.node_gap)?;
            }
            let (primary_size, cross_size) = if options.direction == Direction::Right {
                (node.width, node.height)
            } else {
                (node.height, node.width)
            };
            let (x, y) = if options.direction == Direction::Right {
                (origins[layer], cursors[layer])
            } else {
                (cursors[layer], origins[layer])
            };
            push(
                &mut placed,
                Placement {
                    id: node.id,
                    x,
                    y,
                    width: node.width,
                    height: node.height,
                    strong_component: component[i],
                    weak_component: group,
                    rank: layer,
                },
            )?;
            cursors[layer] = add(cursors[layer], cross_size)?;
            occupied[layer] = true;
            bottom = bottom.max(cursors[layer]);
            extent_primary = extent_primary.max(add(origins[layer], primary_size)?);
        }
        extent_cross = extent_cross.max(bottom);
        if group + 1 < weak_count {
            band = add(bottom, options.component_gap)?;
        }
    }
    guard.sort(&mut placed, maximum_id, |a, b| a.id.cmp(b.id))?;
    let mut classified = Vec::new();
    for (source, target, index) in indexed {
        guard.tick(1)?;
        push(
            &mut classified,
            ClassifiedEdge {
                edge: edges[index],
                internal_strong_component: component[source] == component[target],
            },
        )?;
    }
    let (primary, cross) = if n == 0 {
        (0.0, 0.0)
    } else {
        (
            add(extent_primary, options.padding)?,
            add(extent_cross, options.padding)?,
        )
    };
    let (width, height) = if options.direction == Direction::Right {
        (primary, cross)
    } else {
        (cross, primary)
    };
    guard.tick(0)?;
    Ok(Layout {
        nodes: placed,
        edges: classified,
        width,
        height,
        charged_work: guard.used,
        estimated_workspace_bytes: memory,
    })
}
