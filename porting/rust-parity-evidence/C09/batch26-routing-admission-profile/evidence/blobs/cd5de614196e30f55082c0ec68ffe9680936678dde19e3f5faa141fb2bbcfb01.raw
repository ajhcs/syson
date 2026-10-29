// SPDX-License-Identifier: EPL-2.0
//! Proposed flat directed placement, not ELK or native/UI qualification.
//! SCC ranks and weak-component bands continue the admitted graph's work state.
use crate::admission::{PreparedGraph, filled, push};
use crate::{AdmittedGraph, DirectedEdge, Direction, LayoutError, Limit};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

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

/// Consume one admission, reusing its canonical graph and original work guard.
/// A cancellation/error returns no partial layout; no input is modified.
pub fn place<'a, F: FnMut() -> bool>(
    admitted: AdmittedGraph<'_, 'a, F>,
) -> Result<Layout<'a>, LayoutError> {
    let (graph, mut guard) = admitted.into_parts();
    let PreparedGraph {
        nodes,
        edges,
        options,
        order,
        forward,
        reverse,
        indexed,
        maximum_id,
        memory,
        ..
    } = graph;
    let n = nodes.len();
    // Poll before placement allocations if the caller cancelled after admission.
    guard.tick(0)?;
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
        charged_work: guard.used(),
        estimated_workspace_bytes: memory,
    })
}
