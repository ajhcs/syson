// SPDX-License-Identifier: EPL-2.0
use std::cell::Cell;
use syson_layout::{DirectedEdge, MeasuredNode, Options, Placement};
use syson_routing::{Edge, Error, Limit, Limits, Point, Side, Unsupported, admit};

fn node(id: &str, x: f64, y: f64, width: f64, height: f64) -> Placement<'_> {
    Placement {
        id,
        x,
        y,
        width,
        height,
        strong_component: 0,
        weak_component: 0,
        rank: 0,
    }
}
fn pair() -> [Placement<'static>; 2] {
    [
        node("a", 0.0, 0.0, 4.0, 3.0),
        node("b", 20.0, 0.0, 4.0, 3.0),
    ]
}
fn edge(id: &str) -> Edge<'_> {
    Edge {
        id,
        source: "a",
        target: "b",
        source_side: Side::Right,
        target_side: Side::Left,
    }
}
fn refusal(nodes: &[Placement<'_>], edges: &[Edge<'_>], limits: Limits) -> Error {
    admit(nodes, edges, limits, || false).unwrap_err()
}

#[test]
fn defaults_and_deferred_search_quotas_are_preserved() {
    let limits = Limits::default();
    assert_eq!((limits.max_nodes, limits.max_edges), (128, 256));
    assert_eq!(
        (limits.max_grid_vertices, limits.max_output_points),
        (16_384, 65_536)
    );
    assert_eq!((limits.max_work, limits.max_id_bytes), (20_000_000, 1024));
    let deferred = Limits {
        max_grid_vertices: 0,
        max_output_points: 0,
        ..limits
    };
    let admitted = admit(&pair(), &[edge("e")], deferred, || false).unwrap();
    assert_eq!(admitted.limits().max_grid_vertices, 0);
    assert_eq!(admitted.limits().max_output_points, 0);
    assert_eq!(admitted.limits().max_work, limits.max_work);
}

#[test]
fn canonical_views_preserve_half_units_and_input_bytes() {
    let nodes = pair();
    let edges = [edge("z"), edge("e")];
    let snapshot = edges.map(|e| (e.id, e.source, e.target, e.source_side, e.target_side));
    let admitted = admit(&nodes, &edges, Limits::default(), || false).unwrap();
    let rectangles: Vec<_> = admitted.rectangles().collect();
    assert_eq!(rectangles, [("a", [0, 8, 0, 6]), ("b", [40, 48, 0, 6])]);
    assert_eq!(
        admitted.terminals(),
        [
            ("e", Point { x2: 8, y2: 3 }, Point { x2: 40, y2: 3 }),
            ("z", Point { x2: 8, y2: 3 }, Point { x2: 40, y2: 3 }),
        ]
    );
    let permuted_nodes = [nodes[1], nodes[0]];
    let permuted_edges = [edges[1], edges[0]];
    let other = admit(&permuted_nodes, &permuted_edges, Limits::default(), || {
        false
    })
    .unwrap();
    assert_eq!(other.rectangles().collect::<Vec<_>>(), rectangles);
    assert_eq!(other.terminals(), admitted.terminals());
    assert_eq!(other.charged_work(), admitted.charged_work());
    assert_eq!(nodes, pair());
    assert_eq!(
        edges.map(|e| (e.id, e.source, e.target, e.source_side, e.target_side)),
        snapshot
    );
}

#[test]
fn owned_geometry_outlives_input_slices_and_borrows_ids() {
    let id = String::from("a");
    let admitted = {
        let nodes = [node(&id, 0.0, 0.0, 1.0, 1.0)];
        admit(&nodes, &[], Limits::default(), || false).unwrap()
    };
    let (actual, bounds) = admitted.rectangles().next().unwrap();
    assert_eq!(actual.as_ptr(), id.as_ptr());
    assert_eq!(bounds, [0, 2, 0, 2]);
}

#[test]
fn all_side_centers_are_exact_for_negative_odd_rectangles() {
    let nodes = [
        node("a", -3.0, -5.0, 3.0, 5.0),
        node("b", 10.0, 10.0, 3.0, 5.0),
    ];
    for (side, expected) in [
        (Side::Left, Point { x2: -6, y2: -5 }),
        (Side::Right, Point { x2: 0, y2: -5 }),
        (Side::Top, Point { x2: -3, y2: -10 }),
        (Side::Bottom, Point { x2: -3, y2: 0 }),
    ] {
        let e = Edge {
            source_side: side,
            ..edge("e")
        };
        let admitted = admit(&nodes, &[e], Limits::default(), || false).unwrap();
        assert_eq!(admitted.terminals()[0].1, expected);
    }
}

