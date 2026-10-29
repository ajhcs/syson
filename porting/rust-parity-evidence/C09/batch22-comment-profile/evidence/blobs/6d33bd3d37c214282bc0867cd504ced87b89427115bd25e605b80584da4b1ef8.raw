// SPDX-License-Identifier: EPL-2.0
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use syson_ast::{Arena, Assignment, Builder, Datum, Lexeme, Limits, NodeId, Slot, Value};
use syson_ast_adapter::operator_expression::{Error, OperatorPostprocessor, Report};
use syson_reflection::OrderingQualification;

fn processor() -> OperatorPostprocessor {
    OperatorPostprocessor::pinned().unwrap()
}
fn builder(p: &OperatorPostprocessor) -> Builder {
    p.builder(Limits::default(), Arc::new(AtomicBool::new(false)))
        .unwrap()
}
fn assign(b: &mut Builder, id: NodeId, property: &str, value: Value) {
    b.assign(id, property, Assignment::Set, Datum::new(value))
        .unwrap();
}
fn expression(b: &mut Builder, operator: Option<&str>) -> NodeId {
    let id = b.allocate(Some("OperatorExpression"), None).unwrap();
    if let Some(operator) = operator {
        assign(b, id, "operator", Value::String(operator.into()));
    }
    id
}
fn node_value(arena: &Arena, node: NodeId, property: &str) -> NodeId {
    match arena.node(node).unwrap().slot(property) {
        Slot::Present(Datum {
            value: Value::Node(id),
            ..
        }) => *id,
        _ => panic!("missing node property {property}"),
    }
}
fn operands(arena: &Arena, node: NodeId) -> Vec<NodeId> {
    match arena.node(node).unwrap().slot("operands") {
        Slot::Present(Datum {
            value: Value::Array(values),
            ..
        }) => values
            .iter()
            .map(|d| match d.value {
                Value::Node(id) => id,
                _ => panic!("not a node operand"),
            })
            .collect(),
        _ => panic!("missing operands"),
    }
}
fn assert_parent(
    arena: &Arena,
    child: NodeId,
    parent: NodeId,
    property: &str,
    index: Option<usize>,
) {
    let actual = arena.node(child).unwrap().container().unwrap();
    assert_eq!(
        (actual.node, actual.property.as_str(), actual.index),
        (parent, property, index)
    );
}
fn inserted(report: Report) -> (NodeId, NodeId, NodeId, bool) {
    match report {
        Report::Inserted {
            expression,
            membership,
            feature,
            normalized_operands,
        } => (expression, membership, feature, normalized_operands),
        _ => panic!("expected inserted report: {report:?}"),
    }
}

