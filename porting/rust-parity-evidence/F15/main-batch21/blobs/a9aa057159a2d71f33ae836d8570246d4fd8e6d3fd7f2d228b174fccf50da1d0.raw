// Copyright (c) 2023, 2024, 2025, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
//! Typed host label observations and source-derived leaf/note sizing.
//! This crate does not measure DOM/fonts, lay out containers, render nodes, or
//! claim that a host observation is evidence of an actual browser measurement.
#![forbid(unsafe_code)]
pub use syson_editor::Size;
use syson_editor::{LeafLayout, layout_package_leaf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Package,
    ViewFrame,
    Note,
}
impl NodeKind {
    pub fn from_renderer_type(name: &str) -> Option<Self> {
        match name {
            "sysMLPackageNode" => Some(Self::Package),
            "sysMLViewFrameNode" => Some(Self::ViewFrame),
            "sysMLNoteNode" => Some(Self::Note),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OptionalSize {
    pub width: Option<f64>,
    pub height: Option<f64>,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct BorderWidths {
    pub left: Option<f64>,
    pub all: Option<f64>,
}
#[derive(Clone, Copy, Debug)]
pub struct NodeGeometry<'a> {
    pub id: &'a str,
    pub kind: NodeKind,
    pub current: OptionalSize,
    pub defaults: OptionalSize,
    pub forced: OptionalSize,
    pub is_new: bool,
    pub resized_by_user: bool,
    pub keep_aspect_ratio: bool,
    pub direct_children: usize,
    /// Numeric CSS border values. String/CSS-unit parsing is not implicit.
    pub border: BorderWidths,
}
#[derive(Clone, Copy, Debug)]
pub struct PreviousNode<'a> {
    pub id: &'a str,
    pub dimensions: OptionalSize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeasurementStamp {
    pub layout_revision: u64,
    pub font_revision: u64,
    pub request_id: u64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontSpec<'a> {
    pub family: &'a str,
    pub size_px: f64,
    pub weight: u16,
    pub letter_spacing_px: f64,
    pub line_height_px: Option<f64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LabelTarget {
    /// Includes the source findIndex result, including -1 when absent.
    DomBoundingClientRect {
        element_id: String,
    },
    InsideLabelModel {
        node_id: String,
    },
}
#[derive(Clone, Debug)]
pub struct LabelRequest<'a> {
    pub target: LabelTarget,
    pub text: &'a str,
    pub font: FontSpec<'a>,
    pub stamp: MeasurementStamp,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LabelObservation {
    pub target: LabelTarget,
    pub stamp: MeasurementStamp,
    /// None represents the source's missing label/element, not a zero rectangle
    /// fabricated by the adapter. Width is retained but does not affect sizing.
    pub bounds: Option<Size>,
}
pub trait MeasurementHost {
    type Error;
    fn measure_label(
        &mut self,
        request: &LabelRequest<'_>,
    ) -> Result<LabelObservation, Self::Error>;
}
#[derive(Clone, Debug, PartialEq)]
pub enum MeasurementError<E> {
    Host(E),
    UnsupportedParent(NodeKind),
    InvalidNumber(&'static str),
    StaleObservation,
    WrongTarget,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MeasuredNode {
    pub size: Size,
    pub label: LabelObservation,
}
#[derive(Clone, Copy, Debug)]
pub struct LayoutRequest<'a> {
    pub node: NodeGeometry<'a>,
    pub previous: &'a [PreviousNode<'a>],
    pub visible_node_ids: &'a [&'a str],
    pub label_text: &'a str,
    pub font: FontSpec<'a>,
    pub stamp: MeasurementStamp,
}

/// Source computePreviousSize: new nodes ignore previous/current dimensions;
/// an existing previous object falls back per missing axis to defaults, not to
/// the current node. Matching uses the first representation ID occurrence.
fn previous_size(node: NodeGeometry<'_>, previous: &[PreviousNode<'_>]) -> Size {
    let defaults = Size {
        width: node.defaults.width.unwrap_or(150.0),
        height: node.defaults.height.unwrap_or(70.0),
    };
    let selected = if node.is_new {
        OptionalSize::default()
    } else {
        previous
            .iter()
            .find(|p| p.id == node.id)
            .map(|p| p.dimensions)
            .unwrap_or(node.current)
    };
    Size {
        width: selected.width.unwrap_or(defaults.width),
        height: selected.height.unwrap_or(defaults.height),
    }
}
fn valid<E>(number: f64, field: &'static str) -> Result<(), MeasurementError<E>> {
    if number.is_finite() && number >= 0.0 {
        Ok(())
    } else {
        Err(MeasurementError::InvalidNumber(field))
    }
}
fn valid_optional<E>(value: Option<f64>, field: &'static str) -> Result<(), MeasurementError<E>> {
    if let Some(number) = value {
        valid(number, field)?;
    }
    Ok(())
}
fn validate<E>(request: &LayoutRequest<'_>) -> Result<(), MeasurementError<E>> {
    for dimensions in [
        request.node.current,
        request.node.defaults,
        request.node.forced,
    ] {
        valid_optional(dimensions.width, "node width")?;
        valid_optional(dimensions.height, "node height")?;
    }
    if let Some(previous) = request.previous.iter().find(|p| p.id == request.node.id) {
        valid_optional(previous.dimensions.width, "previous width")?;
        valid_optional(previous.dimensions.height, "previous height")?;
    }
    valid(request.font.size_px, "font size")?;
    valid_optional(request.font.line_height_px, "font line height")?;
    if !request.font.letter_spacing_px.is_finite() {
        return Err(MeasurementError::InvalidNumber("letter spacing"));
    }
    // getNodeBorderWidth deliberately maps nonfinite numeric styles to zero.
    // Negative finite styles are outside this adapter's admitted geometry.
    for value in [request.node.border.left, request.node.border.all]
        .into_iter()
        .flatten()
    {
        if value.is_finite() && value < 0.0 {
            return Err(MeasurementError::InvalidNumber("border width"));
        }
    }
    Ok(())
}
/// Apply one of the three admitted geometry contracts. The caller controls font
/// and layout/request revisions; matching tokens prevent accidental stale reuse,
/// but do not authenticate the host or prove that it read the real DOM.
pub fn measure_node<H: MeasurementHost>(
    host: &mut H,
    request: LayoutRequest<'_>,
) -> Result<MeasuredNode, MeasurementError<H::Error>> {
    let node = request.node;
    if node.kind != NodeKind::Note && node.direct_children > 0 {
        return Err(MeasurementError::UnsupportedParent(node.kind));
    }
    validate(&request)?;
    let target = if node.kind == NodeKind::Note {
        LabelTarget::InsideLabelModel {
            node_id: node.id.into(),
        }
    } else {
        let index = request
            .visible_node_ids
            .iter()
            .position(|id| *id == node.id)
            .map(|index| index.to_string())
            .unwrap_or_else(|| "-1".into());
        LabelTarget::DomBoundingClientRect {
            element_id: format!("{}-label-{index}", node.id),
        }
    };
    let query = LabelRequest {
        target,
        text: request.label_text,
        font: request.font,
        stamp: request.stamp,
    };
    let label = host.measure_label(&query).map_err(MeasurementError::Host)?;
    if label.stamp != query.stamp {
        return Err(MeasurementError::StaleObservation);
    }
    if label.target != query.target {
        return Err(MeasurementError::WrongTarget);
    }
    if let Some(bounds) = label.bounds {
        valid(bounds.width, "label width")?;
        valid(bounds.height, "label height")?;
    }
    let previous = previous_size(node, request.previous);
    let mut size = if node.kind == NodeKind::Note {
        note_size(
            node,
            previous,
            label.bounds.map(|b| b.height).unwrap_or(0.0),
        )
    } else {
        layout_package_leaf(LeafLayout {
            measured_label_height: label.bounds.map(|b| b.height),
            default_width: node.defaults.width,
            default_height: node.defaults.height,
            resized_by_user: node.resized_by_user,
            previous_size: previous,
            forced_width: node.forced.width,
            forced_height: node.forced.height,
        })
    };
    if node.kind == NodeKind::Note && node.keep_aspect_ratio {
        apply_ratio(node.defaults, &mut size);
    }
    valid(size.width, "computed width")?;
    valid(size.height, "computed height")?;
    Ok(MeasuredNode { size, label })
}
fn note_size(node: NodeGeometry<'_>, previous: Size, label_height: f64) -> Size {
    let border = node
        .border
        .left
        .or(node.border.all)
        .filter(|v| v.is_finite())
        .unwrap_or(0.0);
    let minimum_height = label_height + border * 2.0;
    let width = node.forced.width.or(node.current.width).unwrap_or(200.0);
    let height = node
        .forced
        .height
        .unwrap_or_else(|| minimum_height.max(node.defaults.height.unwrap_or(70.0)));
    Size {
        width: if node.resized_by_user && !node.forced.width.is_some_and(|v| v != 0.0) {
            width.max(previous.width)
        } else {
            width
        },
        height: if node.resized_by_user && !node.forced.height.is_some_and(|v| v != 0.0) {
            minimum_height.max(previous.height)
        } else {
            height
        },
    }
}
fn apply_ratio(defaults: OptionalSize, size: &mut Size) {
    // Source uses ||, deliberately unlike the ?? default in previous sizing.
    let width = defaults.width.filter(|v| *v != 0.0).unwrap_or(150.0);
    let height = defaults.height.filter(|v| *v != 0.0).unwrap_or(70.0);
    let initial_ratio = width / height;
    if size.width != 0.0 && size.height != 0.0 {
        let new_ratio = size.width / size.height;
        if initial_ratio > new_ratio {
            size.width = size.height * initial_ratio;
        }
        if initial_ratio < new_ratio {
            size.height = size.width / initial_ratio;
        }
    }
}
