// SPDX-License-Identifier: EPL-2.0
use serde_json::json;
use std::sync::{Arc, OnceLock, atomic::AtomicBool};
use syson_ast::{Arena, NodeId, Slot, Value};
use syson_ast_adapter::{Adapter, Limits};
use syson_linker::{Alias, Binding, Error, Snapshot};
use syson_metamodel::ClassifierId as C;
use syson_model::Document;
use syson_references::{FragmentIdentity, Registry, Resource};
const NODE: &str = "aaaaaaaa-0000-4000-8000-000000000001";
const ELEMENT: &str = "bbbbbbbb-0000-4000-8000-000000000001";
fn parse(text: &str) -> Arena {
    static ADAPTER: OnceLock<Adapter> = OnceLock::new();
    ADAPTER
        .get_or_init(|| Adapter::pinned().unwrap())
        .construct(text, Limits::default(), Arc::new(AtomicBool::new(false)))
        .unwrap()
}
fn named(arena: &Arena, name: &str) -> Vec<NodeId> {
    arena
        .nodes()
        .iter()
        .filter_map(|(id, node)| match node.slot("declaredName") {
            Slot::Present(d) if d.value == Value::String(name.into()) => Some(*id),
            _ => None,
        })
        .collect()
}
fn references(arena: &Arena) -> Vec<NodeId> {
    arena
        .nodes()
        .iter()
        .filter(|(_, n)| n.kind() == Some("TypeReference"))
        .map(|(id, _)| *id)
        .collect()
}
fn registry(class: &str) -> Registry {
    let doc = Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},
        "content":[{"id":NODE,"eClass":class,"data":{"elementId":ELEMENT,"declaredName":"Explicit semantic object"}}]}).to_string()).unwrap();
    Registry::new(vec![Resource {
        key: "resource-A".into(),
        document: doc,
        fragments: FragmentIdentity::SemanticElementId,
    }])
    .unwrap()
}
fn empty(arena: Arena, aliases: Vec<Alias>) -> Result<Snapshot, Error> {
    Snapshot::new(
        arena,
        registry("sysml:PartDefinition"),
        "resource-A".into(),
        aliases,
        vec![],
    )
}
fn alias(scope: NodeId, name: &str, parts: &[&str]) -> Alias {
    Alias {
        scope,
        name: name.into(),
        parts: parts.iter().map(|s| s.to_string()).collect(),
    }
}
#[test]
fn actual_f11_reference_resolves_then_explicit_f04_binding_preserves_both_ids() {
    let arena =
        parse("package Lib { part def Wheel; } package Vehicle { part wheel: Lib :: Wheel; }");
    let reference = references(&arena)[0];
    let wheel = named(&arena, "Wheel")[0];
    let before = format!("{:?}", arena.nodes());
    let snapshot = Snapshot::new(
        arena,
        registry("sysml:PartDefinition"),
        "resource-A".into(),
        vec![],
        vec![Binding {
            declaration: wheel,
            wire: json!(NODE),
        }],
    )
    .unwrap();
    let link = snapshot.link(reference).unwrap();
    assert_eq!(link.target(), Some(wheel));
    assert!(link.error().is_none());
    assert_eq!(link.found().len(), 2);
    assert_eq!(link.reference(), reference);
    let target = snapshot.semantic(&link).unwrap();
    assert_eq!(
        (target.resource(), target.node_id(), target.element_id()),
        ("resource-A", NODE, ELEMENT)
    );
    assert_eq!(
        snapshot.value(&target).unwrap()["data"]["declaredName"],
        "Explicit semantic object"
    );
    assert_eq!(format!("{:?}", snapshot.arena().nodes()), before);
    let Slot::Present(parts) = snapshot.arena().node(reference).unwrap().slot("parts") else {
        panic!()
    };
    let Value::Array(parts) = &parts.value else {
        panic!()
    };
    assert!(
        parts
            .iter()
            .all(|p| matches!(p.value, Value::UnresolvedReference { .. }))
    );
}
#[test]
fn nearest_lexical_scope_wins_without_testing_later_parent_candidates() {
    let arena = parse("part def T; package P { part def T; part x:T; }");
    let ts = named(&arena, "T");
    let reference = references(&arena)[0];
    let snapshot = empty(arena, vec![]).unwrap();
    assert_eq!(snapshot.link(reference).unwrap().target(), Some(ts[1]));
}
#[test]
fn later_qualified_segments_do_not_fall_back_to_the_parent_scope() {
    let arena = parse("part def Wheel; package Lib; part x:Lib::Wheel;");
    let reference = references(&arena)[0];
    let lib = named(&arena, "Lib")[0];
    let snapshot = empty(arena, vec![]).unwrap();
    let link = snapshot.link(reference).unwrap();
    assert_eq!(link.found(), [lib]);
    assert_eq!(link.target(), None);
    assert_eq!(link.error(), Some(&(1, Error::Unresolved("Wheel".into()))));
}
#[test]
fn wrong_type_is_retained_as_found_but_never_as_a_resolved_target() {
    let arena = parse("package T; part x:T;");
    let reference = references(&arena)[0];
    let package = named(&arena, "T")[0];
    let snapshot = empty(arena, vec![]).unwrap();
    let link = snapshot.link(reference).unwrap();
    assert_eq!(link.found(), [package]);
    assert_eq!(link.target(), None);
    assert_eq!(
        link.error(),
        Some(&(
            0,
            Error::TypeMismatch {
                expected: C::Type,
                actual: C::Package
            }
        ))
    );
    assert!(matches!(
        snapshot.semantic(&link),
        Err(Error::TypeMismatch { .. })
    ));
}
#[test]
fn caller_observed_alias_chain_resolves_without_claiming_f11_alias_parsing() {
    let arena = parse("package P { part def Wheel; part wheel:A; }");
    let scope = named(&arena, "P")[0];
    let wheel = named(&arena, "Wheel")[0];
    let reference = references(&arena)[0];
    let snapshot = empty(
        arena,
        vec![alias(scope, "A", &["B"]), alias(scope, "B", &["Wheel"])],
    )
    .unwrap();
    let link = snapshot.link(reference).unwrap();
    assert_eq!(link.target(), Some(wheel));
    assert_eq!(
        snapshot.semantic(&link).unwrap_err(),
        Error::MissingBinding(wheel)
    );
}
#[test]
fn alias_cycles_fail_without_publishing_a_partial_target() {
    let arena = parse("part x:A;");
    let root = arena.root();
    let reference = references(&arena)[0];
    let snapshot = empty(
        arena,
        vec![alias(root, "A", &["B"]), alias(root, "B", &["A"])],
    )
    .unwrap();
    let link = snapshot.link(reference).unwrap();
    assert_eq!(link.target(), None);
    assert!(link.found().is_empty());
    assert_eq!(link.error(), Some(&(0, Error::AliasCycle)));
}
#[test]
fn qualified_alias_targets_use_their_observed_scope_and_wrong_types_stay_visible() {
    let arena = parse("package Lib { part def T; } package P { part x:A; part y:B; }");
    let scope = named(&arena, "P")[0];
    let t = named(&arena, "T")[0];
    let refs = references(&arena);
    let snapshot = empty(
        arena,
        vec![
            alias(scope, "A", &["Lib", "T"]),
            alias(scope, "B", &["Lib"]),
        ],
    )
    .unwrap();
    assert_eq!(snapshot.link(refs[0]).unwrap().target(), Some(t));
    let wrong = snapshot.link(refs[1]).unwrap();
    assert!(matches!(
        wrong.error(),
        Some((
            0,
            Error::TypeMismatch {
                actual: C::Package,
                ..
            }
        ))
    ));
}
#[test]
fn missing_alias_target_and_case_mismatch_are_explicit_unresolved_outcomes() {
    let arena = parse("part def Wheel; part a:A; part b:WHEEL;");
    let root = arena.root();
    let refs = references(&arena);
    let snapshot = empty(arena, vec![alias(root, "A", &["Missing"])]).unwrap();
    assert_eq!(
        snapshot.link(refs[0]).unwrap().error(),
        Some(&(0, Error::Unresolved("Missing".into())))
    );
    assert_eq!(
        snapshot.link(refs[1]).unwrap().error(),
        Some(&(0, Error::Unresolved("WHEEL".into())))
    );
}
#[test]
fn duplicate_or_ambiguous_sanitized_names_refuse_snapshot_admission() {
    assert!(matches!(
        empty(parse("part def T; part def 'T';"), vec![]),
        Err(Error::Ambiguous)
    ));
    let arena = parse("part def T;");
    let root = arena.root();
    assert!(matches!(
        empty(arena, vec![alias(root, "T", &["T"])]),
        Err(Error::Ambiguous)
    ));
    let arena = parse("");
    let root = arena.root();
    assert!(matches!(
        empty(
            arena,
            vec![alias(root, "A", &["T"]), alias(root, "'A'", &["T"])]
        ),
        Err(Error::Ambiguous)
    ));
}
#[test]
fn foreign_alias_scopes_and_foreign_or_duplicate_binding_nodes_refuse() {
    let foreign = parse("part def T;");
    let foreign_t = named(&foreign, "T")[0];
    assert!(matches!(
        empty(
            parse("part def T;"),
            vec![alias(foreign.root(), "A", &["T"])]
        ),
        Err(Error::Unsupported)
    ));
    let arena = parse("part def T;");
    let t = named(&arena, "T")[0];
    assert!(matches!(
        Snapshot::new(
            arena,
            registry("sysml:PartDefinition"),
            "resource-A".into(),
            vec![],
            vec![
                Binding {
                    declaration: t,
                    wire: json!(NODE)
                },
                Binding {
                    declaration: t,
                    wire: json!(NODE)
                }
            ]
        ),
        Err(Error::Unsupported)
    ));
    assert!(matches!(
        Snapshot::new(
            parse("part def T;"),
            registry("sysml:PartDefinition"),
            "resource-A".into(),
            vec![],
            vec![Binding {
                declaration: foreign_t,
                wire: json!(NODE)
            }]
        ),
        Err(Error::Unsupported)
    ));
}
#[test]
fn semantic_bindings_never_fall_back_from_json_node_id_to_semantic_element_id() {
    let arena = parse("part def T; part x:T;");
    let t = named(&arena, "T")[0];
    let reference = references(&arena)[0];
    let snapshot = Snapshot::new(
        arena,
        registry("sysml:PartDefinition"),
        "resource-A".into(),
        vec![],
        vec![Binding {
            declaration: t,
            wire: json!(ELEMENT),
        }],
    )
    .unwrap();
    let link = snapshot.link(reference).unwrap();
    assert_eq!(link.target(), Some(t));
    assert!(matches!(
        snapshot.semantic(&link),
        Err(Error::Reference(syson_references::Error::Unresolved { .. }))
    ));
}
#[test]
fn semantic_mapping_checks_the_actual_declaration_classifier() {
    let arena = parse("part def T; part x:T;");
    let t = named(&arena, "T")[0];
    let reference = references(&arena)[0];
    let snapshot = Snapshot::new(
        arena,
        registry("sysml:Package"),
        "resource-A".into(),
        vec![],
        vec![Binding {
            declaration: t,
            wire: json!(NODE),
        }],
    )
    .unwrap();
    let link = snapshot.link(reference).unwrap();
    assert_eq!(link.target(), Some(t));
    assert!(matches!(
        snapshot.semantic(&link),
        Err(Error::Reference(syson_references::Error::TypeMismatch {
            expected: C::PartDefinition,
            actual: C::Package
        }))
    ));
}
#[test]
fn result_and_semantic_targets_cannot_be_replayed_in_an_identical_new_snapshot() {
    fn make() -> (Snapshot, NodeId) {
        let arena = parse("part def T; part x:T;");
        let t = named(&arena, "T")[0];
        let reference = references(&arena)[0];
        (
            Snapshot::new(
                arena,
                registry("sysml:PartDefinition"),
                "resource-A".into(),
                vec![],
                vec![Binding {
                    declaration: t,
                    wire: json!(NODE),
                }],
            )
            .unwrap(),
            reference,
        )
    }
    let (a, reference) = make();
    let link = a.link(reference).unwrap();
    let target = a.semantic(&link).unwrap();
    let (b, _) = make();
    assert_eq!(b.semantic(&link).unwrap_err(), Error::ForeignResult);
    assert!(matches!(
        b.value(&target),
        Err(Error::Reference(syson_references::Error::StaleSnapshot))
    ));
    assert!(matches!(b.link(reference), Err(Error::Unsupported)));
}
#[test]
fn quoted_unicode_and_escaped_names_are_sanitized_for_lookup_without_editing_syntax() {
    let arena = parse("part def 'Ty\\'pe\\n🚗'; part x:'Ty\\'pe\\n🚗';");
    let t = named(&arena, "'Ty\\'pe\\n🚗'")[0];
    let reference = references(&arena)[0];
    let snapshot = empty(arena, vec![]).unwrap();
    assert_eq!(snapshot.link(reference).unwrap().target(), Some(t));
    let arena = parse("part def 'T'; part x:T;");
    let t = named(&arena, "'T'")[0];
    let reference = references(&arena)[0];
    assert_eq!(
        empty(arena, vec![])
            .unwrap()
            .link(reference)
            .unwrap()
            .target(),
        Some(t)
    );
}
#[test]
fn alias_work_admission_is_bounded_and_external_proxy_bindings_are_not_admitted() {
    let arena = parse("part def T;");
    let root = arena.root();
    let aliases = (0..257)
        .map(|i| alias(root, &format!("A{i}"), &["T"]))
        .collect();
    assert!(matches!(empty(arena, aliases), Err(Error::Limit)));
    let arena = parse("part def T;");
    let root = arena.root();
    assert!(matches!(
        empty(
            arena,
            vec![Alias {
                scope: root,
                name: "A".into(),
                parts: vec!["T".into(); 65]
            }]
        ),
        Err(Error::Limit)
    ));
    let arena = parse("part def T;");
    let t = named(&arena, "T")[0];
    assert!(matches!(
        Snapshot::new(
            arena,
            registry("sysml:PartDefinition"),
            "resource-A".into(),
            vec![],
            vec![Binding {
                declaration: t,
                wire: json!(
                    "sysml:PartDefinition sysmllibrary:///11111111-1111-4111-8111-111111111111#bbbbbbbb-0000-4000-8000-000000000001"
                )
            }]
        ),
        Err(Error::Unsupported)
    ));
}