#[test]
fn node_edge_and_utf8_identity_limits_have_exact_boundaries() {
    let exact = Limits {
        max_nodes: 2,
        max_edges: 1,
        max_id_bytes: 1,
        ..Limits::default()
    };
    assert!(admit(&pair(), &[edge("e")], exact, || false).is_ok());
    assert_eq!(
        refusal(
            &pair(),
            &[edge("e")],
            Limits {
                max_nodes: 1,
                ..exact
            }
        ),
        Error::Limit(Limit::Nodes)
    );
    assert_eq!(
        refusal(
            &pair(),
            &[edge("e")],
            Limits {
                max_edges: 0,
                ..exact
            }
        ),
        Error::Limit(Limit::Edges)
    );
    assert_eq!(
        refusal(&pair(), &[edge("ee")], exact),
        Error::Limit(Limit::IdBytes)
    );
    let unicode = [node("é", 0.0, 0.0, 1.0, 1.0)];
    assert!(
        admit(
            &unicode,
            &[],
            Limits {
                max_id_bytes: 2,
                ..exact
            },
            || false
        )
        .is_ok()
    );
    assert_eq!(refusal(&unicode, &[], exact), Error::Limit(Limit::IdBytes));
    assert_eq!(
        refusal(&[node("", 0.0, 0.0, 1.0, 1.0)], &[], exact),
        Error::EmptyId
    );
    assert_eq!(refusal(&pair(), &[edge("")], exact), Error::EmptyId);
}

#[test]
fn duplicates_and_missing_endpoints_are_distinct_refusals() {
    let n = pair();
    assert_eq!(
        refusal(&[n[0], n[0]], &[], Limits::default()),
        Error::DuplicateNode
    );
    assert_eq!(
        refusal(&n, &[edge("e"), edge("e")], Limits::default()),
        Error::DuplicateEdge
    );
    for (source, target) in [("missing", "b"), ("a", "missing"), ("", "b"), ("a", "")] {
        let e = Edge {
            source,
            target,
            ..edge("e")
        };
        assert_eq!(refusal(&n, &[e], Limits::default()), Error::UnknownEndpoint);
    }
}

#[test]
fn exact_work_and_checked_sort_overflow_preserve_donor_charges() {
    let exact = Limits {
        max_id_bytes: 1,
        max_work: 13,
        ..Limits::default()
    };
    let admitted = admit(&pair(), &[edge("e")], exact, || false).unwrap();
    // Two ID bytes + node sort 4 + overlap 1 + edge byte/sort 2 + endpoints 4.
    assert_eq!(admitted.charged_work(), 13);
    assert_eq!(
        refusal(
            &pair(),
            &[edge("e")],
            Limits {
                max_work: 12,
                ..exact
            }
        ),
        Error::Limit(Limit::Work)
    );
    assert_eq!(
        refusal(
            &pair(),
            &[],
            Limits {
                max_id_bytes: usize::MAX,
                max_work: usize::MAX,
                ..exact
            }
        ),
        Error::Limit(Limit::Work)
    );
}

#[test]
fn nonfinite_fractional_and_degenerate_components_are_refused() {
    let geometry = Error::Unsupported(Unsupported::Geometry);
    for field in 0..4 {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.5] {
            let mut p = node("a", 0.0, 0.0, 4.0, 3.0);
            match field {
                0 => p.x = value,
                1 => p.y = value,
                2 => p.width = value,
                _ => p.height = value,
            }
            assert_eq!(refusal(&[p], &[], Limits::default()), geometry);
        }
    }
    for (width, height) in [
        (0.0, 1.0),
        (1.0, 0.0),
        (-1.0, 1.0),
        (1.0, -1.0),
        (-0.0, 1.0),
    ] {
        assert_eq!(
            refusal(
                &[node("a", 0.0, 0.0, width, height)],
                &[],
                Limits::default()
            ),
            geometry
        );
    }
    let admitted = admit(
        &[node("a", -0.0, -0.0, 1.0, 1.0)],
        &[],
        Limits::default(),
        || false,
    )
    .unwrap();
    assert_eq!(admitted.rectangles().next(), Some(("a", [0, 2, 0, 2])));
}

#[test]
fn rectangle_bounds_are_checked_at_the_original_coordinate_ceiling() {
    let exact = [node(
        "a",
        -1_000_000.0,
        -1_000_000.0,
        2_000_000.0,
        2_000_000.0,
    )];
    let admitted = admit(&exact, &[], Limits::default(), || false).unwrap();
    assert_eq!(
        admitted.rectangles().next(),
        Some(("a", [-2_000_000, 2_000_000, -2_000_000, 2_000_000]))
    );
    for p in [
        node("a", 1_000_000.0, 0.0, 1.0, 1.0),
        node("a", -1_000_001.0, 0.0, 1.0, 1.0),
        node("a", 0.0, 1_000_000.0, 1.0, 1.0),
        node("a", 0.0, -1_000_001.0, 1.0, 1.0),
        node("a", 0.0, 0.0, 2_000_001.0, 1.0),
    ] {
        assert_eq!(
            refusal(&[p], &[], Limits::default()),
            Error::Unsupported(Unsupported::Geometry)
        );
    }
}

