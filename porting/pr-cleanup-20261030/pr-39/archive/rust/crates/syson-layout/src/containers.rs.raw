// Copyright (c) 2023, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
//! Package-only source-derived sizing/clamping around proposed F16 sibling layout.
//! Header footprints and retained/user-minimum geometry are caller observations.
use super::{DirectedEdge, Guard, LayoutError, Limit, Limits, MeasuredNode, Options};
use super::{add, filled, layout, push, valid};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}
#[derive(Clone, Copy, Debug)]
pub struct PackageNode<'a> {
    pub id: &'a str,
    pub parent: Option<&'a str>,
    /// Already measured leaf dimensions, e.g. the F15 synthetic-host result.
    pub leaf_size: Size,
    /// Pre-observed top-header footprint, including a separator if applicable.
    pub header_height_footprint: f64,
    pub border_width: f64,
    pub default_width: Option<f64>,
    pub default_height: Option<f64>,
    /// Precomputed previous dimensions for a user-resized parent, not fixed size.
    pub user_minimum: Option<Size>,
    /// Previous relative position. Newly-created-position overlays are excluded.
    pub retained_position: Option<Position>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Layout(LayoutError),
    UnknownParent,
    ParentCycle,
    CrossParentEdge,
}
impl From<LayoutError> for Error {
    fn from(value: LayoutError) -> Self {
        Self::Layout(value)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PackagePlacement<'a> {
    pub id: &'a str,
    pub parent: Option<&'a str>,
    pub relative: Position,
    pub absolute: Position,
    pub size: Size,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PackageLayout<'a> {
    /// Canonical ID order; ancestors need not precede descendants.
    pub nodes: Vec<PackagePlacement<'a>>,
    pub width: f64,
    pub height: f64,
    pub charged_work: usize,
    /// Logical admission estimate; not allocator overhead or an RSS bound.
    pub estimated_workspace_bytes: usize,
}
fn buffer<T>(length: usize) -> Result<Vec<T>, Error> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(length)
        .map_err(|_| LayoutError::Allocation)?;
    Ok(result)
}
/// Lay out a package forest bottom-up, then project relative to absolute positions.
/// Only edges between siblings are admitted. Fresh siblings use F16; previous
/// positions can overlap and are preserved with the source parent clamps. This
/// is not ELK, DOM measurement, border-port layout or a source cache lifecycle.
/// All supplied geometry is validated, including unused leaf/default values.
/// Work is logical charging, not a deadline. Inputs are immutable; errors return
/// no partial layout. Repeated calls are deterministic for identical inputs.
pub fn layout_packages<'a>(
    nodes: &[PackageNode<'a>],
    edges: &[DirectedEdge<'a>],
    options: Options,
    limits: Limits,
    mut cancelled: impl FnMut() -> bool,
) -> Result<PackageLayout<'a>, Error> {
    let mut guard = Guard {
        cancelled: &mut cancelled,
        used: 0,
        maximum: limits.max_work,
    };
    guard.tick(0)?;
    let (n, m) = (nodes.len(), edges.len());
    if n > limits.max_nodes {
        return Err(LayoutError::Limit(Limit::Nodes).into());
    }
    if m > limits.max_edges {
        return Err(LayoutError::Limit(Limit::Edges).into());
    }
    let memory = n
        .checked_mul(4096)
        .and_then(|x| m.checked_mul(1024).and_then(|y| x.checked_add(y)))
        .ok_or(LayoutError::Limit(Limit::Memory))?;
    if memory > limits.max_workspace_bytes {
        return Err(LayoutError::Limit(Limit::Memory).into());
    }
    guard.tick(
        n.checked_mul(32)
            .and_then(|x| m.checked_mul(8).and_then(|y| x.checked_add(y)))
            .ok_or(LayoutError::Limit(Limit::Work))?,
    )?;
    let mut ordered = buffer(n)?;
    for node in nodes {
        guard.tick(node.id.len().max(1))?;
        if node.id.is_empty() {
            return Err(LayoutError::EmptyId.into());
        }
        if node.id.len() > limits.max_id_bytes {
            return Err(LayoutError::Limit(Limit::IdBytes).into());
        }
        for number in [
            node.leaf_size.width,
            node.leaf_size.height,
            node.header_height_footprint,
            node.border_width,
        ]
        .into_iter()
        .chain(node.default_width)
        .chain(node.default_height)
        .chain(
            node.user_minimum
                .into_iter()
                .flat_map(|s| [s.width, s.height]),
        ) {
            if !valid(number) {
                return Err(LayoutError::InvalidNumber.into());
            }
        }
        if node
            .retained_position
            .is_some_and(|p| !p.x.is_finite() || !p.y.is_finite())
        {
            return Err(LayoutError::InvalidNumber.into());
        }
        ordered.push(*node);
    }
    guard.sort(&mut ordered, limits.max_id_bytes, |a, b| a.id.cmp(b.id))?;
    for pair in ordered.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(LayoutError::DuplicateNode.into());
        }
    }
    let lookup = |id: &str| ordered.binary_search_by_key(&id, |node| node.id);
    let mut parents = filled(n, None)?;
    for (i, node) in ordered.iter().enumerate() {
        guard.tick(n.saturating_mul(limits.max_id_bytes).max(1))?;
        if let Some(parent) = node.parent {
            if parent.len() > limits.max_id_bytes {
                return Err(LayoutError::Limit(Limit::IdBytes).into());
            }
            parents[i] = Some(lookup(parent).map_err(|_| Error::UnknownParent)?);
        }
    }
    let mut depths = filled(n, 0usize)?;
    for (i, depth) in depths.iter_mut().enumerate() {
        let mut cursor = parents[i];
        while let Some(parent) = cursor {
            guard.tick(1)?;
            *depth += 1;
            if *depth >= n {
                return Err(Error::ParentCycle);
            }
            cursor = parents[parent];
        }
    }
    let mut indexed_edges = buffer(m)?;
    for edge in edges {
        guard.tick(
            n.saturating_mul(limits.max_id_bytes)
                .saturating_mul(2)
                .max(1),
        )?;
        for id in [edge.id, edge.source, edge.target] {
            if id.is_empty() {
                return Err(LayoutError::EmptyId.into());
            }
            if id.len() > limits.max_id_bytes {
                return Err(LayoutError::Limit(Limit::IdBytes).into());
            }
        }
        let source = lookup(edge.source).map_err(|_| LayoutError::UnknownEndpoint)?;
        let target = lookup(edge.target).map_err(|_| LayoutError::UnknownEndpoint)?;
        if parents[source] != parents[target] {
            return Err(Error::CrossParentEdge);
        }
        indexed_edges.push((*edge, parents[source]));
    }
    guard.sort(&mut indexed_edges, limits.max_id_bytes, |a, b| {
        a.0.id.cmp(b.0.id)
    })?;
    for pair in indexed_edges.windows(2) {
        if pair[0].0.id == pair[1].0.id {
            return Err(LayoutError::DuplicateEdge.into());
        }
    }
    let mut bottom_up = filled(n, 0usize)?;
    for (i, value) in bottom_up.iter_mut().enumerate() {
        *value = i;
    }
    guard.sort(&mut bottom_up, 1, |a, b| {
        depths[*b].cmp(&depths[*a]).then(a.cmp(b))
    })?;
    let mut sizes = buffer(n)?;
    sizes.extend(ordered.iter().map(|node| node.leaf_size));
    let mut positions = filled(n, Position::default())?;
    for group in bottom_up
        .iter()
        .copied()
        .map(Some)
        .chain(std::iter::once(None))
    {
        let mut children = buffer(n)?;
        let mut flat_nodes = buffer(n)?;
        for (i, node) in ordered.iter().enumerate() {
            guard.tick(1)?;
            if parents[i] == group {
                children.push(i);
                flat_nodes.push(MeasuredNode {
                    id: node.id,
                    width: sizes[i].width,
                    height: sizes[i].height,
                    parent: None,
                    port_count: 0,
                    pinned: false,
                });
            }
        }
        if children.is_empty() && group.is_some() {
            continue;
        }
        let mut flat_edges = buffer(m)?;
        for (edge, parent) in &indexed_edges {
            guard.tick(1)?;
            if *parent == group {
                flat_edges.push(*edge);
            }
        }
        let inner_options = Options {
            padding: if group.is_some() {
                0.0
            } else {
                options.padding
            },
            ..options
        };
        let flat_limits = Limits {
            max_work: guard.maximum.saturating_sub(guard.used),
            ..limits
        };
        let flat = layout(
            &flat_nodes,
            &flat_edges,
            inner_options,
            flat_limits,
            &mut *guard.cancelled,
        )?;
        guard.tick(flat.charged_work)?;
        let (x_offset, y_offset, retained_y) = if let Some(parent) = group {
            let node = ordered[parent];
            (
                add(8.0, node.border_width)?,
                add(add(8.0, node.border_width)?, node.header_height_footprint)?,
                add(8.0, node.header_height_footprint)?,
            )
        } else {
            (0.0, 0.0, 0.0)
        };
        let (mut right, mut bottom) = (0.0f64, 0.0f64);
        for (index, placement) in children.iter().zip(&flat.nodes) {
            guard.tick(1)?;
            let node = ordered[*index];
            let position = if let Some(previous) = node.retained_position {
                Position {
                    x: if previous.x < 0.0 {
                        if group.is_some() { 8.0 } else { 0.0 }
                    } else {
                        previous.x
                    },
                    y: previous.y.max(retained_y),
                }
            } else {
                Position {
                    x: add(placement.x, x_offset)?,
                    y: add(placement.y, y_offset)?,
                }
            };
            positions[*index] = position;
            right = right.max(add(position.x, sizes[*index].width)?);
            bottom = bottom.max(add(position.y, sizes[*index].height)?);
        }
        if let Some(parent) = group {
            let node = ordered[parent];
            let borders = add(node.border_width, node.border_width)?;
            let default_width = if node.user_minimum.is_none() {
                node.default_width.unwrap_or(0.0)
            } else {
                0.0
            };
            let default_height = if node.user_minimum.is_none() {
                node.default_height.unwrap_or(0.0)
            } else {
                0.0
            };
            let minimum = Size {
                width: add(add(right, 8.0)?.max(default_width), borders)?,
                height: add(add(bottom, 8.0)?.max(default_height), borders)?,
            };
            sizes[parent] = match node.user_minimum {
                Some(previous) => Size {
                    width: minimum.width.max(previous.width),
                    height: minimum.height.max(previous.height),
                },
                None => Size {
                    width: minimum.width.max(node.default_width.unwrap_or(150.0)),
                    height: minimum.height.max(node.default_height.unwrap_or(70.0)),
                },
            };
        }
    }
    let mut absolute = filled(n, Position::default())?;
    for index in bottom_up.iter().rev().copied() {
        guard.tick(1)?;
        let base = parents[index].map(|p| absolute[p]).unwrap_or_default();
        absolute[index] = Position {
            x: add(base.x, positions[index].x)?,
            y: add(base.y, positions[index].y)?,
        };
    }
    let (mut width, mut height) = (0.0f64, 0.0f64);
    let mut result = buffer(n)?;
    for (i, node) in ordered.iter().enumerate() {
        guard.tick(1)?;
        if parents[i].is_none() {
            width = width.max(add(positions[i].x, sizes[i].width)?);
            height = height.max(add(positions[i].y, sizes[i].height)?);
        }
        push(
            &mut result,
            PackagePlacement {
                id: node.id,
                parent: node.parent,
                relative: positions[i],
                absolute: absolute[i],
                size: sizes[i],
            },
        )?;
    }
    if n > 0 {
        width = add(width, options.padding)?;
        height = add(height, options.padding)?;
    }
    guard.tick(0)?;
    Ok(PackageLayout {
        nodes: result,
        width,
        height,
        charged_work: guard.used,
        estimated_workspace_bytes: memory,
    })
}
