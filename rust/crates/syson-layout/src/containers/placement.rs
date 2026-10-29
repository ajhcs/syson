// Copyright (c) 2023, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
//! Source-derived package sizing around actual U10b sibling placement.
use super::{AdmittedForest, Error, PackageLayout, PackagePlacement, Position, Size};
use crate::{LayoutError, Limits, MeasuredNode, Options, layout};

fn buffer<T>(length: usize) -> Result<Vec<T>, Error> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(length)
        .map_err(|_| LayoutError::Allocation)?;
    Ok(result)
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

/// Consume one canonical admission and its original callback/work state.
/// Each sibling call receives remaining work and is debited exactly once.
/// New zero-work entry/publication polls retain atomic cancellation, even empty.
/// Admission's early validation of empty/unused inputs remains authoritative.
pub fn place_packages<'a, F: FnMut() -> bool>(
    admitted: AdmittedForest<'a, F>,
) -> Result<PackageLayout<'a>, Error> {
    let AdmittedForest {
        ordered,
        parents,
        bottom_up,
        indexed_edges,
        options,
        limits,
        memory,
        mut budget,
    } = admitted;
    let (n, m) = (ordered.len(), indexed_edges.len());
    budget.tick(0)?;
    let mut sizes = buffer(n)?;
    sizes.extend(ordered.iter().map(|node| node.leaf_size));
    let mut positions = buffer(n)?;
    positions.resize(n, Position::default());
    for group in bottom_up
        .iter()
        .copied()
        .map(Some)
        .chain(std::iter::once(None))
    {
        let mut children = buffer(n)?;
        let mut flat_nodes = buffer(n)?;
        for (i, node) in ordered.iter().enumerate() {
            budget.tick(1)?;
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
            budget.tick(1)?;
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
            max_work: budget.remaining(),
            ..limits
        };
        let flat = layout(&flat_nodes, &flat_edges, inner_options, flat_limits, || {
            budget.poll()
        })?;
        budget.tick(flat.charged_work)?;
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
            budget.tick(1)?;
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
    let mut absolute = buffer(n)?;
    absolute.resize(n, Position::default());
    for index in bottom_up.iter().rev().copied() {
        budget.tick(1)?;
        let base = parents[index].map(|p| absolute[p]).unwrap_or_default();
        absolute[index] = Position {
            x: add(base.x, positions[index].x)?,
            y: add(base.y, positions[index].y)?,
        };
    }
    let (mut width, mut height) = (0.0f64, 0.0f64);
    let mut result = buffer(n)?;
    for (i, node) in ordered.iter().enumerate() {
        budget.tick(1)?;
        if parents[i].is_none() {
            width = width.max(add(positions[i].x, sizes[i].width)?);
            height = height.max(add(positions[i].y, sizes[i].height)?);
        }
        result.push(PackagePlacement {
            id: node.id,
            parent: node.parent,
            relative: positions[i],
            absolute: absolute[i],
            size: sizes[i],
        });
    }
    if n > 0 {
        width = add(width, options.padding)?;
        height = add(height, options.padding)?;
    }
    budget.tick(0)?;
    Ok(PackageLayout {
        nodes: result,
        width,
        height,
        charged_work: limits.max_work - budget.remaining(),
        estimated_workspace_bytes: memory,
    })
}