#[test]
fn four_exact_classification_operators_insert_the_source_placeholder_chain() {
    let p = processor();
    for operator in ["istype", "hastype", "@", "as"] {
        let mut b = builder(&p);
        let root = expression(&mut b, Some(operator));
        let (expr, member, feature, normalized) = inserted(p.apply(&mut b, root).unwrap());
        assert!(normalized);
        let arena = b.finish(root).unwrap();
        assert_eq!(arena.nodes().len(), 4);
        assert_eq!(operands(&arena, root), vec![expr]);
        assert_eq!(node_value(&arena, expr, "expression"), member);
        assert_eq!(node_value(&arena, member, "target"), feature);
        for (id, kind) in [
            (expr, "FeatureReferenceExpression"),
            (member, "ReturnParameterMembership"),
            (feature, "Feature"),
        ] {
            assert_eq!(arena.node(id).unwrap().kind(), Some(kind));
            assert!(arena.node(id).unwrap().origin().is_none());
        }
        assert_parent(&arena, expr, root, "operands", Some(0));
        assert_parent(&arena, member, expr, "expression", None);
        assert_parent(&arena, feature, member, "target", None);
    }
}
#[test]
fn explicit_empty_array_is_truthy_and_does_not_report_normalization() {
    let p = processor();
    let mut b = builder(&p);
    let root = expression(&mut b, Some("as"));
    assign(&mut b, root, "operands", Value::Array(vec![]));
    assert!(!inserted(p.apply(&mut b, root).unwrap()).3);
    assert_eq!(b.finish(root).unwrap().nodes().len(), 4);
}
#[test]
fn representable_falsey_operand_values_are_normalized_before_insertion() {
    let p = processor();
    for value in [
        Value::Null,
        Value::Boolean(false),
        Value::Number(0.0),
        Value::Number(-0.0),
        Value::Number(f64::NAN),
        Value::String(String::new()),
    ] {
        let mut b = builder(&p);
        let root = expression(&mut b, Some("istype"));
        assign(&mut b, root, "operands", value);
        assert!(inserted(p.apply(&mut b, root).unwrap()).3);
        assert_eq!(b.finish(root).unwrap().nodes().len(), 4);
    }
}
#[test]
fn missing_falsey_or_nonmatching_operator_only_normalizes_operands() {
    let p = processor();
    for value in [
        None,
        Some(Value::Null),
        Some(Value::Boolean(false)),
        Some(Value::Boolean(true)),
        Some(Value::Number(1.0)),
        Some(Value::String(String::new())),
        Some(Value::String("isType".into())),
        Some(Value::String("@@".into())),
        Some(Value::String("meta".into())),
        Some(Value::String("as ".into())),
    ] {
        let mut b = builder(&p);
        let root = expression(&mut b, None);
        if let Some(value) = value {
            assign(&mut b, root, "operator", value);
        }
        assert_eq!(
            p.apply(&mut b, root).unwrap(),
            Report::Unchanged {
                normalized_operands: true
            }
        );
        let arena = b.finish(root).unwrap();
        assert_eq!(arena.nodes().len(), 1);
        assert!(operands(&arena, root).is_empty());
    }
}
#[test]
fn unary_binary_and_nonempty_classification_preserve_operand_order() {
    let p = processor();
    for (operator, count) in [("-", 1), ("+", 2), ("as", 2)] {
        let mut b = builder(&p);
        let root = expression(&mut b, Some(operator));
        let mut expected = Vec::new();
        for number in 0..count {
            let leaf = b.allocate(Some("LiteralNumber"), None).unwrap();
            assign(&mut b, leaf, "literal", Value::Number(number as f64));
            b.assign(root, "operands", Assignment::Append, Datum::node(leaf))
                .unwrap();
            expected.push(leaf);
        }
        assert_eq!(
            p.apply(&mut b, root).unwrap(),
            Report::Unchanged {
                normalized_operands: false
            }
        );
        let arena = b.finish(root).unwrap();
        assert_eq!(operands(&arena, root), expected);
        assert_eq!(arena.nodes().len(), 1 + count);
        for (index, leaf) in expected.into_iter().enumerate() {
            assert_parent(&arena, leaf, root, "operands", Some(index));
        }
    }
}
#[test]
fn nested_processing_is_explicit_and_repeated_application_is_idempotent() {
    let p = processor();
    let mut b = builder(&p);
    let root = expression(&mut b, Some("+"));
    let nested = expression(&mut b, Some("hastype"));
    let leaf = b.allocate(Some("LiteralNumber"), None).unwrap();
    for operand in [nested, leaf] {
        b.assign(root, "operands", Assignment::Append, Datum::node(operand))
            .unwrap();
    }
    assert_eq!(
        p.apply(&mut b, root).unwrap(),
        Report::Unchanged {
            normalized_operands: false
        }
    );
    assert!(matches!(
        b.node(nested).unwrap().slot("operands"),
        Slot::Absent
    ));
    let (placeholder, member, feature, _) = inserted(p.apply(&mut b, nested).unwrap());
    assert_eq!(
        p.apply(&mut b, nested).unwrap(),
        Report::Unchanged {
            normalized_operands: false
        }
    );
    let arena = b.finish(root).unwrap();
    assert_eq!(arena.nodes().len(), 6);
    assert_eq!(operands(&arena, root), [nested, leaf]);
    assert_eq!(operands(&arena, nested), [placeholder]);
    assert_parent(&arena, nested, root, "operands", Some(0));
    assert_parent(&arena, leaf, root, "operands", Some(1));
    assert_parent(&arena, placeholder, nested, "operands", Some(0));
    assert_parent(&arena, member, placeholder, "expression", None);
    assert_parent(&arena, feature, member, "target", None);
}
#[test]
fn nonempty_operands_do_not_require_a_string_operator_or_rewrite_other_fields() {
    let p = processor();
    let mut b = builder(&p);
    let root = expression(&mut b, None);
    let leaf = b.allocate(Some("LiteralBoolean"), None).unwrap();
    b.assign(root, "operands", Assignment::Append, Datum::node(leaf))
        .unwrap();
    assign(&mut b, root, "operator", Value::Boolean(true));
    let before = b.node(root).unwrap().fields().clone();
    assert_eq!(
        p.apply(&mut b, root).unwrap(),
        Report::Unchanged {
            normalized_operands: false
        }
    );
    assert_eq!(b.node(root).unwrap().fields(), &before);
}
#[test]
fn truthy_nonarray_operands_are_an_explicit_unchanged_admission_error() {
    let p = processor();
    for value in [
        Value::Boolean(true),
        Value::Number(7.0),
        Value::String("truthy".into()),
    ] {
        let mut b = builder(&p);
        let root = expression(&mut b, Some("as"));
        assign(&mut b, root, "operands", value);
        let before = b.node(root).unwrap().fields().clone();
        assert_eq!(p.apply(&mut b, root), Err(Error::UnsupportedOperands));
        assert_eq!(b.node(root).unwrap().fields(), &before);
    }
}
#[test]
fn other_dispatch_entries_are_not_executed_or_subject_to_operand_admission() {
    let p = processor();
    for kind in [
        "WhileLoopActionUsage",
        "Import",
        "TransitionUsage",
        "SuccessionAsUsage",
        "Feature",
    ] {
        let mut b = builder(&p);
        let root = b.allocate(Some(kind), None).unwrap();
        assign(&mut b, root, "operands", Value::Boolean(true));
        let before = b.node(root).unwrap().fields().clone();
        assert_eq!(p.apply(&mut b, root).unwrap(), Report::NotSelected);
        assert_eq!(b.node(root).unwrap().fields(), &before);
    }
}
#[test]
fn allocation_failure_rolls_back_normalization_new_nodes_and_all_parent_writes() {
    let p = processor();
    for maximum in [1, 3] {
        let mut b = p
            .builder(
                Limits {
                    max_nodes: maximum,
                    ..Limits::default()
                },
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        let root = expression(&mut b, Some("as"));
        let before = b.node(root).unwrap().fields().clone();
        assert!(matches!(
            p.apply(&mut b, root),
            Err(Error::Kernel(syson_ast::Error::Budget))
        ));
        assert_eq!(b.node(root).unwrap().fields(), &before);
        if maximum == 3 {
            let first = b.allocate(Some("Feature"), None).unwrap();
            let second = b.allocate(Some("Feature"), None).unwrap();
            assert!(
                first.value() > root.value() + 1,
                "spent IDs must not be recycled"
            );
            assert!(second.value() > first.value());
        }
    }
}
#[test]
fn cancellation_and_foreign_or_missing_handles_do_not_mutate_the_target() {
    let p = processor();
    let cancel = Arc::new(AtomicBool::new(false));
    let mut a = p.builder(Limits::default(), cancel.clone()).unwrap();
    let root = expression(&mut a, Some("as"));
    let before = a.node(root).unwrap().fields().clone();
    let mut b = builder(&p);
    let foreign = expression(&mut b, Some("as"));
    assert!(matches!(
        p.apply(&mut a, foreign),
        Err(Error::Kernel(syson_ast::Error::ForeignOwner(_)))
    ));
    let missing = a.allocate(None, None).unwrap();
    assert!(matches!(
        p.apply(&mut a, missing),
        Err(Error::Kernel(syson_ast::Error::MissingType(_)))
    ));
    cancel.store(true, Ordering::Relaxed);
    assert_eq!(
        p.apply(&mut a, root),
        Err(Error::Kernel(syson_ast::Error::Cancelled))
    );
    assert_eq!(a.node(root).unwrap().fields(), &before);
}
#[test]
fn source_cst_is_preserved_and_native_cache_aliases_remain_explicitly_unimplemented() {
    let p = processor();
    let mut b = builder(&p);
    let source = "hastype X";
    let root = b
        .allocate(
            Some("OperatorExpression"),
            Some(Lexeme::from_source(source, 0, source.len()).unwrap()),
        )
        .unwrap();
    assign(&mut b, root, "operator", Value::String("hastype".into()));
    let (expr, member, feature, _) = inserted(p.apply(&mut b, root).unwrap());
    let arena = b.finish(root).unwrap();
    assert_eq!(arena.node(root).unwrap().origin().unwrap().text, source);
    for node in [root, expr, member, feature] {
        assert!(
            matches!(arena.node(node).unwrap().slot("$children"), Slot::Present(Datum { value: Value::Array(v), .. }) if v.is_empty())
        );
    }
    for property in ["isNonunique", "isOrdered"] {
        assert!(matches!(
            arena.node(feature).unwrap().slot(property),
            Slot::Present(Datum {
                value: Value::Boolean(false),
                ..
            })
        ));
    }
    assert_eq!(
        p.ordering_qualification(),
        OrderingQualification::SourceDerivedStableShortSortCandidate
    );
}
