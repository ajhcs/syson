// SPDX-License-Identifier: EPL-2.0
use std::sync::{Arc, OnceLock, atomic::AtomicBool};
use syson_ast::{Arena, NodeId, Slot, Value};
use syson_ast_adapter::{Adapter, Error, Limits};
use syson_reflection::{OrderingQualification, ReflectionDb};
fn adapter() -> &'static Adapter {
    static ADAPTER: OnceLock<Adapter> = OnceLock::new();
    ADAPTER.get_or_init(|| Adapter::pinned().unwrap())
}
fn build(source: &str) -> Arena {
    adapter()
        .construct(source, Limits::default(), Arc::new(AtomicBool::new(false)))
        .unwrap()
}
fn slot<'a>(arena: &'a Arena, node: NodeId, property: &str) -> &'a Value {
    match arena.node(node).unwrap().slot(property) {
        Slot::Present(value) => &value.value,
        Slot::Absent => panic!("missing {property}"),
    }
}
fn child(arena: &Arena, node: NodeId, property: &str, index: usize) -> NodeId {
    let Value::Array(values) = slot(arena, node, property) else {
        panic!("not an array")
    };
    let Value::Node(id) = values[index].value else {
        panic!("not a node")
    };
    id
}
fn target(arena: &Arena, node: NodeId) -> NodeId {
    let Value::Node(id) = slot(arena, node, "target") else {
        panic!("not a target")
    };
    *id
}
fn kind(arena: &Arena, node: NodeId) -> &str {
    arena.node(node).unwrap().kind().unwrap()
}
fn assert_parent(
    arena: &Arena,
    node: NodeId,
    parent: NodeId,
    property: &str,
    index: Option<usize>,
) {
    let actual = arena.node(node).unwrap().container().unwrap();
    assert_eq!(
        (actual.node, actual.property.as_str(), actual.index),
        (parent, property, index)
    );
}
#[test]
fn package_part_definition_and_typed_usage_have_source_shaped_wrappers() {
    let source = "package Vehicle { part def Wheel; part wheel: Wheel; }";
    let arena = build(source);
    assert_eq!(arena.nodes().len(), 9);
    let root = arena.root();
    let package_member = child(&arena, root, "children", 0);
    let package = target(&arena, package_member);
    let definition_member = child(&arena, package, "children", 0);
    let definition = target(&arena, definition_member);
    let usage_member = child(&arena, package, "children", 1);
    let usage = target(&arena, usage_member);
    for (node, expected) in [
        (root, "Namespace"),
        (package_member, "OwningMembership"),
        (package, "Package"),
        (definition_member, "OwningMembership"),
        (definition, "PartDefinition"),
        (usage_member, "OwningMembership"),
        (usage, "PartUsage"),
    ] {
        assert_eq!(kind(&arena, node), expected);
    }
    assert_eq!(
        slot(&arena, package, "declaredName"),
        &Value::String("Vehicle".into())
    );
    assert_eq!(
        slot(&arena, definition, "declaredName"),
        &Value::String("Wheel".into())
    );
    assert_eq!(
        slot(&arena, usage, "declaredName"),
        &Value::String("wheel".into())
    );
    assert_parent(&arena, package_member, root, "children", Some(0));
    assert_parent(&arena, package, package_member, "target", None);
    assert_parent(&arena, usage_member, package, "children", Some(1));
    assert_parent(&arena, usage, usage_member, "target", None);
}
#[test]
fn usage_membership_depends_on_package_or_definition_body_context() {
    let arena = build("part top: T; part def D { part inner: T; package P; }");
    let top = child(&arena, arena.root(), "children", 0);
    assert_eq!(kind(&arena, top), "OwningMembership");
    let definition = target(&arena, child(&arena, arena.root(), "children", 1));
    let inner = child(&arena, definition, "children", 0);
    let package = child(&arena, definition, "children", 1);
    assert_eq!(kind(&arena, inner), "FeatureMembership");
    assert_eq!(kind(&arena, package), "OwningMembership");
    assert_parent(&arena, inner, definition, "children", Some(0));
    assert_parent(&arena, target(&arena, inner), inner, "target", None);
}
#[test]
fn qualified_typing_stays_unresolved_with_raw_name_segments_and_parenting() {
    let source = "part wheel: Lib :: 'Ty\\'pe';";
    let arena = build(source);
    let usage = target(&arena, child(&arena, arena.root(), "children", 0));
    let typing = child(&arena, usage, "heritage", 0);
    assert_eq!(kind(&arena, typing), "FeatureTyping");
    let Value::Node(reference) = slot(&arena, typing, "targetRef") else {
        panic!()
    };
    assert_eq!(kind(&arena, *reference), "TypeReference");
    let Value::Array(parts) = slot(&arena, *reference, "parts") else {
        panic!()
    };
    let db = ReflectionDb::pinned();
    let reference_type = db.resolve_type("TypeReference").unwrap();
    for (index, (part, text)) in parts.iter().zip(["Lib", "'Ty\\'pe'"]).enumerate() {
        let position = db
            .reference_position(&reference_type, index, parts.len())
            .unwrap();
        let marker = db.reference_target_type(&position).unwrap();
        let target_name = db.name(&marker).unwrap();
        assert_eq!(target_name, if index == 0 { "Element" } else { "Type" });
        assert_eq!(
            part.value,
            Value::UnresolvedReference {
                text: text.into(),
                target_type: target_name.into()
            }
        );
        assert_eq!(part.lexeme.as_ref().unwrap().text, text);
    }
    assert_eq!(parts.len(), 2);
    assert_eq!(
        arena.node(typing).unwrap().origin().unwrap().text,
        "Lib :: 'Ty\\'pe'"
    );
    assert_parent(&arena, typing, usage, "heritage", Some(0));
    assert_parent(&arena, *reference, typing, "targetRef", None);
}
#[test]
fn raw_quoted_names_preserve_quotes_escapes_and_both_span_units() {
    let source = "package '🚗\\n' { part def 'W\\'heel'; }";
    let arena = build(source);
    let package = target(&arena, child(&arena, arena.root(), "children", 0));
    let Slot::Present(name) = arena.node(package).unwrap().slot("declaredName") else {
        panic!()
    };
    assert_eq!(name.value, Value::String("'🚗\\n'".into()));
    let lexeme = name.lexeme.as_ref().unwrap();
    assert_eq!(&source[lexeme.span.bytes.clone()], lexeme.text);
    assert_eq!(lexeme.span.utf16.len(), 6);
    assert_eq!(lexeme.span.bytes.len(), 8);
    assert_eq!(
        arena
            .node(arena.root())
            .unwrap()
            .origin()
            .unwrap()
            .span
            .utf16
            .end,
        source.encode_utf16().count()
    );
    for node in arena.nodes().values() {
        let origin = node.origin().unwrap();
        assert_eq!(&source[origin.span.bytes.clone()], origin.text);
        assert_eq!(
            origin.span.utf16.start,
            source[..origin.span.bytes.start].encode_utf16().count()
        );
    }
}
#[test]
fn anonymous_and_empty_bodies_keep_optional_names_absent() {
    for source in ["package;", "package {}", "part def;", "part def {}"] {
        let arena = build(source);
        let declaration = target(&arena, child(&arena, arena.root(), "children", 0));
        assert!(matches!(
            arena.node(declaration).unwrap().slot("declaredName"),
            Slot::Absent
        ));
        assert_eq!(slot(&arena, declaration, "children"), &Value::Array(vec![]));
    }
    let empty = build(" \n\t");
    assert_eq!(empty.nodes().len(), 1);
    assert!(empty.node(empty.root()).unwrap().container().is_none());
}
#[test]
fn defaults_and_absent_optionals_are_distinct_from_empty_or_false_values() {
    let arena = build("part x: T;");
    let usage = target(&arena, child(&arena, arena.root(), "children", 0));
    for property in ["isSufficient", "isNonunique", "isOrdered"] {
        assert_eq!(slot(&arena, usage, property), &Value::Boolean(false));
    }
    for property in ["children", "prefixes", "typeRelationships", "$children"] {
        assert_eq!(slot(&arena, usage, property), &Value::Array(vec![]));
    }
    for property in ["isAbstract", "declaredShortName", "multiplicity"] {
        assert!(matches!(
            arena.node(usage).unwrap().slot(property),
            Slot::Absent
        ));
    }
    assert_eq!(
        adapter().ordering_qualification(),
        OrderingQualification::SourceDerivedStableShortSortCandidate
    );
}
#[test]
fn repeated_declarations_preserve_encounter_order_and_distinct_identity() {
    let arena = build("package Z; package A; package Z;");
    let ids: Vec<_> = (0..3)
        .map(|i| target(&arena, child(&arena, arena.root(), "children", i)))
        .collect();
    assert_ne!(ids[0], ids[2]);
    for (id, name) in ids.into_iter().zip(["Z", "A", "Z"]) {
        assert_eq!(
            slot(&arena, id, "declaredName"),
            &Value::String(name.into())
        );
    }
}
#[test]
fn malformed_finite_pairs_produce_no_arena_and_retain_precise_offsets() {
    for (source, expected, offset) in [
        ("package P {", "}", 11),
        ("part def D", "{", 10),
        ("part x: ;", "name", 8),
        ("part x: T::;", "name", 11),
        ("part x: T", ";", 9),
        ("}", "declaration", 0),
        ("package 'bad", "closing name quote", 12),
        ("package 'a\\q';", "admitted name escape", 11),
    ] {
        assert!(
            matches!(adapter().construct(source,Limits::default(),Arc::new(AtomicBool::new(false))),
            Err(Error::Malformed {offset:at,expected:what}) if at==offset && what==expected),
            "{source}"
        );
    }
}
#[test]
fn valid_source_outside_whitelist_is_explicitly_unsupported() {
    for source in [
        "private package P;",
        "part x;",
        "part x: T {}",
        "package <short> Long;",
        "package P; //* comment",
        "port def P;",
        "part def D :> B;",
        "package P;\u{feff}",
    ] {
        assert!(
            matches!(
                adapter().construct(source, Limits::default(), Arc::new(AtomicBool::new(false))),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
}
#[test]
fn keyword_union_never_silently_constructs_keyword_as_regular_name() {
    for source in [
        "package def;",
        "package private;",
        "part in: T;",
        "part x: package;",
    ] {
        assert!(
            matches!(
                adapter().construct(source, Limits::default(), Arc::new(AtomicBool::new(false))),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    build("package 'private';");
}
#[test]
fn all_frontend_limits_and_invalid_depths_fail_closed() {
    let source = "package P { part x: T; }";
    for limits in [
        Limits {
            source_bytes: source.len() - 1,
            ..Limits::default()
        },
        Limits {
            tokens: 1,
            ..Limits::default()
        },
        Limits {
            declarations: 1,
            ..Limits::default()
        },
        Limits {
            depth: 1,
            ..Limits::default()
        },
    ] {
        assert!(matches!(
            adapter().construct(source, limits, Arc::new(AtomicBool::new(false))),
            Err(Error::Limit)
        ));
    }
    for depth in [0, 65] {
        assert!(matches!(
            adapter().construct(
                "",
                Limits {
                    depth,
                    ..Limits::default()
                },
                Arc::new(AtomicBool::new(false))
            ),
            Err(Error::InvalidLimits)
        ));
    }
}
#[test]
fn kernel_limit_and_precancelled_input_never_return_partial_arena() {
    let limits = Limits {
        kernel: syson_ast::Limits {
            max_nodes: 2,
            ..syson_ast::Limits::default()
        },
        ..Limits::default()
    };
    assert!(matches!(
        adapter().construct("package P;", limits, Arc::new(AtomicBool::new(false))),
        Err(Error::Kernel(syson_ast::Error::Budget))
    ));
    assert!(matches!(
        adapter().construct(
            "package P;",
            Limits::default(),
            Arc::new(AtomicBool::new(true))
        ),
        Err(Error::Cancelled)
    ));
}
#[test]
fn prior_arenas_are_immutable_and_node_handles_do_not_cross_owners() {
    let old = build("package Old;");
    let new = build("package New;");
    let old_package = target(&old, child(&old, old.root(), "children", 0));
    assert!(new.node(old_package).is_none());
    assert_eq!(
        slot(&old, old_package, "declaredName"),
        &Value::String("Old".into())
    );
}

#[test]
fn cancelled_kernel_operations_keep_cancellation_distinct_from_other_failures() {
    let cancel = Arc::new(AtomicBool::new(false));
    let mut builder = syson_ast::Builder::new(
        syson_ast::Reflection::default(),
        syson_ast::Limits::default(),
        cancel.clone(),
    )
    .unwrap();
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    let failure = builder.allocate(None, None).unwrap_err();
    assert_eq!(failure, syson_ast::Error::Cancelled);
    assert_eq!(Error::from(failure), Error::Cancelled);

    let mut bounded = syson_ast::Builder::new(
        syson_ast::Reflection::default(),
        syson_ast::Limits {
            max_nodes: 0,
            ..syson_ast::Limits::default()
        },
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    let failure = bounded.allocate(None, None).unwrap_err();
    assert_eq!(
        Error::from(failure),
        Error::Kernel(syson_ast::Error::Budget)
    );
    let failure = syson_ast::Lexeme::from_source("🚗", 1, 2).unwrap_err();
    assert_eq!(
        Error::from(failure),
        Error::Kernel(syson_ast::Error::InvalidSpan)
    );
}

#[test]
fn value_conversion_cancellation_keeps_its_category_and_other_failures_stay_conversion() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use syson_ast_values::{Control, Dispatch, Lexeme, Presence, Span, convert};
    let input = Lexeme {
        text: "Name",
        span: Span { start: 0, end: 4 },
    };
    for cancelled in [false, true] {
        let polls = AtomicUsize::new(0);
        let control = Control {
            max_utf16_units: 0,
            is_cancelled: &|| cancelled && polls.fetch_add(1, Ordering::Relaxed) > 0,
        };
        let failure = convert(Dispatch::Raw, Presence::Value(input), &control).unwrap_err();
        assert_eq!(failure.span, Some(input.span));
        let expected = if cancelled {
            Error::Cancelled
        } else {
            Error::Conversion
        };
        assert_eq!(Error::from(failure), expected);
    }
}
