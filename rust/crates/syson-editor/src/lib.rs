// Copyright (c) 2023, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
// Contributors: Obeo - initial API and implementation; Rust port contributors.
//! Browser-independent ports of bounded SysON editor algorithms.
//! Upstream: 6ba2ac64e6b16ee2b05752ad55395eebb87d4910.
//! This library does not provide a renderer, transport or Sirius layout engine.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Node<T> {
    pub id: String,
    pub position: Point,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub resized_by_user: bool,
    pub connection_handles: Vec<String>,
    /// Uninterpreted renderer/domain data, preserved across rotations.
    pub payload: T,
}
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeData<T> {
    pub bending_points: Option<Vec<Point>>,
    pub payload: T,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Edge<T> {
    pub id: String,
    pub source: String,
    pub target: String,
    pub data: Option<EdgeData<T>>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Diagram<N, E> {
    pub nodes: Vec<Node<N>>,
    pub edges: Vec<Edge<E>>,
}

/// First phase of useRotateNode.ts. Only the first selected ID is rotated;
/// adjacent edge bend points are cleared before layout. Missing sizes are zero.
pub fn prepare_rotation<N: Clone, E: Clone>(
    diagram: &Diagram<N, E>,
    selected: &[String],
) -> Diagram<N, E> {
    let mut result = diagram.clone();
    if let Some(first) = selected.first() {
        for node in &mut result.nodes {
            if node.id == *first {
                let width = node.width.unwrap_or(0.0);
                let height = node.height.unwrap_or(0.0);
                node.position = Point {
                    x: node.position.x + width / 2.0 - height / 2.0,
                    y: node.position.y + height / 2.0 - width / 2.0,
                };
                node.width = Some(height);
                node.height = Some(width);
                node.resized_by_user = true;
                node.connection_handles.clear();
            }
        }
        for edge in &mut result.edges {
            if (edge.source == *first || edge.target == *first)
                && let Some(data) = &mut edge.data
            {
                data.bending_points = None;
            }
        }
    }
    result
}

/// Second phase of useRotateNode.ts: adopt laid-out nodes for every selected ID,
/// and laid-out edges incident to any selection. Unselected objects and objects
/// missing from the layout result retain their prepared state and ordering.
/// The caller must supply the actual layout result; no identity-layout stub.
pub fn finish_rotation<N: Clone, E: Clone>(
    prepared: &Diagram<N, E>,
    laid_out: &Diagram<N, E>,
    selected: &[String],
) -> Diagram<N, E> {
    let mut result = prepared.clone();
    for node in &mut result.nodes {
        if selected
            .iter()
            .find(|id| **id == node.id)
            .is_some_and(|id| !id.is_empty())
            && let Some(updated) = laid_out.nodes.iter().find(|n| n.id == node.id)
        {
            *node = updated.clone();
        }
    }
    for edge in &mut result.edges {
        if selected
            .iter()
            .find(|id| **id == edge.source || **id == edge.target)
            .is_some_and(|id| !id.is_empty())
            && let Some(updated) = laid_out.edges.iter().find(|e| e.id == edge.id)
        {
            *edge = updated.clone();
        }
    }
    result
}

/// Inputs normally obtained from DOM measurement and Sirius computePreviousSize.
#[derive(Clone, Copy, Debug, Default)]
pub struct LeafLayout {
    pub measured_label_height: Option<f64>,
    pub default_width: Option<f64>,
    pub default_height: Option<f64>,
    pub resized_by_user: bool,
    pub previous_size: Size,
    pub forced_width: Option<f64>,
    pub forced_height: Option<f64>,
}
fn js_truthy(value: Option<f64>) -> bool {
    value.is_some_and(|v| v != 0.0 && !v.is_nan())
}
fn js_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
}
/// Exact leaf formulas shared by package, imported-package and view-frame handlers.
/// Upstream force values only suppress old-size retention; they do not become
/// dimensions in these leaf branches. A zero force is false in JavaScript.
pub fn layout_package_leaf(input: LeafLayout) -> Size {
    let min_width = if input.resized_by_user {
        75.0
    } else {
        input.default_width.unwrap_or(75.0)
    };
    let min_height = js_max(
        16.0 + input.measured_label_height.unwrap_or(0.0),
        if input.resized_by_user {
            50.0
        } else {
            input.default_height.unwrap_or(50.0)
        },
    );
    Size {
        width: if input.resized_by_user && !js_truthy(input.forced_width) {
            if min_width > input.previous_size.width {
                min_width
            } else {
                input.previous_size.width
            }
        } else {
            min_width
        },
        height: if input.resized_by_user && !js_truthy(input.forced_height) {
            if min_height > input.previous_size.height {
                min_height
            } else {
                input.previous_size.height
            }
        } else {
            min_height
        },
    }
}

/// SVG path data from SysMLNoteNode.tsx. border_width is the numeric CSS width;
/// callers resolving CSS tokens/strings must implement upstream parseInt semantics.
/// Color/theme resolution and DOM display remain outside this bounded port.
pub fn note_paths(width: f64, height: f64, border_width: f64) -> (String, String) {
    let offset = border_width.trunc() / 2.0;
    (
        format!(
            "M {offset},{offset} H {} L {} 15 V {} H {offset} Z",
            width - 15.0,
            width - offset,
            height - offset
        ),
        format!("M {},{offset} V 15 H {}", width - 15.0, width - offset),
    )
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoteAppearance<'a> {
    pub stroke: &'a str,
    pub fill: &'a str,
    pub fill_opacity: f64,
    pub stroke_opacity: f64,
    /// None means the upstream empty CSS opacity string (default opacity).
    pub container_opacity: Option<f64>,
    pub outline: Option<&'a str>,
    pub container_background: &'static str,
    pub container_border: &'static str,
}
/// Already-resolved theme colors. Stroke and fill remain independent, matching
/// the upstream note regression test; selection/hover decorate the container.
pub fn note_appearance<'a>(
    stroke: &'a str,
    fill: &'a str,
    selected_color: &'a str,
    selected: bool,
    hovered: bool,
    faded: bool,
) -> NoteAppearance<'a> {
    NoteAppearance {
        stroke,
        fill,
        fill_opacity: if faded { 0.4 } else { 1.0 },
        stroke_opacity: if faded { 0.4 } else { 1.0 },
        container_opacity: if faded { Some(0.4) } else { None },
        outline: if selected || hovered {
            Some(selected_color)
        } else {
            None
        },
        container_background: "transparent",
        container_border: "none",
    }
}
