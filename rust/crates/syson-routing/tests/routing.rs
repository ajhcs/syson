// SPDX-License-Identifier: EPL-2.0
use syson_layout::{DirectedEdge, MeasuredNode, Options, Placement, layout};
use syson_routing::{Edge, Error, Limit, Limits, Point, Routing, Side, Unsupported, route};

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
fn edge(id: &str) -> Edge<'_> {
    Edge {
        id,
        source: "a",
        target: "b",
        source_side: Side::Right,
        target_side: Side::Left,
    }
}
fn pair() -> Vec<Placement<'static>> {
    vec![node("a", 0., 0., 4., 4.), node("b", 20., 0., 4., 4.)]
}
fn run<'a>(nodes: &[Placement<'a>], edges: &[Edge<'a>]) -> Routing<'a> {
    route(nodes, edges, Limits::default(), || false).unwrap()
}
fn anchor(p: Placement<'_>, side: Side) -> Point {
    let (x, y) = match side {
        Side::Left => (p.x, p.y + p.height / 2.),
        Side::Right => (p.x + p.width, p.y + p.height / 2.),
        Side::Top => (p.x + p.width / 2., p.y),
        Side::Bottom => (p.x + p.width / 2., p.y + p.height),
    };
    Point {
        x2: (x * 2.) as i64,
        y2: (y * 2.) as i64,
    }
}
/// Independent geometric oracle: segment bounding interval must not have a
/// positive-length intersection with an open rectangle at its fixed coordinate.
fn verify(nodes: &[Placement<'_>], edges: &[Edge<'_>], result: &Routing<'_>) {
    assert_eq!(result.edges.len(), edges.len());
    for r in &result.edges {
        let e = edges.iter().find(|e| e.id == r.id).unwrap();
        let source = *nodes.iter().find(|n| n.id == e.source).unwrap();
        let target = *nodes.iter().find(|n| n.id == e.target).unwrap();
        assert_eq!(r.points.first(), Some(&anchor(source, e.source_side)));
        assert_eq!(r.points.last(), Some(&anchor(target, e.target_side)));
        for s in r.points.windows(2) {
            let (a, b) = (s[0], s[1]);
            assert_ne!(a, b, "zero segment");
            assert!(a.x2 == b.x2 || a.y2 == b.y2, "diagonal");
            for n in nodes {
                let (l, r, t, bottom) = (
                    n.x * 2.,
                    (n.x + n.width) * 2.,
                    n.y * 2.,
                    (n.y + n.height) * 2.,
                );
                let (lo, hi, fixed, near, far) = if a.x2 == b.x2 {
                    (
                        a.y2.min(b.y2) as f64,
                        a.y2.max(b.y2) as f64,
                        a.x2 as f64,
                        t,
                        bottom,
                    )
                } else {
                    (
                        a.x2.min(b.x2) as f64,
                        a.x2.max(b.x2) as f64,
                        a.y2 as f64,
                        l,
                        r,
                    )
                };
                let transverse_inside = if a.x2 == b.x2 {
                    l < fixed && fixed < r
                } else {
                    t < fixed && fixed < bottom
                };
                assert!(
                    !transverse_inside || hi <= near || lo >= far,
                    "interior crossing {r:?} / {n:?}"
                );
            }
        }
        for triple in r.points.windows(3) {
            assert!(!(triple[0].x2 == triple[1].x2 && triple[1].x2 == triple[2].x2));
            assert!(!(triple[0].y2 == triple[1].y2 && triple[1].y2 == triple[2].y2));
        }
    }
}
#[test]
fn straight_route_has_exact_odd_size_half_unit_anchors() {
    let nodes = [node("a", -5., -3., 3., 5.), node("b", 10., -3., 3., 5.)];
    let edges = [edge("one")];
    let result = run(&nodes, &edges);
    assert_eq!(
        result.edges[0].points,
        vec![Point { x2: -4, y2: -1 }, Point { x2: 20, y2: -1 }]
    );
    verify(&nodes, &edges, &result);
}
#[test]
fn obstacle_requires_real_detour() {
    let mut nodes = pair();
    nodes.push(node("obstacle", 9., -3., 5., 10.));
    let edges = [edge("one")];
    let result = run(&nodes, &edges);
    assert!(result.edges[0].points.len() >= 4);
    verify(&nodes, &edges, &result);
}
#[test]
fn self_edge_uses_distinct_side_centers_without_entering_node() {
    let nodes = [node("a", 0., 0., 5., 7.)];
    let edges = [Edge {
        id: "self",
        source: "a",
        target: "a",
        source_side: Side::Right,
        target_side: Side::Bottom,
    }];
    let result = run(&nodes, &edges);
    verify(&nodes, &edges, &result);
    assert!(result.edges[0].points.len() >= 3);
}
#[test]
fn same_anchor_is_unsupported_instead_of_fabricated_zero_length_loop() {
    let nodes = [node("a", 0., 0., 4., 4.)];
    let e = Edge {
        id: "self",
        source: "a",
        target: "a",
        source_side: Side::Right,
        target_side: Side::Right,
    };
    assert_eq!(
        route(&nodes, &[e], Limits::default(), || false),
        Err(Error::Unsupported(Unsupported::CoincidentAnchors))
    );
}
#[test]
fn parallel_ids_survive_and_shared_geometry_is_explicit() {
    let nodes = pair();
    let edges = [edge("z"), edge("a")];
    let result = run(&nodes, &edges);
    assert_eq!(
        result.edges.iter().map(|e| e.id).collect::<Vec<_>>(),
        vec!["a", "z"]
    );
    assert_eq!(result.edges[0].points, result.edges[1].points);
    verify(&nodes, &edges, &result);
}
#[test]
fn corner_and_boundary_touch_are_allowed_without_positive_clearance() {
    let nodes = [
        node("a", 0., 0., 4., 4.),
        node("b", 8., 0., 4., 4.),
        node("top", 4., -2., 4., 4.),
    ];
    let edges = [edge("e")];
    let result = run(&nodes, &edges);
    assert_eq!(
        result.edges[0].points,
        vec![Point { x2: 8, y2: 4 }, Point { x2: 16, y2: 4 }]
    );
    verify(&nodes, &edges, &result);
}
#[test]
fn permutations_do_not_change_routes_or_charged_work() {
    let mut nodes = pair();
    nodes.push(node("o", 9., -3., 5., 10.));
    let mut edges = [edge("z"), edge("a")];
    let first = run(&nodes, &edges);
    nodes.reverse();
    edges.reverse();
    assert_eq!(run(&nodes, &edges), first);
}
#[test]
fn geometry_rejects_fractional_nonfinite_degenerate_and_outside_edges() {
    for bad in [
        node("a", 0.5, 0., 4., 4.),
        node("a", 0., f64::NAN, 4., 4.),
        node("a", 0., 0., f64::INFINITY, 4.),
        node("a", 0., 0., 0., 4.),
        node("a", 0., 0., 4., -1.),
        node("a", 999_999., 0., 2., 4.),
    ] {
        assert_eq!(
            route(&[bad], &[], Limits::default(), || false),
            Err(Error::Unsupported(Unsupported::Geometry))
        );
    }
    let valid = node("wide", -1_000_000., -1_000_000., 2_000_000., 2_000_000.);
    assert!(route(&[valid], &[], Limits::default(), || false).is_ok());
}
#[test]
fn interior_overlap_is_distinct_from_touching() {
    let nodes = [node("a", 0., 0., 4., 4.), node("b", 3., 3., 4., 4.)];
    assert_eq!(
        route(&nodes, &[edge("e")], Limits::default(), || false),
        Err(Error::Unsupported(Unsupported::OverlappingInteriors))
    );
}
#[test]
fn identity_and_endpoint_failures_are_typed() {
    let nodes = pair();
    assert_eq!(
        route(&[nodes[0], nodes[0]], &[], Limits::default(), || false),
        Err(Error::DuplicateNode)
    );
    assert_eq!(
        route(&nodes, &[edge("x"), edge("x")], Limits::default(), || false),
        Err(Error::DuplicateEdge)
    );
    assert_eq!(
        route(
            &nodes,
            &[Edge {
                target: "missing",
                ..edge("x")
            }],
            Limits::default(),
            || false
        ),
        Err(Error::UnknownEndpoint)
    );
    assert_eq!(
        route(&[node("", 0., 0., 1., 1.)], &[], Limits::default(), || {
            false
        }),
        Err(Error::EmptyId)
    );
    assert_eq!(
        route(&nodes, &[edge("")], Limits::default(), || false),
        Err(Error::EmptyId)
    );
}
#[test]
fn empty_graph_and_no_edge_results_have_no_fabricated_routes() {
    assert_eq!(run(&[], &[]).edges, vec![]);
    let result = run(&pair(), &[]);
    assert!(result.edges.is_empty());
    assert_eq!(result.grid_vertices, 0);
}
#[test]
fn all_resource_limit_classes_are_enforced_without_partial_results() {
    let nodes = pair();
    let edges = [edge("e")];
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
                max_grid_vertices: 1,
                ..Limits::default()
            },
            Limit::Grid,
        ),
        (
            Limits {
                max_output_points: 1,
                ..Limits::default()
            },
            Limit::Output,
        ),
        (
            Limits {
                max_work: 1,
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
            route(&nodes, &edges, limits, || false),
            Err(Error::Limit(kind))
        );
    }
}
#[test]
fn exact_grid_and_work_budgets_accept_but_one_less_rejects() {
    let nodes = pair();
    let edges = [edge("e")];
    let result = run(&nodes, &edges);
    let limits = Limits {
        max_grid_vertices: result.grid_vertices,
        max_work: result.charged_work,
        max_output_points: 2,
        ..Limits::default()
    };
    assert_eq!(route(&nodes, &edges, limits, || false).unwrap(), result);
    assert_eq!(
        route(
            &nodes,
            &edges,
            Limits {
                max_work: result.charged_work - 1,
                ..limits
            },
            || false
        ),
        Err(Error::Limit(Limit::Work))
    );
    assert_eq!(
        route(
            &nodes,
            &edges,
            Limits {
                max_grid_vertices: result.grid_vertices - 1,
                ..limits
            },
            || false
        ),
        Err(Error::Limit(Limit::Grid))
    );
}
#[test]
fn cancellation_is_polled_initially_and_during_search() {
    let nodes = pair();
    let edges = [edge("e")];
    assert_eq!(
        route(&nodes, &edges, Limits::default(), || true),
        Err(Error::Cancelled)
    );
    let mut polls = 0;
    let complete = route(&nodes, &edges, Limits::default(), || {
        polls += 1;
        false
    })
    .unwrap();
    assert!(polls > 20);
    let mut late = 0;
    assert_eq!(
        route(&nodes, &edges, Limits::default(), || {
            late += 1;
            late == polls - 1
        }),
        Err(Error::Cancelled)
    );
    verify(&nodes, &edges, &complete);
}
#[test]
fn real_f16_layout_positions_feed_router_without_mutation() {
    let nodes = [
        MeasuredNode {
            id: "a",
            width: 5.,
            height: 7.,
            parent: None,
            port_count: 0,
            pinned: false,
        },
        MeasuredNode {
            id: "b",
            width: 9.,
            height: 3.,
            parent: None,
            port_count: 0,
            pinned: false,
        },
    ];
    let placed = layout(
        &nodes,
        &[DirectedEdge {
            id: "e",
            source: "a",
            target: "b",
        }],
        Options::default(),
        syson_layout::Limits::default(),
        || false,
    )
    .unwrap();
    let original = placed.clone();
    let edges = [edge("e")];
    let result = run(&placed.nodes, &edges);
    verify(&placed.nodes, &edges, &result);
    assert_eq!(placed, original);
}
#[test]
fn generated_obstacles_and_all_distinct_self_sides_preserve_invariants() {
    let mut scenarios = 0;
    for y in [-8., -4., 0., 4., 8.] {
        for width in [2., 5., 9.] {
            let nodes = [
                node("a", 0., 0., 4., 4.),
                node("b", 30., 0., 4., 4.),
                node("o", 10., y, width, 4.),
            ];
            let edges = [edge("e")];
            verify(&nodes, &edges, &run(&nodes, &edges));
            scenarios += 1;
        }
    }
    for source_side in [Side::Left, Side::Right, Side::Top, Side::Bottom] {
        for target_side in [Side::Left, Side::Right, Side::Top, Side::Bottom] {
            if source_side == target_side {
                continue;
            }
            let nodes = [node("a", -5., -8., 3., 7.)];
            let edges = [Edge {
                id: "self",
                source: "a",
                target: "a",
                source_side,
                target_side,
            }];
            verify(&nodes, &edges, &run(&nodes, &edges));
            scenarios += 1;
        }
    }
    assert_eq!(scenarios, 27);
}
