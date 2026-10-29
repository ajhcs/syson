// SPDX-License-Identifier: EPL-2.0
use std::cell::Cell;
use syson_layout::{
    AdmittedGraph, DirectedEdge, Direction, LayoutError, Limit, Limits, MeasuredNode, Options,
    Unsupported, admit,
};

fn node(id: &str, width: f64, height: f64) -> MeasuredNode<'_> {
    MeasuredNode {
        id,
        width,
        height,
        parent: None,
        port_count: 0,
        pinned: false,
    }
}
fn edge<'a>(id: &'a str, source: &'a str, target: &'a str) -> DirectedEdge<'a> {
    DirectedEdge { id, source, target }
}
fn no_cancel() -> bool {
    false
}
fn run<'g, 'a>(
    nodes: &'g [MeasuredNode<'a>],
    edges: &'g [DirectedEdge<'a>],
    limits: Limits,
) -> Result<AdmittedGraph<'g, 'a, fn() -> bool>, LayoutError> {
    admit(
        nodes,
        edges,
        Options::default(),
        limits,
        no_cancel as fn() -> bool,
    )
}

#[test]
fn empty_admission_has_no_logical_work_or_storage() {
    let limits = Limits {
        max_nodes: 0,
        max_edges: 0,
        max_work: 0,
        max_workspace_bytes: 0,
        max_id_bytes: 0,
    };
    let admitted = run(&[], &[], limits).unwrap();
    assert!(admitted.nodes().is_empty());
    assert!(admitted.edges().is_empty());
    assert!(admitted.node_order().is_empty());
    assert!(admitted.forward().is_empty());
    assert!(admitted.reverse().is_empty());
    assert!(admitted.indexed_edges().is_empty());
    assert_eq!(admitted.charged_work(), 0);
    assert_eq!(admitted.estimated_workspace_bytes(), 0);
    let (_, mut guard) = admitted.into_parts();
    assert_eq!(guard.tick(1), Err(LayoutError::Limit(Limit::Work)));
}

#[test]
fn inputs_are_borrowed_with_fractional_dimensions_and_original_options() {
    let nodes = [node("雪🧭", 12.5, -0.0)];
    let edges = [];
    let options = Options {
        direction: Direction::Down,
        layer_gap: 0.5,
        node_gap: 1.25,
        component_gap: 2.75,
        padding: 0.0,
    };
    let admitted = admit(&nodes, &edges, options, Limits::default(), no_cancel).unwrap();
    assert!(std::ptr::eq(admitted.nodes(), nodes.as_slice()));
    assert!(std::ptr::eq(admitted.edges(), edges.as_slice()));
    assert_eq!(admitted.nodes()[0].width, 12.5);
    assert_eq!(admitted.nodes()[0].height.to_bits(), (-0.0f64).to_bits());
    assert_eq!(admitted.options().direction, Direction::Down);
    assert_eq!(admitted.options().node_gap, 1.25);
    assert_eq!(admitted.options().component_gap, 2.75);
    assert_eq!(admitted.options().layer_gap, 0.5);
    assert_eq!(admitted.options().padding, 0.0);
    assert_eq!(admitted.limits().max_work, Limits::default().max_work);
    assert_eq!(admitted.maximum_id_bytes(), "雪🧭".len());
}

#[test]
fn canonical_adjacency_retains_loops_parallel_edges_and_separate_id_domains() {
    let nodes = [
        node("c", 3.0, 4.0),
        node("a", 1.0, 2.0),
        node("b", 2.0, 3.0),
    ];
    let edges = [
        edge("z", "a", "b"),
        edge("a", "b", "c"),
        edge("m", "a", "a"),
        edge("q", "a", "b"),
    ];
    let original = (nodes, edges);
    let admitted = run(&nodes, &edges, Limits::default()).unwrap();
    assert_eq!(admitted.node_order(), &[1, 2, 0]);
    assert_eq!(
        admitted.indexed_edges(),
        &[(1, 2, 1), (0, 0, 2), (0, 1, 3), (0, 1, 0)]
    );
    assert_eq!(admitted.forward(), &[vec![0, 1, 1], vec![2], vec![]]);
    assert_eq!(admitted.reverse(), &[vec![0], vec![0, 0], vec![1]]);
    assert_eq!(admitted.estimated_workspace_bytes(), 3 * 1024 + 4 * 640);
    assert_eq!((nodes, edges), original);
}

