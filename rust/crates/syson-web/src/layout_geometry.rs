// SPDX-License-Identifier: EPL-2.0
//! Finite package/part canvas geometry. Proposed integer policy, not native rendering.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use syson_editor::{Diagram, Edge, EdgeData, Node, Point};
use syson_layout::containers::{PackageNode, Size, layout_packages};
use syson_model::Document;
use syson_node_measurement as measure;
use syson_routing::{Edge as RouteEdge, Side};

#[derive(Clone, Debug, PartialEq)]
pub struct Payload {
    pub parent: Option<String>,
    pub header: f64,
}
pub type Scene = Diagram<Payload, ()>;
pub(crate) fn fail() -> String {
    "This canvas supports at most 32 package/part nodes and 64 same-parent typing edges with finite integer geometry.".into()
}
pub(crate) fn number(v: &Value) -> Result<f64, String> {
    v.as_f64()
        .filter(|n| n.is_finite() && n.abs() <= 1_000_000.)
        .ok_or_else(fail)
}
pub(crate) fn identity(v: &Value) -> Result<&str, String> {
    v.as_str()
        .filter(|s| !s.is_empty() && s.len() <= 1024)
        .ok_or_else(fail)
}
fn supported(class: &str) -> bool {
    matches!(class, "Package" | "PartDefinition" | "PartUsage")
}
pub fn measured(
    doc: &Document,
    raw: &str,
    epoch: u64,
    previous: Option<&Scene>,
) -> Result<Scene, String> {
    if raw.len() > 1_048_576 {
        return Err(fail());
    }
    let observations: Value =
        serde_json::from_str(raw).map_err(|_| "Label observations are missing or invalid.")?;
    let rows = observations.as_array().ok_or_else(fail)?;
    let all = doc.elements();
    if all.iter().any(|e| {
        !supported(&e.class)
            && !matches!(
                e.class.as_str(),
                "OwningMembership" | "FeatureMembership" | "FeatureTyping"
            )
    }) {
        return Err(fail());
    }
    let elements: Vec<_> = all.into_iter().filter(|e| supported(&e.class)).collect();
    if elements.len() > 32 || rows.len() != elements.len() {
        return Err(fail());
    }
    let ids: Vec<_> = elements.iter().map(|e| e.element_id.as_str()).collect();
    let mut bounds = BTreeMap::new();
    for row in rows {
        let id = identity(&row["id"])?;
        let size = measure::Size {
            width: number(&row["width"])?,
            height: number(&row["height"])?,
        };
        if size.width < 0. || size.height <= 0. || bounds.insert(id, size).is_some() {
            return Err(fail());
        }
    }
    struct Host(measure::Size);
    impl measure::MeasurementHost for Host {
        type Error = ();
        fn measure_label(
            &mut self,
            q: &measure::LabelRequest<'_>,
        ) -> Result<measure::LabelObservation, ()> {
            Ok(measure::LabelObservation {
                target: q.target.clone(),
                stamp: q.stamp,
                bounds: Some(self.0),
            })
        }
    }
    let mut nodes = Vec::new();
    for e in &elements {
        if e.element_id.is_empty() || e.element_id.len() > 1024 {
            return Err(fail());
        }
        let observation = *bounds.get(e.element_id.as_str()).ok_or_else(fail)?;
        if let Some(parent) = &e.owner
            && !elements
                .iter()
                .any(|p| &p.element_id == parent && p.class == "Package")
        {
            return Err(fail());
        }
        let old = previous.and_then(|scene| scene.nodes.iter().find(|n| n.id == e.element_id));
        let prior = old
            .map(|n| measure::PreviousNode {
                id: &e.element_id,
                dimensions: measure::OptionalSize {
                    width: n.width,
                    height: n.height,
                },
            })
            .into_iter()
            .collect::<Vec<_>>();
        let parent_node = elements
            .iter()
            .any(|n| n.owner.as_deref() == Some(e.element_id.as_str()));
        // Parents take an explicit F18 header observation; F15 is leaf-only.
        let size = if parent_node {
            measure::Size {
                width: old.and_then(|n| n.width).unwrap_or(180.),
                height: old.and_then(|n| n.height).unwrap_or(70.),
            }
        } else {
            measure::measure_node(
                &mut Host(observation),
                measure::LayoutRequest {
                    node: measure::NodeGeometry {
                        id: &e.element_id,
                        kind: measure::NodeKind::Package,
                        current: old
                            .map(|n| measure::OptionalSize {
                                width: n.width,
                                height: n.height,
                            })
                            .unwrap_or_default(),
                        defaults: measure::OptionalSize {
                            width: Some(180.),
                            height: Some(70.),
                        },
                        forced: measure::OptionalSize::default(),
                        is_new: old.is_none(),
                        resized_by_user: old.is_some_and(|n| n.resized_by_user),
                        keep_aspect_ratio: false,
                        direct_children: 0,
                        border: measure::BorderWidths::default(),
                    },
                    previous: &prior,
                    visible_node_ids: &ids,
                    label_text: e.name.as_deref().unwrap_or("(unnamed)"),
                    font: measure::FontSpec {
                        family: "system-ui",
                        size_px: 14.,
                        weight: 400,
                        letter_spacing_px: 0.,
                        line_height_px: Some(24.),
                    },
                    stamp: measure::MeasurementStamp {
                        layout_revision: epoch,
                        font_revision: 1,
                        request_id: epoch,
                    },
                },
            )
            .map_err(|_| fail())?
            .size
        };
        if !size.width.is_finite()
            || !size.height.is_finite()
            || size.width <= 0.
            || size.height <= 0.
            || size.width.ceil() > 1_000_000.
            || size.height.ceil() > 1_000_000.
        {
            return Err(fail());
        }
        let header = observation.height.ceil() + 20.;
        if header > 1_000_000. {
            return Err(fail());
        }
        nodes.push(Node {
            id: e.element_id.clone(),
            position: Point::default(),
            width: Some(size.width.ceil()),
            height: Some(size.height.ceil()),
            resized_by_user: old.is_some_and(|n| n.resized_by_user),
            connection_handles: vec![],
            payload: Payload {
                parent: e.owner.clone(),
                header,
            },
        });
    }
    let mut edges = Vec::new();
    for e in doc.elements().iter().filter(|e| e.class == "FeatureTyping") {
        let data = &doc.element(&e.element_id).ok_or_else(fail)?["data"];
        let source = identity(&data["typedFeature"])?;
        let target = identity(&data["type"])?;
        let a = nodes.iter().find(|n| n.id == source).ok_or_else(fail)?;
        let b = nodes.iter().find(|n| n.id == target).ok_or_else(fail)?;
        if !elements
            .iter()
            .any(|e| e.element_id == source && e.class == "PartUsage")
            || !elements
                .iter()
                .any(|e| e.element_id == target && e.class == "PartDefinition")
            || a.payload.parent != b.payload.parent
        {
            return Err(fail());
        }
        edges.push(Edge {
            id: e.element_id.clone(),
            source: source.into(),
            target: target.into(),
            data: Some(EdgeData {
                bending_points: None,
                payload: (),
            }),
        });
    }
    if edges.len() > 64 {
        return Err(fail());
    }
    Ok(Diagram { nodes, edges })
}
pub fn computed(input: &Scene, selected: &[String]) -> Result<Scene, String> {
    if input.nodes.len() > 32 || input.edges.len() > 64 || selected.len() > 32 {
        return Err(fail());
    }
    let mut ids = BTreeSet::new();
    for n in &input.nodes {
        let (Some(w), Some(h)) = (n.width, n.height) else {
            return Err(fail());
        };
        if !n.connection_handles.is_empty()
            || n.id.is_empty()
            || n.id.len() > 1024
            || !ids.insert(&n.id)
            || !n.payload.header.is_finite()
            || n.payload.header < 0.
            || n.payload.header > 1_000_000.
            || [n.position.x, n.position.y, w, h]
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 1_000_000.)
            || w <= 0.
            || h <= 0.
            || w.fract() != 0.
            || h.fract() != 0.
        {
            return Err(fail());
        }
    }
    ids.clear();
    for e in &input.edges {
        if e.id.is_empty() || e.id.len() > 1024 || !ids.insert(&e.id) || e.data.is_none() {
            return Err(fail());
        }
        let a = input
            .nodes
            .iter()
            .find(|n| n.id == e.source)
            .ok_or_else(fail)?;
        let b = input
            .nodes
            .iter()
            .find(|n| n.id == e.target)
            .ok_or_else(fail)?;
        if a.payload.parent != b.payload.parent {
            return Err(fail());
        }
    }
    if selected.iter().any(|id| {
        !input.nodes.iter().any(|n| &n.id == id)
            || input
                .nodes
                .iter()
                .any(|n| n.payload.parent.as_deref() == Some(id))
    }) {
        return Err(fail());
    }
    let packages: Vec<_> = input
        .nodes
        .iter()
        .map(|n| PackageNode {
            id: &n.id,
            parent: n.payload.parent.as_deref(),
            leaf_size: Size {
                width: n.width.unwrap(),
                height: n.height.unwrap(),
            },
            header_height_footprint: n.payload.header,
            border_width: 1.,
            default_width: Some(180.),
            default_height: Some(70.),
            user_minimum: n.resized_by_user.then_some(Size {
                width: n.width.unwrap(),
                height: n.height.unwrap(),
            }),
            retained_position: if !selected.is_empty() && !selected.contains(&n.id) {
                let base = input
                    .nodes
                    .iter()
                    .find(|p| Some(p.id.as_str()) == n.payload.parent.as_deref())
                    .map(|p| p.position)
                    .unwrap_or_default();
                Some(syson_layout::containers::Position {
                    x: n.position.x - base.x,
                    y: n.position.y - base.y,
                })
            } else {
                None
            },
        })
        .collect();
    let graph: Vec<_> = input
        .edges
        .iter()
        .map(|e| syson_layout::DirectedEdge {
            id: &e.id,
            source: &e.source,
            target: &e.target,
        })
        .collect();
    let placed = layout_packages(
        &packages,
        &graph,
        syson_layout::Options::default(),
        syson_layout::Limits {
            max_nodes: 32,
            max_edges: 64,
            ..Default::default()
        },
        || false,
    )
    .map_err(|_| fail())?;
    // Previous routes are not inputs to this computation; do not copy their buffers.
    let mut result = Scene {
        nodes: input.nodes.clone(),
        edges: input
            .edges
            .iter()
            .map(|e| Edge {
                id: e.id.clone(),
                source: e.source.clone(),
                target: e.target.clone(),
                data: Some(EdgeData {
                    bending_points: None,
                    payload: (),
                }),
            })
            .collect(),
    };
    for node in &mut result.nodes {
        let p = placed
            .nodes
            .iter()
            .find(|p| p.id == node.id)
            .ok_or_else(fail)?;
        let values = [p.absolute.x, p.absolute.y, p.size.width, p.size.height];
        // F17 admits integers. Measurement is ceiled; fractional layout is refused.
        if values
            .iter()
            .any(|v| !v.is_finite() || v.fract() != 0. || *v < 0. || *v > 1_000_000.)
        {
            return Err(fail());
        }
        node.position = Point {
            x: p.absolute.x,
            y: p.absolute.y,
        };
        node.width = Some(p.size.width);
        node.height = Some(p.size.height);
    }
    let groups: BTreeSet<_> = result
        .nodes
        .iter()
        .map(|n| n.payload.parent.clone())
        .collect();
    for parent in groups {
        let siblings: Vec<_> = placed
            .nodes
            .iter()
            .filter(|n| n.parent == parent.as_deref())
            .map(|n| syson_layout::Placement {
                id: n.id,
                x: n.absolute.x,
                y: n.absolute.y,
                width: n.size.width,
                height: n.size.height,
                strong_component: 0,
                weak_component: 0,
                rank: 0,
            })
            .collect();
        let routes: Vec<_> = input
            .edges
            .iter()
            .filter(|e| siblings.iter().any(|n| n.id == e.source))
            .map(|e| RouteEdge {
                id: &e.id,
                source: &e.source,
                target: &e.target,
                source_side: Side::Right,
                target_side: Side::Left,
            })
            .collect();
        let routed = syson_routing::route(
            &siblings,
            &routes,
            syson_routing::Limits {
                max_nodes: 32,
                max_edges: 64,
                ..Default::default()
            },
            || false,
        )
        .map_err(|_| fail())?;
        for route in routed.edges {
            let edge = result.edges.iter_mut().find(|e| e.id == route.id).unwrap();
            edge.data.as_mut().unwrap().bending_points = Some(
                route
                    .points
                    .iter()
                    .map(|p| Point {
                        x: p.x2 as f64 / 2.,
                        y: p.y2 as f64 / 2.,
                    })
                    .collect(),
            );
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use syson_editor::prepare_rotation;
    use syson_model::{Editor, NodeIds};
    fn ids(n: u32) -> NodeIds {
        NodeIds::new(
            &format!("00000000-0000-4000-8000-{n:012x}"),
            &format!("10000000-0000-4000-8000-{n:012x}"),
        )
        .unwrap()
    }
    fn document() -> Document {
        let mut editor = Editor::new(Document::empty());
        let mut tx = editor.begin();
        tx.create_package(None, ids(1), None, "Vehicle").unwrap();
        tx.create_part_definition(&ids(1).element, ids(2), ids(3), "Wheel")
            .unwrap();
        tx.create_part_usage(&ids(1).element, ids(4), ids(5), "wheel")
            .unwrap();
        tx.type_part_usage(&ids(4).element, &ids(2).element, ids(6))
            .unwrap();
        editor.commit(tx).unwrap();
        editor.document().clone()
    }
    fn labels(doc: &Document) -> String {
        Value::Array(
            doc.elements()
                .iter()
                .filter(|e| supported(&e.class))
                .map(|e| json!({"id":e.element_id,"width":80.25,"height":24.25}))
                .collect(),
        )
        .to_string()
    }
    fn scene(doc: &Document) -> Scene {
        computed(&measured(doc, &labels(doc), 1, None).unwrap(), &[]).unwrap()
    }
    fn verify(scene: &Scene) {
        for node in &scene.nodes {
            let (w, h) = (node.width.unwrap(), node.height.unwrap());
            assert!(
                w > 0.
                    && h > 0.
                    && [node.position.x, node.position.y, w, h]
                        .iter()
                        .all(|v| v.is_finite() && v.fract() == 0.)
            );
            if let Some(parent) = &node.payload.parent {
                let p = scene.nodes.iter().find(|n| &n.id == parent).unwrap();
                assert!(node.position.x >= p.position.x && node.position.y >= p.position.y);
                assert!(node.position.x + w <= p.position.x + p.width.unwrap());
                assert!(node.position.y + h <= p.position.y + p.height.unwrap());
            }
        }
        for edge in &scene.edges {
            let a = scene.nodes.iter().find(|n| n.id == edge.source).unwrap();
            let b = scene.nodes.iter().find(|n| n.id == edge.target).unwrap();
            let points = edge.data.as_ref().unwrap().bending_points.as_ref().unwrap();
            assert_eq!(
                points.first(),
                Some(&Point {
                    x: a.position.x + a.width.unwrap(),
                    y: a.position.y + a.height.unwrap() / 2.
                })
            );
            assert_eq!(
                points.last(),
                Some(&Point {
                    x: b.position.x,
                    y: b.position.y + b.height.unwrap() / 2.
                })
            );
            for pair in points.windows(2) {
                assert_ne!(pair[0], pair[1]);
                assert!(pair[0].x == pair[1].x || pair[0].y == pair[1].y);
                for n in scene
                    .nodes
                    .iter()
                    .filter(|n| n.payload.parent == a.payload.parent)
                {
                    let (p, q) = (pair[0], pair[1]);
                    let crosses = if p.x == q.x {
                        n.position.x < p.x
                            && p.x < n.position.x + n.width.unwrap()
                            && p.y.max(q.y) > n.position.y
                            && p.y.min(q.y) < n.position.y + n.height.unwrap()
                    } else {
                        n.position.y < p.y
                            && p.y < n.position.y + n.height.unwrap()
                            && p.x.max(q.x) > n.position.x
                            && p.x.min(q.x) < n.position.x + n.width.unwrap()
                    };
                    assert!(!crosses, "route enters sibling interior");
                }
            }
        }
    }
    #[test]
    fn measured_package_part_canvas_routes_actual_typing_without_mutating_model() {
        let doc = document();
        let original = doc.to_json().unwrap();
        let result = scene(&doc);
        assert_eq!((result.nodes.len(), result.edges.len()), (3, 1));
        verify(&result);
        assert_eq!(doc.to_json().unwrap(), original);
        assert_eq!(result, scene(&doc));
    }
    #[test]
    fn label_observation_identity_shape_and_bounds_are_required() {
        let doc = document();
        let valid: Value = serde_json::from_str(&labels(&doc)).unwrap();
        for bad in ["[]".to_string(), "{bad".into(), "null".into()] {
            assert!(measured(&doc, &bad, 1, None).is_err());
        }
        for key in ["id", "width", "height"] {
            let mut bad = valid.clone();
            bad[0][key] = Value::Null;
            assert!(measured(&doc, &bad.to_string(), 1, None).is_err());
        }
        let mut bad = valid.clone();
        bad[1] = bad[0].clone();
        assert!(measured(&doc, &bad.to_string(), 1, None).is_err());
        for value in [-1., 0., 999_990., 1_000_001.] {
            let mut bad = valid.clone();
            bad[0]["height"] = json!(value);
            assert!(measured(&doc, &bad.to_string(), 1, None).is_err());
        }
    }
    #[test]
    fn nonpackage_ownership_and_cross_parent_typing_are_explicitly_unsupported() {
        let mut editor = Editor::new(document());
        let mut tx = editor.begin();
        tx.create_part_usage(&ids(2).element, ids(7), ids(8), "nested")
            .unwrap();
        editor.commit(tx).unwrap();
        assert!(measured(editor.document(), &labels(editor.document()), 1, None).is_err());
        let mut editor = Editor::new(document());
        let mut tx = editor.begin();
        tx.create_package(None, ids(10), None, "Other").unwrap();
        tx.create_part_usage(&ids(10).element, ids(11), ids(12), "foreign")
            .unwrap();
        tx.type_part_usage(&ids(11).element, &ids(2).element, ids(13))
            .unwrap();
        editor.commit(tx).unwrap();
        assert!(measured(editor.document(), &labels(editor.document()), 1, None).is_err());
    }
    #[test]
    fn unsupported_imported_classes_cannot_silently_disappear_from_canvas() {
        let doc = document();
        let mut raw = doc.value().clone();
        raw["content"][0]["eClass"] = serde_json::json!("sysml:PortDefinition");
        let unsupported = Document::from_json(&raw.to_string()).unwrap();
        let before = unsupported.to_json().unwrap();
        assert!(measured(&unsupported, &labels(&unsupported), 1, None).is_err());
        assert_eq!(unsupported.to_json().unwrap(), before);
    }
    #[test]
    fn geometry_api_rejects_missing_sizes_ids_parents_and_endpoints_without_panics() {
        let doc = document();
        let original = scene(&doc);
        let mut bad = original.clone();
        bad.nodes[0].width = None;
        assert!(computed(&bad, &[]).is_err());
        let mut bad = original.clone();
        bad.nodes[0].id = bad.nodes[1].id.clone();
        assert!(computed(&bad, &[]).is_err());
        let mut bad = original.clone();
        bad.nodes[0].payload.parent = Some("missing".into());
        assert!(computed(&bad, &[]).is_err());
        let mut bad = original.clone();
        bad.edges[0].target = "missing".into();
        assert!(computed(&bad, &[]).is_err());
        let mut bad = original.clone();
        bad.edges[0].data = None;
        assert!(computed(&bad, &[]).is_err());
        let mut bad = original.clone();
        bad.nodes[0].width = Some(f64::INFINITY);
        assert!(computed(&bad, &[]).is_err());
        let mut bad = original.clone();
        bad.nodes[0].connection_handles.push("port".into());
        assert!(computed(&bad, &[]).is_err());
        assert!(computed(&original, &vec![ids(4).element; 33]).is_err());
        let mut prior = original.clone();
        prior.edges[0].data.as_mut().unwrap().bending_points = Some(vec![
            Point {
                x: f64::NAN,
                y: f64::INFINITY
            };
            10_000
        ]);
        assert_eq!(computed(&prior, &[]).unwrap(), original);
        assert_eq!(original, scene(&doc));
    }
    #[test]
    fn leaf_rotation_computation_keeps_unselected_endpoints_at_retained_positions() {
        let doc = document();
        let original = scene(&doc);
        let selected = vec![ids(4).element];
        let prepared = prepare_rotation(&original, &selected);
        let result = computed(&prepared, &selected).unwrap();
        let target = result
            .nodes
            .iter()
            .find(|n| n.id == ids(2).element)
            .unwrap();
        let old = original.nodes.iter().find(|n| n.id == target.id).unwrap();
        assert_eq!(
            (target.position, target.width, target.height),
            (old.position, old.width, old.height)
        );
        let source = result.nodes.iter().find(|n| n.id == selected[0]).unwrap();
        assert_eq!((source.width, source.height), (Some(70.), Some(180.)));
        verify(&result);
        assert!(computed(&original, &[ids(1).element]).is_err());
    }
    #[test]
    fn full_layout_reapplies_real_f15_user_resized_leaf_minima() {
        let doc = document();
        let mut previous = scene(&doc);
        let usage = previous
            .nodes
            .iter_mut()
            .find(|n| n.id == ids(4).element)
            .unwrap();
        usage.width = Some(70.);
        usage.height = Some(180.);
        usage.resized_by_user = true;
        let next = measured(&doc, &labels(&doc), 2, Some(&previous)).unwrap();
        let usage = next.nodes.iter().find(|n| n.id == ids(4).element).unwrap();
        assert_eq!((usage.width, usage.height), (Some(75.), Some(180.)));
        verify(&computed(&next, &[]).unwrap());
    }
    #[test]
    fn fractional_layout_and_limits_are_refused_instead_of_rounding_routes() {
        let doc = document();
        let mut input = measured(&doc, &labels(&doc), 1, None).unwrap();
        input
            .nodes
            .iter_mut()
            .find(|n| n.id == ids(4).element)
            .unwrap()
            .height = Some(71.);
        // Height71 is valid; it must not be rejected to satisfy a fixture assumption.
        verify(&computed(&input, &[]).unwrap());
        let mut retained = scene(&doc);
        retained
            .nodes
            .iter_mut()
            .find(|n| n.id == ids(2).element)
            .unwrap()
            .position
            .x += 0.5;
        assert!(computed(&retained, &[ids(4).element]).is_err());
        let mut fractional = scene(&doc);
        fractional.nodes[0].width = Some(70.5);
        assert!(computed(&fractional, &[]).is_err());
        let mut many = scene(&doc);
        many.nodes = vec![many.nodes[0].clone(); 33];
        assert!(computed(&many, &[]).is_err());
        assert!(measured(&doc, &"x".repeat(1_048_577), 1, None).is_err());
    }
    #[test]
    fn empty_document_has_a_real_empty_layout() {
        let doc = Document::empty();
        let result = computed(&measured(&doc, "[]", 1, None).unwrap(), &[]).unwrap();
        assert!(result.nodes.is_empty() && result.edges.is_empty());
    }
}
