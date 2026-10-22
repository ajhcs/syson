// Copyright (c) 2026 Rust port contributors.
// SPDX-License-Identifier: EPL-2.0
use syson_editor::*;
fn node(id: &str) -> Node<String> {
    Node {
        id: id.into(),
        position: Point { x: 10., y: 20. },
        width: Some(100.),
        height: Some(40.),
        resized_by_user: false,
        connection_handles: vec!["north".into()],
        payload: format!("opaque:{id}"),
    }
}
fn fixture() -> Diagram<String, String> {
    Diagram {
        nodes: vec![node("a"), node("b"), node("c")],
        edges: vec![
            Edge {
                id: "ab".into(),
                source: "a".into(),
                target: "b".into(),
                data: Some(EdgeData {
                    bending_points: Some(vec![Point { x: 1., y: 2. }]),
                    payload: "edge".into(),
                }),
            },
            Edge {
                id: "bc".into(),
                source: "b".into(),
                target: "c".into(),
                data: None,
            },
        ],
    }
}
fn selected(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}
#[test]
fn rotation_preserves_center_payload_and_only_rotates_first_selection() {
    let before = fixture();
    let after = prepare_rotation(&before, &selected(&["a", "b"]));
    assert_eq!(after.nodes[0].position, Point { x: 40., y: -10. });
    assert_eq!(
        (after.nodes[0].width, after.nodes[0].height),
        (Some(40.), Some(100.))
    );
    assert!(after.nodes[0].resized_by_user);
    assert!(after.nodes[0].connection_handles.is_empty());
    assert_eq!(after.nodes[0].payload, before.nodes[0].payload);
    assert_eq!(after.nodes[1], before.nodes[1]);
    assert_eq!(after.edges[0].data.as_ref().unwrap().bending_points, None);
    assert_eq!(
        before.edges[0]
            .data
            .as_ref()
            .unwrap()
            .bending_points
            .as_ref()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(after.edges[1], before.edges[1]);
}
#[test]
fn empty_or_unknown_selection_does_not_modify_snapshot() {
    let before = fixture();
    assert_eq!(prepare_rotation(&before, &[]), before);
    assert_eq!(prepare_rotation(&before, &selected(&["missing"])), before);
}
#[test]
fn absent_dimensions_use_zero() {
    let mut before = fixture();
    before.nodes[0].width = None;
    let after = prepare_rotation(&before, &selected(&["a"]));
    assert_eq!(after.nodes[0].position, Point { x: -10., y: 40. });
    assert_eq!(after.nodes[0].height, Some(0.));
}
#[test]
fn two_rotations_restore_geometry_but_do_not_restore_handles_or_bends() {
    let before = fixture();
    let ids = selected(&["a"]);
    let after = prepare_rotation(&prepare_rotation(&before, &ids), &ids);
    assert_eq!(after.nodes[0].position, before.nodes[0].position);
    assert_eq!(after.nodes[0].width, before.nodes[0].width);
    assert!(after.nodes[0].connection_handles.is_empty());
}
#[test]
fn layout_merge_adopts_all_selected_and_incident_edges_only() {
    let prepared = prepare_rotation(&fixture(), &selected(&["a", "b"]));
    let mut layout = prepared.clone();
    for n in &mut layout.nodes {
        n.position.x = 999.;
    }
    layout.edges[1].data = Some(EdgeData {
        bending_points: Some(vec![]),
        payload: "laid-out".into(),
    });
    layout.nodes.reverse();
    layout.edges.reverse();
    let result = finish_rotation(&prepared, &layout, &selected(&["a", "b"]));
    assert_eq!(result.nodes[0].position.x, 999.);
    assert_eq!(result.nodes[1].position.x, 999.);
    assert_eq!(result.nodes[2], prepared.nodes[2]);
    assert_eq!(result.edges[1].data.as_ref().unwrap().payload, "laid-out");
    assert_eq!(result.nodes[0].id, "a");
}
#[test]
fn layout_merge_missing_results_falls_back_without_deleting_objects() {
    let before = fixture();
    let empty = Diagram {
        nodes: vec![],
        edges: vec![],
    };
    assert_eq!(finish_rotation(&before, &empty, &selected(&["a"])), before);
}
#[test]
fn leaf_defaults_measurement_and_explicit_zero_defaults() {
    assert_eq!(
        layout_package_leaf(LeafLayout::default()),
        Size {
            width: 75.,
            height: 50.
        }
    );
    assert_eq!(
        layout_package_leaf(LeafLayout {
            measured_label_height: Some(80.),
            default_width: Some(0.),
            ..Default::default()
        }),
        Size {
            width: 0.,
            height: 96.
        }
    );
}
#[test]
fn user_resized_leaves_retain_larger_old_size_and_ignore_defaults() {
    assert_eq!(
        layout_package_leaf(LeafLayout {
            resized_by_user: true,
            previous_size: Size {
                width: 300.,
                height: 200.
            },
            default_width: Some(900.),
            default_height: Some(900.),
            ..Default::default()
        }),
        Size {
            width: 300.,
            height: 200.
        }
    );
}
#[test]
fn forced_dimensions_suppress_retention_but_are_not_applied_as_leaf_sizes() {
    let base = LeafLayout {
        resized_by_user: true,
        previous_size: Size {
            width: 300.,
            height: 200.,
        },
        ..Default::default()
    };
    assert_eq!(
        layout_package_leaf(LeafLayout {
            forced_width: Some(800.),
            forced_height: Some(700.),
            ..base
        }),
        Size {
            width: 75.,
            height: 50.
        }
    );
    assert_eq!(
        layout_package_leaf(LeafLayout {
            forced_width: Some(0.),
            forced_height: Some(0.),
            ..base
        }),
        Size {
            width: 300.,
            height: 200.
        }
    );
}
#[test]
fn note_regression_background_change_preserves_border_and_container_transparency() {
    let before = note_appearance("#000000", "#ffffff", "#123456", false, false, false);
    let after = note_appearance("#000000", "#ffff00", "#123456", false, false, false);
    assert_eq!(after.stroke, before.stroke);
    assert_eq!(after.fill, "#ffff00");
    assert_eq!(after.container_background, "transparent");
    assert_eq!(after.container_border, "none");
}
#[test]
fn note_paths_and_faded_selection() {
    let (body, fold) = note_paths(200., 70., 3.9);
    assert_eq!(body, "M 1.5,1.5 H 185 L 198.5 15 V 68.5 H 1.5 Z");
    assert_eq!(fold, "M 185,1.5 V 15 H 198.5");
    let style = note_appearance("a", "b", "selection", false, true, true);
    assert_eq!(style.fill_opacity, 0.4);
    assert_eq!(style.stroke_opacity, 0.4);
    assert_eq!(style.container_opacity, Some(0.4));
    assert_eq!(style.outline, Some("selection"));
}