#[test]
fn permutations_keep_canonical_membership_and_endpoint_identity() {
    let nodes = [
        node("c", 1.0, 2.0),
        node("a", 3.0, 4.0),
        node("b", 5.0, 6.0),
    ];
    let edges = [edge("z", "a", "b"), edge("a", "b", "c")];
    for node_order in [[0, 1, 2], [2, 1, 0], [1, 2, 0]] {
        for edge_order in [[0, 1], [1, 0]] {
            let n = node_order.map(|i| nodes[i]);
            let e = edge_order.map(|i| edges[i]);
            let graph = run(&n, &e, Limits::default()).unwrap();
            let ids: Vec<_> = graph.node_order().iter().map(|&i| n[i].id).collect();
            let triples: Vec<_> = graph
                .indexed_edges()
                .iter()
                .map(|&(s, t, i)| {
                    (
                        e[i].id,
                        n[graph.node_order()[s]].id,
                        n[graph.node_order()[t]].id,
                    )
                })
                .collect();
            assert_eq!(ids, ["a", "b", "c"]);
            assert_eq!(triples, [("a", "b", "c"), ("z", "a", "b")]);
        }
    }
}

#[test]
fn empty_duplicate_and_unknown_ids_fail_without_rewriting_inputs() {
    let nodes = [node("a", 1.0, 2.0), node("b", 3.0, 4.0)];
    for (n, e, expected) in [
        (vec![nodes[0], nodes[0]], vec![], LayoutError::DuplicateNode),
        (vec![node("", 1.0, 1.0)], vec![], LayoutError::EmptyId),
        (
            nodes.to_vec(),
            vec![edge("e", "a", "b"), edge("e", "b", "a")],
            LayoutError::DuplicateEdge,
        ),
        (
            nodes.to_vec(),
            vec![edge("", "a", "b")],
            LayoutError::EmptyId,
        ),
        (
            nodes.to_vec(),
            vec![edge("e", "", "b")],
            LayoutError::EmptyId,
        ),
        (
            nodes.to_vec(),
            vec![edge("e", "a", "")],
            LayoutError::EmptyId,
        ),
        (
            nodes.to_vec(),
            vec![edge("e", "missing", "b")],
            LayoutError::UnknownEndpoint,
        ),
        (
            nodes.to_vec(),
            vec![edge("e", "a", "missing")],
            LayoutError::UnknownEndpoint,
        ),
    ] {
        let original = (n.clone(), e.clone());
        assert_eq!(run(&n, &e, Limits::default()).unwrap_err(), expected);
        assert_eq!((n, e), original);
    }
}

#[test]
fn nonfinite_negative_dimensions_and_all_invalid_spacing_fields_are_refused() {
    for bad in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for n in [node("a", bad, 1.0), node("a", 1.0, bad)] {
            assert_eq!(
                run(&[n], &[], Limits::default()).unwrap_err(),
                LayoutError::InvalidNumber
            );
        }
        for field in 0..4 {
            let mut options = Options::default();
            match field {
                0 => options.layer_gap = bad,
                1 => options.node_gap = bad,
                2 => options.component_gap = bad,
                _ => options.padding = bad,
            }
            assert_eq!(
                admit(&[], &[], options, Limits::default(), no_cancel).unwrap_err(),
                LayoutError::InvalidNumber
            );
        }
    }
}

#[test]
fn containers_ports_and_pins_are_typed_unsupported_inputs() {
    for (parent, port_count, pinned, expected) in [
        (Some("unloaded-parent"), 0, false, Unsupported::Container),
        (None, 1, false, Unsupported::Ports),
        (None, 0, true, Unsupported::Pinned),
    ] {
        let n = MeasuredNode {
            parent,
            port_count,
            pinned,
            ..node("a", 1.0, 2.0)
        };
        assert_eq!(
            run(&[n], &[], Limits::default()).unwrap_err(),
            LayoutError::Unsupported(expected)
        );
    }
}

