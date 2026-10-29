// SPDX-License-Identifier: EPL-2.0
//! Proposed-algorithm invariants, not upstream ELK runtime goldens.
use syson_layout::*;
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
fn run<'a>(
    nodes: &[MeasuredNode<'a>],
    edges: &[DirectedEdge<'a>],
    direction: Direction,
) -> Layout<'a> {
    layout(
        nodes,
        edges,
        Options {
            direction,
            ..Options::default()
        },
        Limits::default(),
        || false,
    )
    .unwrap()
}
fn placed<'a, 'b>(result: &'b Layout<'a>, id: &str) -> &'b Placement<'a> {
    result.nodes.iter().find(|p| p.id == id).unwrap()
}
fn invariants(result: &Layout<'_>, direction: Direction) {
    assert!(result.width.is_finite() && result.height.is_finite());
    for (i, a) in result.nodes.iter().enumerate() {
        assert!(a.x.is_finite() && a.y.is_finite() && a.x >= 0.0 && a.y >= 0.0);
        assert!(a.x + a.width <= result.width && a.y + a.height <= result.height);
        for b in result.nodes.iter().skip(i + 1) {
            assert!(
                a.x + a.width <= b.x
                    || b.x + b.width <= a.x
                    || a.y + a.height <= b.y
                    || b.y + b.height <= a.y,
                "overlap {} {}",
                a.id,
                b.id
            );
        }
    }
    for e in &result.edges {
        let a = placed(result, e.edge.source);
        let b = placed(result, e.edge.target);
        assert_eq!(
            e.internal_strong_component,
            a.strong_component == b.strong_component
        );
        if !e.internal_strong_component {
            assert!(a.rank < b.rank);
            match direction {
                Direction::Right => assert!(a.x + a.width <= b.x),
                Direction::Down => assert!(a.y + a.height <= b.y),
            }
        }
    }
}
#[test]
fn an_empty_graph_needs_no_work_or_logical_buffer_storage() {
    let limits = Limits {
        max_nodes: 0,
        max_edges: 0,
        max_work: 0,
        max_workspace_bytes: 0,
        max_id_bytes: 0,
    };
    let result = layout(&[], &[], Options::default(), limits, || false).unwrap();
    assert!(result.nodes.is_empty() && result.edges.is_empty());
    assert_eq!(
        (
            result.width,
            result.height,
            result.charged_work,
            result.estimated_workspace_bytes
        ),
        (0.0, 0.0, 0, 0)
    );
}
#[test]
fn chain_ranks_and_positions_depend_on_edge_direction() {
    let nodes = [
        node("a", 30.0, 20.0),
        node("b", 60.0, 50.0),
        node("c", 15.0, 40.0),
    ];
    let chain = [edge("ab", "a", "b"), edge("bc", "b", "c")];
    for direction in [Direction::Right, Direction::Down] {
        let result = run(&nodes, &chain, direction);
        invariants(&result, direction);
        assert_eq!(
            result.nodes.iter().map(|p| p.rank).collect::<Vec<_>>(),
            [0, 1, 2]
        );
        let reversed = run(
            &nodes,
            &[edge("ba", "b", "a"), edge("cb", "c", "b")],
            direction,
        );
        assert_eq!(
            reversed.nodes.iter().map(|p| p.rank).collect::<Vec<_>>(),
            [2, 1, 0]
        );
        assert_ne!(result.nodes, reversed.nodes);
        invariants(&reversed, direction);
    }
}
#[test]
fn a_diamond_uses_measured_layer_extents_and_longest_path_rank() {
    let nodes = [
        node("a", 10.0, 20.0),
        node("b", 100.0, 70.0),
        node("c", 20.0, 30.0),
        node("d", 15.0, 10.0),
    ];
    let edges = [
        edge("ab", "a", "b"),
        edge("ac", "a", "c"),
        edge("bd", "b", "d"),
        edge("cd", "c", "d"),
        edge("ad", "a", "d"),
    ];
    let result = run(&nodes, &edges, Direction::Right);
    invariants(&result, Direction::Right);
    assert_eq!(
        result.nodes.iter().map(|p| p.rank).collect::<Vec<_>>(),
        [0, 1, 1, 2]
    );
    assert_eq!(
        (
            placed(&result, "a").x,
            placed(&result, "b").x,
            placed(&result, "c").x,
            placed(&result, "d").x
        ),
        (16.0, 66.0, 66.0, 206.0)
    );
    assert_eq!(placed(&result, "c").y, 106.0);
}
#[test]
fn cyclic_components_self_loops_and_parallel_edges_are_retained() {
    let nodes = [
        node("a", 30.0, 20.0),
        node("b", 100.0, 40.0),
        node("c", 40.0, 80.0),
        node("d", 20.0, 25.0),
        node("e", 80.0, 20.0),
        node("f", 50.0, 50.0),
    ];
    let edges = [
        edge("ab", "a", "b"),
        edge("ba", "b", "a"),
        edge("bc", "b", "c"),
        edge("bc2", "b", "c"),
        edge("cd", "c", "d"),
        edge("dc", "d", "c"),
        edge("de", "d", "e"),
        edge("ff", "f", "f"),
    ];
    let result = run(&nodes, &edges, Direction::Right);
    invariants(&result, Direction::Right);
    assert_eq!(result.edges.len(), edges.len());
    assert_eq!(
        placed(&result, "a").strong_component,
        placed(&result, "b").strong_component
    );
    assert_eq!(
        placed(&result, "c").strong_component,
        placed(&result, "d").strong_component
    );
    assert_eq!(
        result.nodes.iter().map(|p| p.rank).collect::<Vec<_>>(),
        [0, 0, 1, 1, 2, 0]
    );
    assert!(
        result
            .edges
            .iter()
            .find(|e| e.edge.id == "ff")
            .unwrap()
            .internal_strong_component
    );
    assert_ne!(
        placed(&result, "a").weak_component,
        placed(&result, "f").weak_component
    );
}
#[test]
fn disconnected_weak_components_have_explicit_cross_axis_gutters() {
    let nodes = [
        node("a", 40.0, 20.0),
        node("b", 30.0, 100.0),
        node("c", 300.0, 50.0),
        node("d", 70.0, 30.0),
    ];
    let edges = [edge("ab", "a", "b"), edge("cd", "c", "d")];
    for direction in [Direction::Right, Direction::Down] {
        let result = run(&nodes, &edges, direction);
        invariants(&result, direction);
        let first: Vec<_> = result
            .nodes
            .iter()
            .filter(|p| p.weak_component == 0)
            .collect();
        let second: Vec<_> = result
            .nodes
            .iter()
            .filter(|p| p.weak_component == 1)
            .collect();
        let end = first
            .iter()
            .map(|p| {
                if direction == Direction::Right {
                    p.y + p.height
                } else {
                    p.x + p.width
                }
            })
            .fold(0.0, f64::max);
        let start = second
            .iter()
            .map(|p| {
                if direction == Direction::Right {
                    p.y
                } else {
                    p.x
                }
            })
            .fold(f64::INFINITY, f64::min);
        assert_eq!(start - end, Options::default().component_gap);
    }
}
#[test]
fn node_and_edge_permutations_produce_identical_canonical_results() {
    let nodes = [
        node("z", 10.0, 25.0),
        node("a", 90.0, 20.0),
        node("m", 40.0, 60.0),
        node("q", 10.0, 10.0),
    ];
    let edges = [
        edge("z-m", "z", "m"),
        edge("a-z", "a", "z"),
        edge("m-a", "m", "a"),
    ];
    let original = run(&nodes, &edges, Direction::Right);
    let permuted = run(
        &[nodes[2], nodes[0], nodes[3], nodes[1]],
        &[edges[2], edges[0], edges[1]],
        Direction::Right,
    );
    assert_eq!(original, permuted);
    assert_eq!(
        original.nodes.iter().map(|p| p.id).collect::<Vec<_>>(),
        ["a", "m", "q", "z"]
    );
}
#[test]
fn duplicate_empty_and_unknown_identifiers_are_errors() {
    let n = [node("a", 10.0, 10.0), node("b", 10.0, 10.0)];
    for (nodes, edges, expected) in [
        (vec![n[0], n[0]], vec![], LayoutError::DuplicateNode),
        (vec![node("", 1.0, 1.0)], vec![], LayoutError::EmptyId),
        (
            n.to_vec(),
            vec![edge("e", "a", "b"), edge("e", "b", "a")],
            LayoutError::DuplicateEdge,
        ),
        (n.to_vec(), vec![edge("", "a", "b")], LayoutError::EmptyId),
        (
            n.to_vec(),
            vec![edge("e", "missing", "b")],
            LayoutError::UnknownEndpoint,
        ),
        (
            n.to_vec(),
            vec![edge("e", "a", "missing")],
            LayoutError::UnknownEndpoint,
        ),
    ] {
        assert_eq!(
            layout(
                &nodes,
                &edges,
                Options::default(),
                Limits::default(),
                || false
            ),
            Err(expected)
        );
    }
}
#[test]
fn unsupported_containers_ports_and_pinning_are_not_silently_dropped() {
    let mut n = node("a", 10.0, 10.0);
    n.parent = Some("parent");
    assert_eq!(
        layout(&[n], &[], Options::default(), Limits::default(), || false),
        Err(LayoutError::Unsupported(Unsupported::Container))
    );
    n.parent = None;
    n.port_count = 1;
    assert_eq!(
        layout(&[n], &[], Options::default(), Limits::default(), || false),
        Err(LayoutError::Unsupported(Unsupported::Ports))
    );
    n.port_count = 0;
    n.pinned = true;
    assert_eq!(
        layout(&[n], &[], Options::default(), Limits::default(), || false),
        Err(LayoutError::Unsupported(Unsupported::Pinned))
    );
}
#[test]
fn invalid_dimensions_and_spacing_are_rejected() {
    for bad in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for n in [node("a", bad, 1.0), node("a", 1.0, bad)] {
            assert_eq!(
                layout(&[n], &[], Options::default(), Limits::default(), || false),
                Err(LayoutError::InvalidNumber)
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
                layout(&[], &[], options, Limits::default(), || false),
                Err(LayoutError::InvalidNumber)
            );
        }
    }
}
#[test]
fn each_budget_fails_explicitly_and_exact_admitted_work_is_reproducible() {
    let nodes = [node("a", 10.0, 10.0), node("b", 20.0, 15.0)];
    let edges = [edge("ab", "a", "b")];
    let baseline = run(&nodes, &edges, Direction::Right);
    assert_eq!(
        baseline.estimated_workspace_bytes,
        1024 * nodes.len() + 640 * edges.len()
    );
    for (limits, expected) in [
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
                max_workspace_bytes: baseline.estimated_workspace_bytes - 1,
                ..Limits::default()
            },
            Limit::Memory,
        ),
        (
            Limits {
                max_id_bytes: 1,
                ..Limits::default()
            },
            Limit::IdBytes,
        ),
        (
            Limits {
                max_work: baseline.charged_work - 1,
                ..Limits::default()
            },
            Limit::Work,
        ),
    ] {
        assert_eq!(
            layout(&nodes, &edges, Options::default(), limits, || false),
            Err(LayoutError::Limit(expected))
        );
    }
    let exact = Limits {
        max_work: baseline.charged_work,
        max_workspace_bytes: baseline.estimated_workspace_bytes,
        ..Limits::default()
    };
    assert_eq!(
        layout(&nodes, &edges, Options::default(), exact, || false).unwrap(),
        baseline
    );
}
#[test]
fn cancellation_before_during_and_late_in_layout_returns_no_partial_result() {
    let nodes = [
        node("a", 10.0, 10.0),
        node("b", 20.0, 15.0),
        node("c", 40.0, 20.0),
    ];
    let edges = [edge("ab", "a", "b"), edge("bc", "b", "c")];
    let originals = (nodes, edges);
    let mut polls = 0;
    layout(
        &nodes,
        &edges,
        Options::default(),
        Limits::default(),
        || {
            polls += 1;
            false
        },
    )
    .unwrap();
    for stop in [1, polls / 2, polls - 1] {
        let mut now = 0;
        assert_eq!(
            layout(
                &nodes,
                &edges,
                Options::default(),
                Limits::default(),
                || {
                    now += 1;
                    now >= stop
                }
            ),
            Err(LayoutError::Cancelled)
        );
        assert_eq!((nodes, edges), originals);
    }
}
#[test]
fn numeric_overflow_and_lost_positive_coordinate_advance_are_explicit() {
    let huge = [node("a", f64::MAX, f64::MAX)];
    let options = Options {
        padding: f64::MAX,
        ..Options::default()
    };
    assert_eq!(
        layout(&huge, &[], options, Limits::default(), || false),
        Err(LayoutError::ArithmeticOverflow)
    );
    let tiny = [node("a", 1e-300, 10.0)];
    assert_eq!(
        layout(&tiny, &[], Options::default(), Limits::default(), || false),
        Err(LayoutError::PrecisionLoss)
    );
}
#[test]
fn zero_size_rectangles_and_zero_gaps_are_supported_without_overlap() {
    let nodes = [node("a", 0.0, 0.0), node("b", 10.0, 15.0)];
    let options = Options {
        layer_gap: 0.0,
        node_gap: 0.0,
        component_gap: 0.0,
        padding: 0.0,
        direction: Direction::Right,
    };
    let result = layout(
        &nodes,
        &[edge("ab", "a", "b")],
        options,
        Limits::default(),
        || false,
    )
    .unwrap();
    invariants(&result, Direction::Right);
    assert_eq!(placed(&result, "b").rank, 1);
}
#[test]
fn all_three_node_directed_graphs_match_independent_reachability_and_invariants() {
    let nodes = [
        node("a", 30.0, 20.0),
        node("b", 50.0, 60.0),
        node("c", 10.0, 25.0),
    ];
    let ids = ["a", "b", "c"];
    let edge_ids = ["aa", "ab", "ac", "ba", "bb", "bc", "ca", "cb", "cc"];
    for mask in 0..512 {
        let mut edges = Vec::new();
        let mut reach = [[false; 3]; 3];
        for (i, row) in reach.iter_mut().enumerate() {
            row[i] = true;
        }
        for (bit, id) in edge_ids.iter().enumerate() {
            if mask & (1 << bit) != 0 {
                let (a, b) = (bit / 3, bit % 3);
                edges.push(edge(id, ids[a], ids[b]));
                reach[a][b] = true;
            }
        }
        for through in 0..3 {
            for a in 0..3 {
                for b in 0..3 {
                    let indirect = reach[a][through] && reach[through][b];
                    reach[a][b] |= indirect;
                }
            }
        }
        for direction in [Direction::Right, Direction::Down] {
            let result = run(&nodes, &edges, direction);
            invariants(&result, direction);
            assert_eq!(result.edges.len(), edges.len());
            for (a, from_a) in reach.iter().enumerate() {
                for (b, from_b) in reach.iter().enumerate() {
                    assert_eq!(
                        result.nodes[a].strong_component == result.nodes[b].strong_component,
                        from_a[b] && from_b[a],
                        "mask {mask}"
                    );
                }
            }
            for (a, from_a) in reach.iter().enumerate() {
                let expected_rank = edges
                    .iter()
                    .filter_map(|e| {
                        let source = ids.iter().position(|id| *id == e.source).unwrap();
                        let target = ids.iter().position(|id| *id == e.target).unwrap();
                        (from_a[target]
                            && reach[target][a]
                            && !(from_a[source] && reach[source][a]))
                            .then_some(result.nodes[source].rank + 1)
                    })
                    .max()
                    .unwrap_or(0);
                assert_eq!(result.nodes[a].rank, expected_rank, "mask {mask}, node {a}");
            }
        }
    }
}
#[test]
fn unused_layer_node_and_trailing_component_gaps_do_not_overflow() {
    let nodes = [
        node("a", 20.0, 10.0),
        node("b", 30.0, 15.0),
        node("c", 40.0, 25.0),
    ];
    for direction in [Direction::Right, Direction::Down] {
        let options = Options {
            direction,
            layer_gap: f64::MAX,
            node_gap: f64::MAX,
            component_gap: f64::MAX,
            padding: 0.0,
        };
        let result = layout(&nodes[..1], &[], options, Limits::default(), || false).unwrap();
        assert_eq!((result.width, result.height), (20.0, 10.0));
        let options = Options {
            component_gap: 60.0,
            ..options
        };
        let result = layout(&nodes, &[], options, Limits::default(), || false).unwrap();
        invariants(&result, direction);
    }
}
