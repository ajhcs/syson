// SPDX-License-Identifier: EPL-2.0
use std::cell::Cell;
use syson_layout::containers::{
    AdmittedForest, Error, PackageNode, Position, Size, admit_packages,
};
use syson_layout::{DirectedEdge, Direction, LayoutError, Limit, Limits, Options};

fn size(width: f64, height: f64) -> Size {
    Size { width, height }
}
fn node<'a>(id: &'a str, parent: Option<&'a str>) -> PackageNode<'a> {
    PackageNode {
        id,
        parent,
        leaf_size: size(97.125, 21.5),
        header_height_footprint: 12.25,
        border_width: 1.5,
        default_width: Some(0.),
        default_height: Some(0.),
        user_minimum: None,
        retained_position: None,
    }
}
fn edge<'a>(id: &'a str, source: &'a str, target: &'a str) -> DirectedEdge<'a> {
    DirectedEdge { id, source, target }
}
fn refusal(nodes: &[PackageNode<'_>], edges: &[DirectedEdge<'_>], limits: Limits) -> Error {
    admit_packages(nodes, edges, Options::default(), limits, || false).unwrap_err()
}

fn accepted<'a>(
    nodes: &[PackageNode<'a>],
    edges: &[DirectedEdge<'a>],
) -> AdmittedForest<'a, fn() -> bool> {
    let cancelled: fn() -> bool = || false;
    admit_packages(
        nodes,
        edges,
        Options::default(),
        Limits::default(),
        cancelled,
    )
    .unwrap()
}

#[test]
fn canonical_parent_and_sibling_views_survive_input_permutation() {
    let nodes = [
        node("r", None),
        node("b", Some("r")),
        node("c", Some("a")),
        node("a", Some("r")),
    ];
    let edges = [edge("z", "b", "a"), edge("e", "a", "b")];
    let before = (format!("{nodes:?}"), format!("{edges:?}"));
    let a = accepted(&nodes, &edges);
    let b = accepted(
        &[nodes[2], nodes[0], nodes[3], nodes[1]],
        &[edges[1], edges[0]],
    );
    assert_eq!(
        a.nodes().iter().map(|n| n.id).collect::<Vec<_>>(),
        ["a", "b", "c", "r"]
    );
    assert_eq!(a.parent_indices(), [Some(3), Some(3), Some(0), None]);
    assert_eq!(a.bottom_up(), [2, 0, 1, 3]);
    assert_eq!(
        a.sibling_edges(),
        [(edges[1], Some(3)), (edges[0], Some(3))]
    );
    assert_eq!(format!("{:?}", a.nodes()), format!("{:?}", b.nodes()));
    assert_eq!(a.parent_indices(), b.parent_indices());
    assert_eq!(a.bottom_up(), b.bottom_up());
    assert_eq!(a.sibling_edges(), b.sibling_edges());
    assert_eq!(a.charged_work(), b.charged_work());
    assert_eq!(a.charged_work(), 53_416);
    assert_eq!(a.estimated_workspace_bytes(), 4 * 4096 + 2 * 1024);
    assert_eq!(before, (format!("{nodes:?}"), format!("{edges:?}")));
    assert_eq!(a.nodes()[0].leaf_size, size(97.125, 21.5));
}

#[test]
fn geometry_is_owned_while_ids_still_borrow_the_original_storage() {
    let id = String::from("persistent-id");
    let token = {
        let nodes = [node(&id, None)];
        accepted(&nodes, &[])
    };
    assert_eq!(token.nodes()[0].id, id);
    assert_eq!(token.nodes()[0].id.as_ptr(), id.as_ptr());
    assert_eq!(token.parent_indices(), [None]);
    assert_eq!(token.bottom_up(), [0]);
}

#[test]
fn exact_work_workspace_and_count_boundaries_are_distinct() {
    let nodes = [node("a", None), node("b", None)];
    let edges = [edge("e", "a", "b")];
    let baseline = accepted(&nodes, &edges);
    let exact = Limits {
        max_nodes: 2,
        max_edges: 1,
        max_work: baseline.charged_work(),
        max_workspace_bytes: 2 * 4096 + 1024,
        ..Limits::default()
    };
    let token = admit_packages(&nodes, &edges, Options::default(), exact, || false).unwrap();
    assert_eq!(token.charged_work(), exact.max_work);
    assert_eq!(token.estimated_workspace_bytes(), exact.max_workspace_bytes);
    assert_eq!(
        refusal(
            &nodes,
            &edges,
            Limits {
                max_work: exact.max_work - 1,
                ..exact
            }
        ),
        Error::Layout(LayoutError::Limit(Limit::Work))
    );
    assert_eq!(
        refusal(
            &nodes,
            &edges,
            Limits {
                max_workspace_bytes: exact.max_workspace_bytes - 1,
                ..exact
            }
        ),
        Error::Layout(LayoutError::Limit(Limit::Memory))
    );
    for field in 0..5 {
        let mut limits = Limits::default();
        let kind = match field {
            0 => {
                limits.max_nodes = 1;
                Limit::Nodes
            }
            1 => {
                limits.max_edges = 0;
                Limit::Edges
            }
            2 => {
                limits.max_work = 0;
                Limit::Work
            }
            3 => {
                limits.max_workspace_bytes = 0;
                Limit::Memory
            }
            _ => {
                limits.max_id_bytes = 0;
                Limit::IdBytes
            }
        };
        assert_eq!(
            refusal(&nodes, &edges, limits),
            Error::Layout(LayoutError::Limit(kind))
        );
    }
    let overflowing_sort = Limits {
        max_id_bytes: usize::MAX,
        max_work: usize::MAX,
        ..Limits::default()
    };
    assert_eq!(
        refusal(&nodes, &[], overflowing_sort),
        Error::Layout(LayoutError::Limit(Limit::Work))
    );
    let one_byte = Limits {
        max_id_bytes: 1,
        ..Limits::default()
    };
    assert!(admit_packages(&nodes, &edges, Options::default(), one_byte, || false).is_ok());
    assert_eq!(
        refusal(&[node("é", None)], &[], one_byte),
        Error::Layout(LayoutError::Limit(Limit::IdBytes))
    );
}

