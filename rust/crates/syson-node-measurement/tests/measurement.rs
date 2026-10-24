// Copyright (c) 2023, 2024, 2025, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
use syson_node_measurement::*;

const STAMP: MeasurementStamp = MeasurementStamp {
    layout_revision: 1,
    font_revision: 2,
    request_id: 3,
};
#[derive(Default)]
struct Host {
    bounds: Option<Size>,
    stamp: Option<MeasurementStamp>,
    target: Option<LabelTarget>,
    fail: bool,
    calls: Vec<(LabelTarget, String, String, f64, MeasurementStamp)>,
}
impl MeasurementHost for Host {
    type Error = &'static str;
    fn measure_label(
        &mut self,
        request: &LabelRequest<'_>,
    ) -> Result<LabelObservation, Self::Error> {
        self.calls.push((
            request.target.clone(),
            request.text.into(),
            request.font.family.into(),
            request.font.size_px,
            request.stamp,
        ));
        if self.fail {
            return Err("font/DOM provider unavailable");
        }
        Ok(LabelObservation {
            target: self
                .target
                .clone()
                .unwrap_or_else(|| request.target.clone()),
            stamp: self.stamp.unwrap_or(request.stamp),
            bounds: self.bounds,
        })
    }
}
fn node(kind: NodeKind) -> NodeGeometry<'static> {
    NodeGeometry {
        id: "node",
        kind,
        current: OptionalSize::default(),
        defaults: OptionalSize::default(),
        forced: OptionalSize::default(),
        is_new: false,
        resized_by_user: false,
        keep_aspect_ratio: false,
        direct_children: 0,
        border: BorderWidths::default(),
    }
}
fn request<'a>(
    node: NodeGeometry<'a>,
    previous: &'a [PreviousNode<'a>],
    visible: &'a [&'a str],
) -> LayoutRequest<'a> {
    LayoutRequest {
        node,
        previous,
        visible_node_ids: visible,
        label_text: "Vehicle\nPower",
        font: FontSpec {
            family: "Inter",
            size_px: 14.0,
            weight: 500,
            letter_spacing_px: 0.0,
            line_height_px: Some(20.0),
        },
        stamp: STAMP,
    }
}
fn size(width: f64, height: f64) -> Size {
    Size { width, height }
}
fn optional(width: Option<f64>, height: Option<f64>) -> OptionalSize {
    OptionalSize { width, height }
}
fn measured(node: NodeGeometry<'_>, label_height: Option<f64>) -> Size {
    let mut host = Host {
        bounds: label_height.map(|h| size(400.0, h)),
        ..Host::default()
    };
    measure_node(&mut host, request(node, &[], &["node"]))
        .unwrap()
        .size
}