#[test]
fn overlap_refuses_but_boundary_corner_and_parallel_terminals_remain_supported() {
    let a = node("a", 0.0, 0.0, 4.0, 4.0);
    assert_eq!(
        refusal(&[a, node("b", 3.0, 3.0, 4.0, 4.0)], &[], Limits::default()),
        Error::Unsupported(Unsupported::OverlappingInteriors)
    );
    for b in [node("b", 4.0, 0.0, 4.0, 4.0), node("b", 4.0, 4.0, 4.0, 4.0)] {
        assert!(admit(&[a, b], &[], Limits::default(), || false).is_ok());
    }
    assert_eq!(
        refusal(
            &[a, node("b", 4.0, 0.0, 4.0, 4.0)],
            &[edge("e")],
            Limits::default()
        ),
        Error::Unsupported(Unsupported::CoincidentAnchors)
    );
    let admitted = admit(&pair(), &[edge("e"), edge("f")], Limits::default(), || {
        false
    })
    .unwrap();
    assert_eq!(admitted.terminals()[0].1, admitted.terminals()[1].1);
    assert_eq!(admitted.terminals()[0].2, admitted.terminals()[1].2);
}

#[test]
fn cancellation_covers_entry_each_charged_phase_and_publication() {
    let polls = Cell::new(0);
    let admitted = admit(&pair(), &[edge("e")], Limits::default(), || {
        polls.set(polls.get() + 1);
        false
    })
    .unwrap();
    let total = polls.get();
    assert_eq!(total, 10);
    assert!(admitted.charged_work() > 0);
    for stop in 1..=total {
        let calls = Cell::new(0);
        let result = admit(&pair(), &[edge("e")], Limits::default(), || {
            calls.set(calls.get() + 1);
            calls.get() == stop
        });
        assert_eq!(result.unwrap_err(), Error::Cancelled);
        assert_eq!(calls.get(), stop);
    }
    assert_eq!(
        refusal(
            &pair(),
            &[],
            Limits {
                max_nodes: 0,
                ..Limits::default()
            }
        ),
        Error::Limit(Limit::Nodes)
    );
    assert_eq!(
        admit(
            &pair(),
            &[],
            Limits {
                max_nodes: 0,
                ..Limits::default()
            },
            || true
        )
        .unwrap_err(),
        Error::Cancelled
    );
}

#[test]
fn empty_success_is_polled_at_publication_without_charging_work() {
    let polls = Cell::new(0);
    let admitted = admit(
        &[],
        &[],
        Limits {
            max_work: 0,
            ..Limits::default()
        },
        || {
            polls.set(polls.get() + 1);
            false
        },
    )
    .unwrap();
    assert_eq!(
        (
            admitted.rectangles().len(),
            admitted.terminals().len(),
            admitted.charged_work()
        ),
        (0, 0, 0)
    );
    assert_eq!(polls.get(), 4);
    let calls = Cell::new(0);
    let result = admit(&[], &[], Limits::default(), || {
        calls.set(calls.get() + 1);
        calls.get() == 4
    });
    assert_eq!(result.unwrap_err(), Error::Cancelled);
}

#[test]
fn actual_landed_layout_result_is_consumed_through_public_types() {
    let nodes = ["a", "b"].map(|id| MeasuredNode {
        id,
        width: 4.0,
        height: 3.0,
        parent: None,
        port_count: 0,
        pinned: false,
    });
    let edges = [DirectedEdge {
        id: "e",
        source: "a",
        target: "b",
    }];
    let layout = syson_layout::layout(
        &nodes,
        &edges,
        Options::default(),
        syson_layout::Limits::default(),
        || false,
    )
    .unwrap();
    let admitted = admit(&layout.nodes, &[edge("e")], Limits::default(), || false).unwrap();
    let source = layout.nodes.iter().find(|p| p.id == "a").unwrap();
    let target = layout.nodes.iter().find(|p| p.id == "b").unwrap();
    assert_eq!(
        admitted.terminals()[0],
        (
            "e",
            Point {
                x2: ((source.x + source.width) * 2.0) as i64,
                y2: (source.y * 2.0 + source.height) as i64,
            },
            Point {
                x2: (target.x * 2.0) as i64,
                y2: (target.y * 2.0 + target.height) as i64,
            }
        )
    );
}