#[test]
fn identity_and_endpoint_errors_remain_typed() {
    let limits = Limits::default();
    assert_eq!(
        refusal(&[node("", None)], &[], limits),
        Error::Layout(LayoutError::EmptyId)
    );
    assert_eq!(
        refusal(&[node("a", None), node("a", None)], &[], limits),
        Error::Layout(LayoutError::DuplicateNode)
    );
    let nodes = [node("a", None), node("b", None)];
    assert_eq!(
        refusal(&nodes, &[edge("", "a", "b")], limits),
        Error::Layout(LayoutError::EmptyId)
    );
    assert_eq!(
        refusal(&nodes, &[edge("e", "", "b")], limits),
        Error::Layout(LayoutError::EmptyId)
    );
    assert_eq!(
        refusal(&nodes, &[edge("e", "a", "missing")], limits),
        Error::Layout(LayoutError::UnknownEndpoint)
    );
    let duplicate = [edge("e", "a", "b"), edge("e", "b", "a")];
    assert_eq!(
        refusal(&nodes, &duplicate, limits),
        Error::Layout(LayoutError::DuplicateEdge)
    );
}

#[test]
fn parent_cycles_unknown_parents_and_cross_parent_edges_are_separate() {
    let limits = Limits::default();
    for parent in ["unknown", ""] {
        assert_eq!(
            refusal(&[node("a", Some(parent))], &[], limits),
            Error::UnknownParent
        );
    }
    assert_eq!(
        refusal(&[node("a", Some("a"))], &[], limits),
        Error::ParentCycle
    );
    assert_eq!(
        refusal(&[node("a", Some("b")), node("b", Some("a"))], &[], limits),
        Error::ParentCycle
    );
    let nodes = [
        node("p", None),
        node("q", None),
        node("a", Some("p")),
        node("b", Some("q")),
    ];
    assert_eq!(
        refusal(&nodes, &[edge("e", "a", "b")], limits),
        Error::CrossParentEdge
    );
    assert_eq!(
        refusal(&nodes, &[edge("e", "p", "a")], limits),
        Error::CrossParentEdge
    );
    let token = admit_packages(
        &nodes,
        &[edge("e", "p", "q")],
        Options::default(),
        limits,
        || false,
    )
    .unwrap();
    assert_eq!(token.sibling_edges()[0].1, None);
    let long_parent = "x".repeat(1025);
    assert_eq!(
        refusal(&[node("a", Some(&long_parent))], &[], limits),
        Error::Layout(LayoutError::Limit(Limit::IdBytes))
    );
}

#[test]
fn every_unused_geometry_axis_is_validated_before_publication() {
    for field in 0..8 {
        for number in [f64::NAN, f64::INFINITY, -1.] {
            let mut parent = node("p", None);
            match field {
                0 => parent.leaf_size.width = number,
                1 => parent.leaf_size.height = number,
                2 => parent.header_height_footprint = number,
                3 => parent.border_width = number,
                4 => parent.default_width = Some(number),
                5 => parent.default_height = Some(number),
                6 => parent.user_minimum = Some(size(number, 1.)),
                _ => parent.user_minimum = Some(size(1., number)),
            }
            assert_eq!(
                refusal(&[parent, node("c", Some("p"))], &[], Limits::default()),
                Error::Layout(LayoutError::InvalidNumber)
            );
        }
    }
    for x in [f64::NAN, f64::INFINITY] {
        let mut leaf = node("a", None);
        leaf.retained_position = Some(Position { x, y: 0. });
        assert_eq!(
            refusal(&[leaf], &[], Limits::default()),
            Error::Layout(LayoutError::InvalidNumber)
        );
        leaf.retained_position = Some(Position { x: 0., y: x });
        assert_eq!(
            refusal(&[leaf], &[], Limits::default()),
            Error::Layout(LayoutError::InvalidNumber)
        );
    }
}