#[test]
fn only_the_three_renderer_types_are_admitted() {
    for (name, kind) in [
        ("sysMLPackageNode", NodeKind::Package),
        ("sysMLViewFrameNode", NodeKind::ViewFrame),
        ("sysMLNoteNode", NodeKind::Note),
    ] {
        assert_eq!(NodeKind::from_renderer_type(name), Some(kind));
    }
    for name in [
        "",
        "package",
        "sysMLImportedPackageNode",
        "sysMLPartNode",
        "sysMLNoteNode\0",
    ] {
        assert_eq!(NodeKind::from_renderer_type(name), None);
    }
}
#[test]
fn leaf_defaults_and_caller_label_height_match_both_pinned_handlers() {
    for kind in [NodeKind::Package, NodeKind::ViewFrame] {
        assert_eq!(measured(node(kind), None), size(75.0, 50.0));
        assert_eq!(measured(node(kind), Some(60.0)), size(75.0, 76.0));
        let mut n = node(kind);
        n.defaults = optional(Some(140.0), Some(80.0));
        assert_eq!(measured(n, Some(20.0)), size(140.0, 80.0));
        n.defaults = optional(Some(0.0), Some(0.0));
        assert_eq!(measured(n, None), size(0.0, 16.0));
    }
}
#[test]
fn leaf_force_values_suppress_retention_but_do_not_become_dimensions() {
    for kind in [NodeKind::Package, NodeKind::ViewFrame] {
        let mut n = node(kind);
        n.current = optional(Some(400.0), Some(300.0));
        n.resized_by_user = true;
        assert_eq!(measured(n, Some(20.0)), size(400.0, 300.0));
        n.forced = optional(Some(900.0), Some(800.0));
        assert_eq!(measured(n, Some(20.0)), size(75.0, 50.0));
        n.forced = optional(Some(0.0), Some(-0.0));
        assert_eq!(measured(n, Some(20.0)), size(400.0, 300.0));
    }
}
#[test]
fn previous_dimensions_use_new_previous_and_current_branches_with_per_axis_defaults() {
    let mut n = node(NodeKind::Package);
    n.resized_by_user = true;
    n.current = optional(Some(900.0), Some(800.0));
    n.defaults = optional(Some(120.0), Some(90.0));
    let previous = [PreviousNode {
        id: "node",
        dimensions: optional(Some(600.0), Some(500.0)),
    }];
    let mut host = Host::default();
    n.is_new = true;
    assert_eq!(
        measure_node(&mut host, request(n, &previous, &[]))
            .unwrap()
            .size,
        size(120.0, 90.0)
    );
    n.is_new = false;
    assert_eq!(
        measure_node(&mut host, request(n, &previous, &[]))
            .unwrap()
            .size,
        size(600.0, 500.0)
    );
    let missing = [PreviousNode {
        id: "node",
        dimensions: optional(None, Some(180.0)),
    }];
    assert_eq!(
        measure_node(&mut host, request(n, &missing, &[]))
            .unwrap()
            .size,
        size(120.0, 180.0)
    );
    assert_eq!(
        measure_node(&mut host, request(n, &[], &[])).unwrap().size,
        size(900.0, 800.0)
    );
    n.current = OptionalSize::default();
    n.defaults = OptionalSize::default();
    assert_eq!(
        measure_node(&mut host, request(n, &[], &[])).unwrap().size,
        size(150.0, 70.0)
    );
}
#[test]
fn previous_and_visible_lookup_keep_source_first_match_order() {
    let mut n = node(NodeKind::ViewFrame);
    n.resized_by_user = true;
    let previous = [
        PreviousNode {
            id: "other",
            dimensions: optional(Some(999.0), None),
        },
        PreviousNode {
            id: "node",
            dimensions: optional(Some(210.0), Some(180.0)),
        },
        PreviousNode {
            id: "node",
            dimensions: optional(Some(700.0), Some(600.0)),
        },
    ];
    let mut host = Host::default();
    let result =
        measure_node(&mut host, request(n, &previous, &["other", "node", "node"])).unwrap();
    assert_eq!(result.size, size(210.0, 180.0));
    assert_eq!(
        host.calls[0].0,
        LabelTarget::DomBoundingClientRect {
            element_id: "node-label-1".into()
        }
    );
    assert_eq!(
        (
            &host.calls[0].1,
            &host.calls[0].2,
            host.calls[0].3,
            host.calls[0].4
        ),
        (
            &"Vehicle\nPower".to_string(),
            &"Inter".to_string(),
            14.0,
            STAMP
        )
    );
    measure_node(&mut host, request(n, &[], &[])).unwrap();
    assert_eq!(
        host.calls[1].0,
        LabelTarget::DomBoundingClientRect {
            element_id: "node-label--1".into()
        }
    );
}
#[test]
fn note_height_defaults_border_and_forced_dimensions_use_the_note_contract() {
    let mut n = node(NodeKind::Note);
    assert_eq!(measured(n, None), size(200.0, 70.0));
    n.border.all = Some(2.0);
    assert_eq!(measured(n, Some(100.0)), size(200.0, 104.0));
    n.current.width = Some(130.0);
    n.defaults.height = Some(120.0);
    assert_eq!(measured(n, Some(20.0)), size(130.0, 120.0));
    n.forced = optional(Some(90.0), Some(40.0));
    assert_eq!(measured(n, Some(1000.0)), size(90.0, 40.0));
    n.forced = optional(Some(0.0), Some(0.0));
    assert_eq!(measured(n, Some(1000.0)), size(0.0, 0.0));
}
#[test]
fn note_zero_force_is_nullish_for_selection_but_falsy_for_resized_retention() {
    let mut n = node(NodeKind::Note);
    n.current = optional(Some(400.0), Some(300.0));
    n.forced = optional(Some(0.0), Some(0.0));
    n.resized_by_user = true;
    assert_eq!(measured(n, Some(60.0)), size(400.0, 300.0));
    n.forced = optional(Some(90.0), Some(40.0));
    assert_eq!(measured(n, Some(1000.0)), size(90.0, 40.0));
}
#[test]
fn note_border_left_override_and_nonfinite_fallback_match_numeric_css_helper() {
    let mut n = node(NodeKind::Note);
    n.defaults.height = Some(0.0);
    n.border = BorderWidths {
        left: Some(2.5),
        all: Some(12.0),
    };
    assert_eq!(measured(n, Some(100.0)), size(200.0, 105.0));
    n.border.left = Some(f64::NAN);
    assert_eq!(measured(n, Some(100.0)), size(200.0, 100.0));
    n.border = BorderWidths {
        left: None,
        all: Some(f64::INFINITY),
    };
    assert_eq!(measured(n, Some(100.0)), size(200.0, 100.0));
    n.border.left = Some(0.0);
    assert_eq!(measured(n, Some(100.0)), size(200.0, 100.0));
}
#[test]
fn aspect_ratio_expands_one_axis_with_source_truthy_default_fallbacks() {
    let mut n = node(NodeKind::Note);
    n.keep_aspect_ratio = true;
    n.defaults = optional(Some(100.0), Some(50.0));
    n.forced = optional(Some(200.0), Some(200.0));
    assert_eq!(measured(n, None), size(400.0, 200.0));
    n.forced = optional(Some(300.0), Some(100.0));
    assert_eq!(measured(n, None), size(300.0, 150.0));
    n.defaults = optional(Some(0.0), Some(0.0));
    n.forced = optional(Some(150.0), Some(70.0));
    assert_eq!(measured(n, None), size(150.0, 70.0));
    n.forced = optional(Some(0.0), Some(50.0));
    assert_eq!(measured(n, None), size(0.0, 50.0));
}
#[test]
fn missing_label_observation_is_preserved_and_host_errors_are_explicit() {
    let mut host = Host::default();
    let result = measure_node(&mut host, request(node(NodeKind::Package), &[], &[])).unwrap();
    assert_eq!(result.label.bounds, None);
    assert_eq!(result.size, size(75.0, 50.0));
    host.fail = true;
    assert_eq!(
        measure_node(&mut host, request(node(NodeKind::Note), &[], &[])),
        Err(MeasurementError::Host("font/DOM provider unavailable"))
    );
}
#[test]
fn stale_layout_font_and_request_observations_are_refused() {
    for stale in [
        MeasurementStamp {
            layout_revision: 0,
            ..STAMP
        },
        MeasurementStamp {
            font_revision: 0,
            ..STAMP
        },
        MeasurementStamp {
            request_id: 0,
            ..STAMP
        },
    ] {
        let mut host = Host {
            stamp: Some(stale),
            ..Host::default()
        };
        assert_eq!(
            measure_node(&mut host, request(node(NodeKind::Package), &[], &[])),
            Err(MeasurementError::StaleObservation)
        );
    }
}
#[test]
fn a_different_label_target_is_not_accepted_with_a_matching_stamp() {
    let mut host = Host {
        target: Some(LabelTarget::InsideLabelModel {
            node_id: "sibling".into(),
        }),
        ..Host::default()
    };
    assert_eq!(
        measure_node(&mut host, request(node(NodeKind::Note), &[], &[])),
        Err(MeasurementError::WrongTarget)
    );
}
#[test]
fn unsupported_container_paths_fail_before_requesting_host_measurement() {
    for kind in [NodeKind::Package, NodeKind::ViewFrame] {
        let mut n = node(kind);
        n.direct_children = 1;
        let mut host = Host::default();
        assert_eq!(
            measure_node(&mut host, request(n, &[], &[])),
            Err(MeasurementError::UnsupportedParent(kind))
        );
        assert!(host.calls.is_empty());
    }
    let mut n = node(NodeKind::Note);
    n.direct_children = 1; // The note's source handler ignores children.
    assert_eq!(measured(n, None), size(200.0, 70.0));
}
#[test]
fn invalid_dimension_label_and_font_numbers_are_rejected() {
    for bad in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for field in 0..6 {
            let mut n = node(NodeKind::Package);
            match field {
                0 => n.current.width = Some(bad),
                1 => n.current.height = Some(bad),
                2 => n.defaults.width = Some(bad),
                3 => n.defaults.height = Some(bad),
                4 => n.forced.width = Some(bad),
                _ => n.forced.height = Some(bad),
            }
            let mut host = Host::default();
            assert!(matches!(
                measure_node(&mut host, request(n, &[], &[])),
                Err(MeasurementError::InvalidNumber(_))
            ));
            assert!(host.calls.is_empty());
        }
        for bounds in [size(bad, 20.0), size(30.0, bad)] {
            let mut host = Host {
                bounds: Some(bounds),
                ..Host::default()
            };
            assert!(matches!(
                measure_node(&mut host, request(node(NodeKind::Note), &[], &[])),
                Err(MeasurementError::InvalidNumber(_))
            ));
        }
        let mut host = Host::default();
        let mut r = request(node(NodeKind::Package), &[], &[]);
        r.font.size_px = bad;
        assert_eq!(
            measure_node(&mut host, r),
            Err(MeasurementError::InvalidNumber("font size"))
        );
        r.font.size_px = 14.0;
        r.font.line_height_px = Some(bad);
        assert_eq!(
            measure_node(&mut host, r),
            Err(MeasurementError::InvalidNumber("font line height"))
        );
    }
}
#[test]
fn invalid_matching_previous_sizes_and_negative_borders_are_explicit() {
    let previous = [PreviousNode {
        id: "node",
        dimensions: optional(Some(f64::NAN), None),
    }];
    let mut host = Host::default();
    assert_eq!(
        measure_node(&mut host, request(node(NodeKind::Note), &previous, &[])),
        Err(MeasurementError::InvalidNumber("previous width"))
    );
    let mut n = node(NodeKind::Note);
    n.border.all = Some(-1.0);
    assert_eq!(
        measure_node(&mut host, request(n, &[], &[])),
        Err(MeasurementError::InvalidNumber("border width"))
    );
}
#[test]
fn finite_input_arithmetic_overflow_is_not_reported_as_measured_success() {
    let mut host = Host {
        bounds: Some(size(10.0, f64::MAX)),
        ..Host::default()
    };
    assert_eq!(
        measure_node(&mut host, request(node(NodeKind::Package), &[], &[]))
            .unwrap()
            .size,
        size(75.0, f64::MAX)
    );
    let mut n = node(NodeKind::Note);
    n.border.all = Some(f64::MAX);
    assert_eq!(
        measure_node(&mut host, request(n, &[], &[])),
        Err(MeasurementError::InvalidNumber("computed height"))
    );
    n.border = BorderWidths::default();
    n.defaults = optional(Some(f64::MAX), Some(f64::MIN_POSITIVE));
    n.forced = optional(Some(100.0), Some(100.0));
    n.keep_aspect_ratio = true;
    assert_eq!(
        measure_node(&mut Host::default(), request(n, &[], &[])),
        Err(MeasurementError::InvalidNumber("computed width"))
    );
}
#[test]
fn finite_negative_letter_spacing_is_admitted_but_nonfinite_is_not() {
    let mut host = Host::default();
    let mut r = request(node(NodeKind::Package), &[], &[]);
    r.font.letter_spacing_px = -2.0;
    assert_eq!(measure_node(&mut host, r).unwrap().size, size(75.0, 50.0));
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        r.font.letter_spacing_px = bad;
        assert_eq!(
            measure_node(&mut host, r),
            Err(MeasurementError::InvalidNumber("letter spacing"))
        );
    }
}