fn boundary_source(declarations: usize) -> String {
    assert!(declarations >= 3);
    let mut source = String::from("package P {");
    for n in 0..declarations - 2 {
        source.push_str(&format!("part def T{n};"));
    }
    source.push_str(&format!("part component:T{};}}", declarations - 3));
    source
}
#[test]
fn default_parser_declaration_boundary_excludes_the_synthetic_namespace() {
    for declarations in [255, 256] {
        let arena = parse(&boundary_source(declarations));
        let target = named(&arena, &format!("T{}", declarations - 3))[0];
        let reference = references(&arena)[0];
        let snapshot = empty(arena, vec![]).unwrap_or_else(|e| {
            panic!("{declarations} admitted real declarations must fit the linker: {e:?}")
        });
        assert_eq!(snapshot.link(reference).unwrap().target(), Some(target));
    }
}
#[test]
fn custom_parser_budget_does_not_expand_the_linker_declaration_budget() {
    let arena = Adapter::pinned()
        .unwrap()
        .construct(
            &boundary_source(257),
            Limits {
                declarations: 257,
                ..Limits::default()
            },
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(matches!(empty(arena, vec![]), Err(Error::Limit)));
}
#[test]
fn default_parser_still_refuses_257_real_declarations() {
    assert!(matches!(
        Adapter::pinned().unwrap().construct(
            &boundary_source(257),
            Limits::default(),
            Arc::new(AtomicBool::new(false))
        ),
        Err(syson_ast_adapter::Error::Limit)
    ));
}

#[test]
fn a_real_package_root_does_not_receive_a_synthetic_root_allowance() {
    use syson_ast::{Assignment, Builder, Datum, DefaultValue, Reflection};
    let build = |declarations| {
        let mut reflection = Reflection::default();
        for kind in ["Package", "PartDefinition"] {
            reflection
                .declare(kind.into(), vec![DefaultValue::Array("children".into())])
                .unwrap();
        }
        reflection
            .declare("OwningMembership".into(), vec![])
            .unwrap();
        let mut builder = Builder::new(
            reflection,
            syson_ast::Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        let root = builder.allocate(Some("Package"), None).unwrap();
        for n in 1..declarations {
            let child = builder.allocate(Some("PartDefinition"), None).unwrap();
            builder
                .assign(
                    child,
                    "declaredName",
                    Assignment::Set,
                    Datum::new(Value::String(format!("T{n}"))),
                )
                .unwrap();
            let membership = builder.allocate(Some("OwningMembership"), None).unwrap();
            builder
                .assign(membership, "target", Assignment::Set, Datum::node(child))
                .unwrap();
            builder
                .assign(
                    root,
                    "children",
                    Assignment::Append,
                    Datum::node(membership),
                )
                .unwrap();
        }
        builder.finish(root).unwrap()
    };
    assert!(empty(build(256), vec![]).is_ok());
    assert!(matches!(empty(build(257), vec![]), Err(Error::Limit)));
}