#[test]
fn negative_retained_positions_signed_zero_and_fractional_sizes_are_preserved() {
    let mut leaf = node("a", None);
    leaf.retained_position = Some(Position { x: -13.75, y: -0. });
    leaf.leaf_size.height = -0.;
    leaf.user_minimum = Some(size(1.125, 2.25));
    let token = accepted(&[leaf], &[]);
    assert_eq!(token.nodes()[0].retained_position, leaf.retained_position);
    assert_eq!(
        token.nodes()[0].leaf_size.height.to_bits(),
        (-0.0_f64).to_bits()
    );
    assert_eq!(token.nodes()[0].leaf_size.width, 97.125);
    assert_eq!(token.nodes()[0].user_minimum, leaf.user_minimum);
}

#[test]
fn all_options_are_checked_even_for_an_empty_forest() {
    for field in 0..4 {
        for number in [f64::NAN, f64::NEG_INFINITY, -1.] {
            let mut options = Options::default();
            match field {
                0 => options.layer_gap = number,
                1 => options.node_gap = number,
                2 => options.component_gap = number,
                _ => options.padding = number,
            }
            assert_eq!(
                admit_packages(&[], &[], options, Limits::default(), || false).unwrap_err(),
                Error::Layout(LayoutError::InvalidNumber)
            );
        }
    }
    let options = Options {
        direction: Direction::Down,
        padding: 0.,
        ..Options::default()
    };
    let token = admit_packages(&[], &[], options, Limits::default(), || false).unwrap();
    assert!(
        token.nodes().is_empty()
            && token.bottom_up().is_empty()
            && token.sibling_edges().is_empty()
    );
    assert_eq!(token.charged_work(), 0);
    assert_eq!(token.estimated_workspace_bytes(), 0);
    assert_eq!(token.options().direction, Direction::Down);
    let limits = token.limits();
    assert_eq!(
        (
            limits.max_nodes,
            limits.max_edges,
            limits.max_work,
            limits.max_workspace_bytes,
            limits.max_id_bytes
        ),
        (4096, 16384, 100_000_000, 16 * 1024 * 1024, 1024)
    );
}

#[test]
fn original_callback_is_polled_at_every_phase_and_final_publication() {
    let nodes = [node("p", None), node("c", Some("p"))];
    let polls = Cell::new(0);
    let token = admit_packages(&nodes, &[], Options::default(), Limits::default(), || {
        polls.set(polls.get() + 1);
        false
    })
    .unwrap();
    let total = polls.get();
    assert!(total > 10);
    let _ = (
        token.nodes(),
        token.parent_indices(),
        token.bottom_up(),
        token.charged_work(),
    );
    assert_eq!(polls.get(), total);
    for stop in 1..=total {
        let calls = Cell::new(0);
        let result = admit_packages(&nodes, &[], Options::default(), Limits::default(), || {
            calls.set(calls.get() + 1);
            calls.get() == stop
        });
        assert_eq!(result.unwrap_err(), Error::Layout(LayoutError::Cancelled));
        assert_eq!(calls.get(), stop);
    }
    assert_eq!(
        format!("{nodes:?}"),
        format!("{:?}", [node("p", None), node("c", Some("p"))])
    );
}

#[test]
fn deep_forest_order_is_canonical_and_cycles_cannot_publish_a_token() {
    let ids: Vec<_> = (0..64).map(|i| format!("node-{i:02}")).collect();
    let nodes: Vec<_> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| node(id, i.checked_sub(1).map(|parent| ids[parent].as_str())))
        .collect();
    let token = accepted(&nodes, &[]);
    assert_eq!(token.bottom_up(), (0..64).rev().collect::<Vec<_>>());
    assert_eq!(token.parent_indices()[0], None);
    assert_eq!(token.parent_indices()[63], Some(62));
    assert_eq!(token.estimated_workspace_bytes(), 64 * 4096);
    let mut cycle = nodes.clone();
    cycle[0].parent = Some(&ids[63]);
    assert_eq!(refusal(&cycle, &[], Limits::default()), Error::ParentCycle);
    let wide_ids: Vec<_> = (0..256).map(|i| format!("wide-{i:03}")).collect();
    let wide: Vec<_> = wide_ids.iter().map(|id| node(id, None)).collect();
    assert_eq!(
        refusal(&wide, &[], Limits::default()),
        Error::Layout(LayoutError::Limit(Limit::Work))
    );
}

#[test]
fn empty_publication_retains_the_original_cancellation_callback() {
    let polls = Cell::new(0);
    let token = admit_packages(&[], &[], Options::default(), Limits::default(), || {
        polls.set(polls.get() + 1);
        false
    })
    .unwrap();
    let total = polls.get();
    assert!(total > 2 && token.nodes().is_empty());
    let calls = Cell::new(0);
    let result = admit_packages(&[], &[], Options::default(), Limits::default(), || {
        calls.set(calls.get() + 1);
        calls.get() == total
    });
    assert_eq!(result.unwrap_err(), Error::Layout(LayoutError::Cancelled));
    assert_eq!(calls.get(), total);
}