#[test]
fn each_guard_has_an_exact_boundary_and_an_independent_one_over_case() {
    let nodes = [node("node/雪", 10.0, 11.0), node("node/🚗", 12.0, 13.0)];
    let edges = [edge("edge/reference-🧭", "node/雪", "node/🚗")];
    let graph = run(&nodes, &edges, Limits::default()).unwrap();
    let exact = Limits {
        max_nodes: nodes.len(),
        max_edges: edges.len(),
        max_id_bytes: edges[0].id.len(),
        max_work: graph.charged_work(),
        max_workspace_bytes: graph.estimated_workspace_bytes(),
    };
    assert_eq!(graph.maximum_id_bytes(), exact.max_id_bytes);
    let repeated = run(&nodes, &edges, exact).unwrap();
    assert_eq!(repeated.charged_work(), exact.max_work);
    for (limits, expected) in [
        (
            Limits {
                max_nodes: exact.max_nodes - 1,
                ..exact
            },
            Limit::Nodes,
        ),
        (
            Limits {
                max_edges: exact.max_edges - 1,
                ..exact
            },
            Limit::Edges,
        ),
        (
            Limits {
                max_workspace_bytes: exact.max_workspace_bytes - 1,
                ..exact
            },
            Limit::Memory,
        ),
        (
            Limits {
                max_id_bytes: exact.max_id_bytes - 1,
                ..exact
            },
            Limit::IdBytes,
        ),
        (
            Limits {
                max_work: exact.max_work - 1,
                ..exact
            },
            Limit::Work,
        ),
    ] {
        assert_eq!(
            run(&nodes, &edges, limits).unwrap_err(),
            LayoutError::Limit(expected)
        );
    }
    let unicode = [node("🧭", 1.0, 1.0)];
    assert!(
        run(
            &unicode,
            &[],
            Limits {
                max_id_bytes: 4,
                ..Limits::default()
            }
        )
        .is_ok()
    );
    assert_eq!(
        run(
            &unicode,
            &[],
            Limits {
                max_id_bytes: 3,
                ..Limits::default()
            }
        )
        .unwrap_err(),
        LayoutError::Limit(Limit::IdBytes)
    );
}

#[test]
fn cancellation_at_every_observed_poll_refuses_without_partial_publication() {
    let nodes = [node("b", 1.0, 2.0), node("a", 3.0, 4.0)];
    let edges = [edge("ab", "a", "b"), edge("aa", "a", "a")];
    let original = (nodes, edges);
    let polls = Cell::new(0);
    let successful = admit(
        &nodes,
        &edges,
        Options::default(),
        Limits::default(),
        || {
            polls.set(polls.get() + 1);
            false
        },
    )
    .unwrap();
    let total = polls.get();
    drop(successful);
    for stop in 1..=total {
        let mut now = 0;
        let result = admit(
            &nodes,
            &edges,
            Options::default(),
            Limits::default(),
            || {
                now += 1;
                now >= stop
            },
        );
        assert_eq!(result.unwrap_err(), LayoutError::Cancelled);
        assert_eq!(now, stop);
        assert_eq!((nodes, edges), original);
    }
    let invalid = Limits {
        max_nodes: 0,
        ..Limits::default()
    };
    assert_eq!(
        admit(&nodes, &edges, Options::default(), invalid, || true).unwrap_err(),
        LayoutError::Cancelled
    );
}

#[test]
fn continuation_retains_spent_work_and_checked_arithmetic() {
    let nodes = [node("a", 1.0, 2.0)];
    let baseline = run(&nodes, &[], Limits::default()).unwrap().charged_work();
    let admitted = run(
        &nodes,
        &[],
        Limits {
            max_work: baseline + 2,
            ..Limits::default()
        },
    )
    .unwrap();
    let (graph, mut guard) = admitted.into_parts();
    assert!(std::ptr::eq(graph.nodes(), nodes.as_slice()));
    assert_eq!(guard.used(), baseline);
    guard.tick(2).unwrap();
    assert_eq!(guard.used(), baseline + 2);
    assert_eq!(guard.tick(1), Err(LayoutError::Limit(Limit::Work)));
    assert_eq!(guard.used(), baseline + 2);
    let admitted = run(
        &nodes,
        &[],
        Limits {
            max_work: usize::MAX,
            ..Limits::default()
        },
    )
    .unwrap();
    let (_, mut overflow) = admitted.into_parts();
    assert_eq!(
        overflow.tick(usize::MAX),
        Err(LayoutError::Limit(Limit::Work))
    );
    assert_eq!(overflow.used(), baseline);
}

#[test]
fn continuation_uses_the_same_live_cancellation_callback() {
    let nodes = [node("a", 1.0, 2.0)];
    let flag = Cell::new(false);
    let polls = Cell::new(0);
    let admitted = admit(&nodes, &[], Options::default(), Limits::default(), || {
        polls.set(polls.get() + 1);
        flag.get()
    })
    .unwrap();
    let spent = admitted.charged_work();
    let old_polls = polls.get();
    flag.set(true);
    let (_, mut guard) = admitted.into_parts();
    assert_eq!(guard.tick(0), Err(LayoutError::Cancelled));
    assert_eq!(guard.used(), spent);
    assert_eq!(polls.get(), old_polls + 1);
}
