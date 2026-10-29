// SPDX-License-Identifier: EPL-2.0
use syson_layout::containers::{
    Error, PackageLayout, PackageNode, Position, Size, layout_packages,
};
use syson_layout::{DirectedEdge, Direction, LayoutError, Limit, Limits, Options};

fn node<'a>(id: &'a str, parent: Option<&'a str>) -> PackageNode<'a> {
    PackageNode {
        id,
        parent,
        leaf_size: Size {
            width: 30.,
            height: 10.,
        },
        header_height_footprint: 20.,
        border_width: 2.,
        default_width: Some(0.),
        default_height: Some(0.),
        user_minimum: None,
        retained_position: None,
    }
}
fn options() -> Options {
    Options {
        padding: 0.,
        ..Options::default()
    }
}
fn run<'a>(nodes: &[PackageNode<'a>], edges: &[DirectedEdge<'a>]) -> PackageLayout<'a> {
    layout_packages(nodes, edges, options(), Limits::default(), || false).unwrap()
}
fn verify(result: &PackageLayout<'_>, fresh: bool) {
    for child in &result.nodes {
        assert!(
            [
                child.absolute.x,
                child.absolute.y,
                child.relative.x,
                child.relative.y,
                child.size.width,
                child.size.height
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.)
        );
        if let Some(parent_id) = child.parent {
            let parent = result.nodes.iter().find(|n| n.id == parent_id).unwrap();
            assert_eq!(child.absolute.x, parent.absolute.x + child.relative.x);
            assert_eq!(child.absolute.y, parent.absolute.y + child.relative.y);
            assert!(child.relative.x + child.size.width <= parent.size.width);
            assert!(child.relative.y + child.size.height <= parent.size.height);
        } else {
            assert_eq!(child.absolute, child.relative);
        }
        if fresh {
            for other in &result.nodes {
                if child.id < other.id && child.parent == other.parent {
                    assert!(
                        child.relative.x + child.size.width <= other.relative.x
                            || other.relative.x + other.size.width <= child.relative.x
                            || child.relative.y + child.size.height <= other.relative.y
                            || other.relative.y + other.size.height <= child.relative.y
                    );
                }
            }
        }
    }
}
#[test]
fn nested_offsets_are_relative_and_absolute_and_parent_bounds_follow_children() {
    let mut grand = node("grand", None);
    grand.header_height_footprint = 10.;
    grand.border_width = 1.;
    let nodes = [
        node("leaf", Some("parent")),
        node("parent", Some("grand")),
        grand,
    ];
    let result = run(&nodes, &[]);
    assert_eq!(
        result.nodes.iter().map(|n| n.id).collect::<Vec<_>>(),
        ["grand", "leaf", "parent"]
    );
    let parent = result.nodes.iter().find(|n| n.id == "parent").unwrap();
    let leaf = result.nodes.iter().find(|n| n.id == "leaf").unwrap();
    assert_eq!(
        parent.size,
        Size {
            width: 52.,
            height: 52.
        }
    );
    assert_eq!(parent.relative, Position { x: 9., y: 19. });
    assert_eq!(leaf.relative, Position { x: 10., y: 30. });
    assert_eq!(leaf.absolute, Position { x: 19., y: 49. });
    assert_eq!(
        result.nodes[0].size,
        Size {
            width: 71.,
            height: 81.
        }
    );
    assert_eq!((result.width, result.height), (71., 81.));
    verify(&result, true);
}
#[test]
fn previous_branch_uses_header_plus_padding_without_border_and_clamps_negative_x() {
    let parent = node("parent", None);
    let mut child = node("child", Some("parent"));
    child.retained_position = Some(Position { x: -3., y: -4. });
    let result = run(&[parent, child], &[]);
    assert_eq!(result.nodes[0].relative, Position { x: 8., y: 28. });
    assert_eq!(
        result.nodes[1].size,
        Size {
            width: 50.,
            height: 50.
        }
    );
    verify(&result, false);
}
#[test]
fn resized_parent_uses_previous_minimum_instead_of_default_minimum() {
    let mut parent = node("parent", None);
    parent.default_width = Some(9000.);
    parent.default_height = Some(5000.);
    parent.user_minimum = Some(Size {
        width: 1000.,
        height: 200.,
    });
    let child = node("child", Some("parent"));
    let resized = run(&[parent, child], &[]);
    assert_eq!(
        resized.nodes[1].size,
        Size {
            width: 1000.,
            height: 200.
        }
    );
    parent.user_minimum = Some(Size {
        width: 1.,
        height: 1.,
    });
    assert_eq!(
        run(&[parent, child], &[]).nodes[1].size,
        Size {
            width: 52.,
            height: 52.
        }
    );
    parent.user_minimum = None;
    assert_eq!(
        run(&[parent, child], &[]).nodes[1].size,
        Size {
            width: 9004.,
            height: 5004.
        }
    );
}
#[test]
fn nullish_default_axes_and_parent_to_leaf_transition_are_distinct() {
    let mut parent = node("parent", None);
    parent.default_width = None;
    parent.default_height = None;
    let child = node("child", Some("parent"));
    assert_eq!(
        run(&[parent, child], &[]).nodes[1].size,
        Size {
            width: 150.,
            height: 70.
        }
    );
    parent.default_width = Some(0.);
    assert_eq!(
        run(&[parent, child], &[]).nodes[1].size,
        Size {
            width: 52.,
            height: 70.
        }
    );
    parent.user_minimum = Some(Size {
        width: 500.,
        height: 500.,
    });
    assert_eq!(run(&[parent], &[]).nodes[0].size, parent.leaf_size);
}
#[test]
fn repeated_incremental_inputs_preserve_geometry_and_expand_for_larger_children() {
    let mut nodes = [node("parent", None), node("child", Some("parent"))];
    let first = run(&nodes, &[]);
    for n in &mut nodes {
        n.retained_position = Some(first.nodes.iter().find(|p| p.id == n.id).unwrap().relative);
    }
    let repeat = run(&nodes, &[]);
    assert_eq!(repeat.nodes, first.nodes);
    nodes[1].leaf_size.width = 100.;
    let enlarged = run(&nodes, &[]);
    assert_eq!(enlarged.nodes[1].size.width, 122.);
    assert_eq!(enlarged.nodes[0].relative, first.nodes[0].relative);
    assert_eq!(run(&nodes, &[]), enlarged);
}
#[test]
fn retained_overlapping_siblings_are_not_silently_reflowed() {
    let mut a = node("a", Some("p"));
    let mut b = node("b", Some("p"));
    a.retained_position = Some(Position { x: 12., y: 40. });
    b.retained_position = a.retained_position;
    let result = run(&[node("p", None), a, b], &[]);
    assert_eq!(result.nodes[0].relative, result.nodes[1].relative);
    verify(&result, false);
}
#[test]
fn sibling_edges_feed_f16_and_permutations_preserve_output() {
    let nodes = [node("p", None), node("a", Some("p")), node("b", Some("p"))];
    let edges = [DirectedEdge {
        id: "ab",
        source: "a",
        target: "b",
    }];
    let first = run(&nodes, &edges);
    assert!(first.nodes[0].relative.x + first.nodes[0].size.width < first.nodes[1].relative.x);
    let reversed = [nodes[2], nodes[1], nodes[0]];
    assert_eq!(run(&reversed, &edges), first);
    verify(&first, true);
}
#[test]
fn unknown_parent_cycles_and_cross_parent_edges_are_separate_errors() {
    assert_eq!(
        layout_packages(
            &[node("a", Some("missing"))],
            &[],
            options(),
            Limits::default(),
            || false
        ),
        Err(Error::UnknownParent)
    );
    assert_eq!(
        layout_packages(
            &[node("a", Some("a"))],
            &[],
            options(),
            Limits::default(),
            || false
        ),
        Err(Error::ParentCycle)
    );
    assert_eq!(
        layout_packages(
            &[node("a", Some("b")), node("b", Some("a"))],
            &[],
            options(),
            Limits::default(),
            || false
        ),
        Err(Error::ParentCycle)
    );
    let nodes = [node("p", None), node("c", Some("p"))];
    let edges = [DirectedEdge {
        id: "pc",
        source: "p",
        target: "c",
    }];
    assert_eq!(
        layout_packages(&nodes, &edges, options(), Limits::default(), || false),
        Err(Error::CrossParentEdge)
    );
}
#[test]
fn identity_and_endpoint_errors_are_rejected_without_partial_results() {
    fn bad<'a>(
        nodes: &[PackageNode<'a>],
        edges: &[DirectedEdge<'a>],
    ) -> Result<PackageLayout<'a>, Error> {
        layout_packages(nodes, edges, options(), Limits::default(), || false)
    }
    assert_eq!(
        bad(&[node("", None)], &[]),
        Err(Error::Layout(LayoutError::EmptyId))
    );
    assert_eq!(
        bad(&[node("a", None), node("a", None)], &[]),
        Err(Error::Layout(LayoutError::DuplicateNode))
    );
    let edge = DirectedEdge {
        id: "e",
        source: "a",
        target: "missing",
    };
    assert_eq!(
        bad(&[node("a", None)], &[edge]),
        Err(Error::Layout(LayoutError::UnknownEndpoint))
    );
    let edge = DirectedEdge {
        target: "a",
        ..edge
    };
    assert_eq!(
        bad(&[node("a", None)], &[edge, edge]),
        Err(Error::Layout(LayoutError::DuplicateEdge))
    );
}
#[test]
fn malformed_unused_geometry_is_explicit_input_restriction() {
    for value in [f64::NAN, f64::INFINITY, -1.] {
        let mut parent = node("p", None);
        parent.leaf_size.width = value;
        assert_eq!(
            layout_packages(
                &[parent, node("c", Some("p"))],
                &[],
                options(),
                Limits::default(),
                || false
            ),
            Err(Error::Layout(LayoutError::InvalidNumber))
        );
    }
    let mut leaf = node("a", None);
    leaf.retained_position = Some(Position {
        x: f64::NEG_INFINITY,
        y: 0.,
    });
    assert_eq!(
        layout_packages(&[leaf], &[], options(), Limits::default(), || false),
        Err(Error::Layout(LayoutError::InvalidNumber))
    );
    leaf.retained_position = None;
    leaf.border_width = f64::MAX;
    let result = layout_packages(
        &[leaf, node("c", Some("a"))],
        &[],
        options(),
        Limits::default(),
        || false,
    );
    assert!(matches!(
        result,
        Err(Error::Layout(
            LayoutError::ArithmeticOverflow | LayoutError::PrecisionLoss
        ))
    ));
}
#[test]
fn all_logical_limits_and_exact_work_boundary_are_enforced() {
    let nodes = [node("a", None), node("b", None)];
    let edges = [DirectedEdge {
        id: "ab",
        source: "a",
        target: "b",
    }];
    for (limits, kind) in [
        (
            Limits {
                max_nodes: 1,
                ..Limits::default()
            },
            Limit::Nodes,
        ),
        (
            Limits {
                max_edges: 0,
                ..Limits::default()
            },
            Limit::Edges,
        ),
        (
            Limits {
                max_workspace_bytes: 0,
                ..Limits::default()
            },
            Limit::Memory,
        ),
        (
            Limits {
                max_work: 0,
                ..Limits::default()
            },
            Limit::Work,
        ),
        (
            Limits {
                max_id_bytes: 0,
                ..Limits::default()
            },
            Limit::IdBytes,
        ),
    ] {
        assert_eq!(
            layout_packages(&nodes, &edges, options(), limits, || false),
            Err(Error::Layout(LayoutError::Limit(kind)))
        );
    }
    let good = run(&nodes, &edges);
    let exact = Limits {
        max_work: good.charged_work,
        max_workspace_bytes: good.estimated_workspace_bytes,
        ..Limits::default()
    };
    assert_eq!(
        layout_packages(&nodes, &edges, options(), exact, || false).unwrap(),
        good
    );
    assert_eq!(
        layout_packages(
            &nodes,
            &edges,
            options(),
            Limits {
                max_work: exact.max_work - 1,
                ..exact
            },
            || false
        ),
        Err(Error::Layout(LayoutError::Limit(Limit::Work)))
    );
}
#[test]
fn initial_and_late_cancellation_return_no_partial_layout() {
    let nodes = [node("p", None), node("c", Some("p"))];
    assert_eq!(
        layout_packages(&nodes, &[], options(), Limits::default(), || true),
        Err(Error::Layout(LayoutError::Cancelled))
    );
    let mut polls = 0;
    layout_packages(&nodes, &[], options(), Limits::default(), || {
        polls += 1;
        false
    })
    .unwrap();
    let mut late = 0;
    assert_eq!(
        layout_packages(&nodes, &[], options(), Limits::default(), || {
            late += 1;
            late == polls - 1
        }),
        Err(Error::Layout(LayoutError::Cancelled))
    );
}
#[test]
fn empty_forest_is_empty_and_options_remain_validated() {
    let empty = run(&[], &[]);
    assert!(empty.nodes.is_empty());
    assert_eq!((empty.width, empty.height), (0., 0.));
    assert_eq!(
        layout_packages(
            &[],
            &[],
            Options {
                padding: f64::NAN,
                ..options()
            },
            Limits::default(),
            || false
        ),
        Err(Error::Layout(LayoutError::InvalidNumber))
    );
}
#[test]
fn finite_forests_in_both_directions_have_containment_and_fresh_nonoverlap() {
    let mut scenarios = 0;
    for direction in [Direction::Right, Direction::Down] {
        for depth in 1..=4 {
            for width in [10., 20., 30.] {
                let ids: Vec<_> = (0..=depth).map(|i| format!("n{i}")).collect();
                let mut nodes = Vec::new();
                for i in 0..=depth {
                    let mut n = node(&ids[i], if i == 0 { None } else { Some(&ids[i - 1]) });
                    n.leaf_size.width = width;
                    nodes.push(n);
                }
                let result = layout_packages(
                    &nodes,
                    &[],
                    Options {
                        direction,
                        ..options()
                    },
                    Limits::default(),
                    || false,
                )
                .unwrap();
                verify(&result, true);
                scenarios += 1;
            }
        }
    }
    assert_eq!(scenarios, 24);
}
#[test]
fn actual_f15_measurement_supplies_leaf_size_without_dom_claims() {
    use syson_node_measurement::*;
    struct Host;
    impl MeasurementHost for Host {
        type Error = ();
        fn measure_label(&mut self, request: &LabelRequest<'_>) -> Result<LabelObservation, ()> {
            Ok(LabelObservation {
                target: request.target.clone(),
                stamp: request.stamp,
                bounds: Some(syson_node_measurement::Size {
                    width: 60.,
                    height: 42.,
                }),
            })
        }
    }
    let request = LayoutRequest {
        node: NodeGeometry {
            id: "child",
            kind: NodeKind::Package,
            current: OptionalSize::default(),
            defaults: OptionalSize::default(),
            forced: OptionalSize::default(),
            is_new: true,
            resized_by_user: false,
            keep_aspect_ratio: false,
            direct_children: 0,
            border: BorderWidths::default(),
        },
        previous: &[],
        visible_node_ids: &["child"],
        label_text: "label",
        font: FontSpec {
            family: "Synthetic",
            size_px: 12.,
            weight: 400,
            letter_spacing_px: 0.,
            line_height_px: None,
        },
        stamp: MeasurementStamp {
            layout_revision: 1,
            font_revision: 2,
            request_id: 3,
        },
    };
    let measured = measure_node(&mut Host, request).unwrap();
    let mut child = node("child", Some("parent"));
    child.leaf_size = syson_layout::containers::Size {
        width: measured.size.width,
        height: measured.size.height,
    };
    let result = run(&[node("parent", None), child], &[]);
    assert_eq!(result.nodes[0].size.width, 75.);
    assert_eq!(result.nodes[0].size.height, 58.);
    assert_eq!(
        result.nodes[1].size,
        syson_layout::containers::Size {
            width: 97.,
            height: 100.
        }
    );
    verify(&result, true);
}
