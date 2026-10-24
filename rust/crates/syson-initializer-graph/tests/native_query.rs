// SPDX-License-Identifier: EPL-2.0
//! Source-derived ElementUtil query cases; pinned ElementUtilTest tests UUIDs.
//! These local adapter tests do not establish upstream runtime parity.
use serde_json::{Value, json};
use syson_domain::model::Visibility;
use syson_initializer_graph::native_query::{
    ImportedMembershipObservation, ObjectId, ResourceContext, ResourceSet, ResourceSnapshot,
};
use syson_initializer_graph::{Error, NameKind, ReferenceEncoding};
use syson_metamodel::ClassifierId as C;
use syson_model::Document;

const LOCAL: ReferenceEncoding = ReferenceEncoding::SemanticElementIds;
fn object(resource: &str, element: &str) -> ObjectId {
    ObjectId::new(resource, element)
}
fn node(id: &str, class: &str, extra: Value) -> Value {
    let mut data = extra.as_object().unwrap().clone();
    data.insert("elementId".into(), json!(id));
    json!({"id":format!("wire-{id}"), "eClass":format!("sysml:{class}"), "data":data})
}
fn member(id: &str, children: Vec<Value>) -> Value {
    node(
        id,
        "OwningMembership",
        json!({"ownedRelatedElement":children}),
    )
}
fn package(id: &str, name: Option<&str>, relationships: Vec<Value>) -> Value {
    node(
        id,
        "Package",
        json!({"declaredName":name,"ownedRelationship":relationships}),
    )
}
fn document(roots: Vec<Value>) -> Document {
    Document::from_json(
        &json!({"json":{"version":"1.0"},
        "ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":roots})
        .to_string(),
    )
    .unwrap()
}
fn snapshot(resource: &str, roots: Vec<Value>) -> ResourceSnapshot {
    ResourceSnapshot::new(resource, document(roots))
}
fn context(resource: &str, element: &str) -> ResourceContext {
    ResourceContext::Member(object(resource, element))
}

#[test]
fn unqualified_raw_traversal_keeps_relationships_while_qualified_owned_elements_skip_wrappers() {
    let d = snapshot(
        "r",
        vec![package(
            "pkg",
            Some("Pkg"),
            vec![node(
                "wrapper",
                "OwningMembership",
                json!({"declaredName":"alias",
            "ownedRelatedElement":[node("part", "PartUsage", json!({"declaredName":"wheel"}))]}),
            )],
        )],
    );
    let resources = ResourceSet::new(vec![d]).unwrap();
    assert_eq!(
        resources
            .find_from(&context("r", "part"), "alias", C::Relationship, LOCAL)
            .unwrap(),
        Some(object("r", "wrapper"))
    );
    assert_eq!(
        resources
            .find_from(&context("r", "part"), "Pkg::alias", C::Relationship, LOCAL)
            .unwrap(),
        None
    );
    assert_eq!(
        resources
            .find_from(&context("r", "part"), "Pkg::wheel", C::Feature, LOCAL)
            .unwrap(),
        Some(object("r", "part"))
    );
    assert_eq!(
        resources.semantic_owner(&object("r", "wrapper")).unwrap(),
        None
    );
    assert_eq!(
        resources.semantic_owner(&object("r", "part")).unwrap(),
        Some(object("r", "pkg"))
    );
}

#[test]
fn explicit_contained_roots_do_not_become_top_level_unnamed_namespaces() {
    // A root OwningMembership has no owningRelatedElement. Its related Package
    // has no semantic owner, but still has a raw eContainer, so it is not an
    // unnamed top-level Namespace for findInRootByQualifiedNameAndTypeFrom.
    let resources = ResourceSet::new(vec![snapshot(
        "r",
        vec![member(
            "outer",
            vec![package(
                "contained",
                None,
                vec![member(
                    "nested",
                    vec![package(
                        "named",
                        Some("Named"),
                        vec![member(
                            "leaf-member",
                            vec![node("leaf", "PartUsage", json!({"declaredName":"leaf"}))],
                        )],
                    )],
                )],
            )],
        )],
    )])
    .unwrap();
    assert_eq!(
        resources.semantic_owner(&object("r", "contained")).unwrap(),
        None
    );
    assert_eq!(
        resources
            .find_in_roots(
                &[object("r", "contained")],
                "Named::leaf",
                C::PartUsage,
                LOCAL
            )
            .unwrap(),
        None
    );
    assert_eq!(
        resources
            .find_in_roots(&[object("r", "named")], "Named::leaf", C::PartUsage, LOCAL)
            .unwrap(),
        Some(object("r", "leaf"))
    );
}

#[test]
fn resource_and_root_order_are_preserved_with_semantic_identities_scoped_per_resource() {
    let resources = ResourceSet::new(vec![
        snapshot(
            "z-first",
            vec![package(
                "same",
                Some("Pkg"),
                vec![member(
                    "m",
                    vec![node("leaf", "PartUsage", json!({"declaredName":"target"}))],
                )],
            )],
        ),
        snapshot(
            "a-second",
            vec![package(
                "same",
                Some("Pkg"),
                vec![member(
                    "m",
                    vec![node("leaf", "PartUsage", json!({"declaredName":"target"}))],
                )],
            )],
        ),
    ])
    .unwrap();
    // Context's resource does not move it to the front of ResourceSet order.
    assert_eq!(
        resources
            .find_from(
                &context("a-second", "same"),
                "Pkg::target",
                C::PartUsage,
                LOCAL
            )
            .unwrap(),
        Some(object("z-first", "leaf"))
    );
    assert_eq!(
        resources
            .find_in_roots(
                &[object("a-second", "same"), object("z-first", "same")],
                "Pkg::target",
                C::PartUsage,
                LOCAL
            )
            .unwrap(),
        Some(object("a-second", "leaf"))
    );
    assert_eq!(
        resources
            .semantic_owner(&object("a-second", "leaf"))
            .unwrap(),
        Some(object("a-second", "same"))
    );
    let roots = ResourceSet::new(vec![snapshot(
        "r",
        vec![
            package("z-root", Some("duplicate"), vec![]),
            package("a-root", Some("duplicate"), vec![]),
        ],
    )])
    .unwrap();
    assert_eq!(
        roots
            .find_from(&context("r", "a-root"), "duplicate", C::Element, LOCAL)
            .unwrap(),
        Some(object("r", "z-root"))
    );
}

#[test]
fn absent_resourceset_is_known_absence_while_unregistered_member_and_duplicate_keys_are_errors() {
    let resources = ResourceSet::new(vec![snapshot(
        "r",
        vec![package("pkg", Some("Pkg"), vec![])],
    )])
    .unwrap();
    for c in [
        ResourceContext::Detached,
        ResourceContext::ResourceWithoutSet,
    ] {
        assert_eq!(
            resources.find_from(&c, "Pkg", C::Package, LOCAL).unwrap(),
            None
        );
        assert_eq!(resources.find_general_view(&c, LOCAL).unwrap(), None);
    }
    assert!(
        resources
            .find_from(
                &context("missing-resource", "pkg"),
                "Pkg",
                C::Package,
                LOCAL
            )
            .is_err()
    );
    assert!(
        resources
            .find_from(&context("r", "missing-element"), "Pkg", C::Package, LOCAL)
            .is_err()
    );
    assert!(
        ResourceSet::new(vec![
            snapshot("duplicate", vec![]),
            snapshot("duplicate", vec![])
        ])
        .is_err()
    );
}

#[test]
fn wrong_type_first_owned_name_prevents_import_fallback_but_unqualified_filters_type_first() {
    let resources = ResourceSet::new(vec![snapshot(
        "r",
        vec![package(
            "pkg",
            Some("Pkg"),
            vec![
                member(
                    "first",
                    vec![node("wrong", "PortUsage", json!({"declaredName":"same"}))],
                ),
                member(
                    "second",
                    vec![node("right", "PartUsage", json!({"declaredName":"same"}))],
                ),
                node(
                    "unresolved-import",
                    "MembershipImport",
                    json!({"importedMembership":"missing"}),
                ),
            ],
        )],
    )])
    .unwrap();
    assert_eq!(
        resources
            .find_from(&context("r", "pkg"), "Pkg::same", C::PartUsage, LOCAL)
            .unwrap(),
        None
    );
    assert_eq!(
        resources
            .find_from(&context("r", "pkg"), "same", C::PartUsage, LOCAL)
            .unwrap(),
        Some(object("r", "right"))
    );
    assert!(matches!(
        resources.find_from(&context("r", "pkg"), "Pkg::absent", C::PartUsage, LOCAL),
        Err(Error::Unresolved(_))
    ));
}

#[test]
fn explicit_import_observations_preserve_visibility_order_and_owned_precedence() {
    let resources = ResourceSet::new(vec![
        snapshot(
            "local",
            vec![package(
                "pkg",
                Some("Pkg"),
                vec![member(
                    "own",
                    vec![node("owned", "PortUsage", json!({"declaredName":"owned"}))],
                )],
            )],
        ),
        snapshot(
            "remote",
            vec![
                node("private", "PartUsage", json!({"declaredName":"target"})),
                node("first", "PartUsage", json!({"declaredName":"target"})),
                node("second", "PartUsage", json!({"declaredName":"target"})),
                node("shadowed", "PartUsage", json!({"declaredName":"owned"})),
            ],
        ),
    ])
    .unwrap()
    .with_imported_memberships(
        object("local", "pkg"),
        vec![
            ImportedMembershipObservation {
                visibility: Visibility::Private,
                related: vec![object("remote", "private")],
            },
            ImportedMembershipObservation {
                visibility: Visibility::Public,
                related: vec![
                    object("remote", "first"),
                    object("remote", "second"),
                    object("remote", "shadowed"),
                ],
            },
        ],
    )
    .unwrap();
    assert_eq!(
        resources
            .find_from(&context("local", "pkg"), "Pkg::target", C::PartUsage, LOCAL)
            .unwrap(),
        Some(object("remote", "first"))
    );
    assert_eq!(
        resources
            .find_from(&context("local", "pkg"), "Pkg::owned", C::PartUsage, LOCAL)
            .unwrap(),
        None
    );
    assert_eq!(
        resources
            .find_from(&context("local", "pkg"), "Pkg::owned", C::PortUsage, LOCAL)
            .unwrap(),
        Some(object("local", "owned"))
    );
}

#[test]
fn effective_short_names_and_inheritance_are_used_without_eagerly_reading_unvisited_gaps() {
    let resources = ResourceSet::new(vec![snapshot("r", vec![
        node("root", "PartUsage", json!({"ownedRelationship":[
            node("redef", "Redefinition", json!({"redefinedFeature":"named"})),
            member("child-member", vec![node("child", "PortUsage", json!({"declaredName":"child","declaredShortName":"c"}))]),
        ]})),
        node("named", "PartUsage", json!({"declaredName":"DerivedRoot","declaredShortName":"dr"})),
        node("unvisited", "PartUsage", json!({"ownedRelationship":[node("gap", "Redefinition", json!({"redefinedFeature":"remote#unresolved"}))]})),
    ])]).unwrap();
    assert_eq!(
        resources
            .effective_name(&object("r", "root"), NameKind::Name, LOCAL)
            .unwrap(),
        Some("DerivedRoot".into())
    );
    assert_eq!(
        resources
            .effective_name(&object("r", "root"), NameKind::ShortName, LOCAL)
            .unwrap(),
        Some("dr".into())
    );
    assert_eq!(
        resources
            .find_from(&context("r", "root"), "dr::c", C::Feature, LOCAL)
            .unwrap(),
        Some(object("r", "child"))
    );
    assert_eq!(
        resources
            .find_from(
                &context("r", "root"),
                "DerivedRoot::'child'",
                C::PortUsage,
                LOCAL
            )
            .unwrap(),
        Some(object("r", "child"))
    );
}

#[test]
fn unresolved_feature_names_do_not_become_absence_and_type_checks_remain_lazy() {
    let resources = ResourceSet::new(vec![snapshot("r", vec![
        node("feature", "PartUsage", json!({"ownedRelationship":[node("redef", "Redefinition", json!({"redefinedFeature":"outside"}))]})),
        package("pkg", Some("safe"), vec![]),
    ])]).unwrap();
    assert_eq!(
        resources
            .find_from(&context("r", "pkg"), "safe", C::Package, LOCAL)
            .unwrap(),
        Some(object("r", "pkg"))
    );
    assert!(matches!(
        resources.find_from(&context("r", "pkg"), "missing", C::Feature, LOCAL),
        Err(Error::Unresolved(_))
    ));
    assert!(
        resources
            .effective_name(
                &object("r", "feature"),
                NameKind::Name,
                ReferenceEncoding::Disabled
            )
            .is_err()
    );
}

#[test]
fn cyclic_foreign_proxy_and_wrong_type_naming_references_stay_explicit_errors() {
    for target in [
        json!("missing"),
        json!("remote#target"),
        json!({"$ref":"target"}),
        json!("package"),
    ] {
        let resources = ResourceSet::new(vec![snapshot("r", vec![
            node("root", "PartUsage", json!({"ownedRelationship":[node("redef", "Redefinition", json!({"redefinedFeature":target}))]})),
            package("package", Some("wrong type"), vec![]),
        ])]).unwrap();
        assert!(
            resources
                .effective_name(&object("r", "root"), NameKind::Name, LOCAL)
                .is_err()
        );
    }
    let resources = ResourceSet::new(vec![snapshot("r", vec![
        node("a", "PartUsage", json!({"ownedRelationship":[node("ra", "Redefinition", json!({"redefinedFeature":"b"}))]})),
        node("b", "PartUsage", json!({"ownedRelationship":[node("rb", "Redefinition", json!({"redefinedFeature":"a"}))]})),
    ])]).unwrap();
    assert!(
        resources
            .effective_name(&object("r", "a"), NameKind::Name, LOCAL)
            .is_err()
    );
}

#[test]
fn general_view_lookup_uses_pinned_qualified_name_type_and_resource_order() {
    let resources = ResourceSet::new(vec![
        snapshot(
            "application",
            vec![package("app", Some("Application"), vec![])],
        ),
        snapshot(
            "wrong-type-library",
            vec![package(
                "library",
                Some("StandardViewDefinitions"),
                vec![member(
                    "m",
                    vec![node(
                        "view",
                        "PartDefinition",
                        json!({"declaredName":"GeneralView"}),
                    )],
                )],
            )],
        ),
        snapshot(
            "stdlib",
            vec![package(
                "library",
                Some("StandardViewDefinitions"),
                vec![member(
                    "m",
                    vec![node(
                        "view",
                        "ViewDefinition",
                        json!({"declaredName":"GeneralView"}),
                    )],
                )],
            )],
        ),
    ])
    .unwrap();
    assert_eq!(
        resources
            .find_general_view(&context("application", "app"), LOCAL)
            .unwrap(),
        Some(object("stdlib", "view"))
    );
    assert_eq!(
        resources
            .find_from(
                &context("application", "app"),
                "StandardViewDefinitions::GeneralView",
                C::ViewDefinition,
                LOCAL
            )
            .unwrap(),
        Some(object("stdlib", "view"))
    );
}

#[test]
fn resource_snapshot_keeps_document_bytes_immutable_after_caller_edits() {
    let original = document(vec![package("pkg", Some("Before"), vec![])]);
    let before = original.to_json().unwrap();
    let resources = ResourceSet::new(vec![ResourceSnapshot::new("r", original.clone())]).unwrap();
    let mut editor = syson_model::Editor::new(original);
    let mut tx = editor.begin();
    tx.rename("pkg", "After").unwrap();
    editor.commit(tx).unwrap();
    assert_ne!(editor.document().to_json().unwrap(), before);
    assert_eq!(
        resources
            .find_from(&context("r", "pkg"), "Before", C::Package, LOCAL)
            .unwrap(),
        Some(object("r", "pkg"))
    );
    assert_eq!(
        resources
            .find_from(&context("r", "pkg"), "After", C::Package, LOCAL)
            .unwrap(),
        None
    );
}

#[test]
fn conjugated_port_effective_name_uses_first_conjugator_and_ignores_declared_name() {
    let resources = ResourceSet::new(vec![snapshot("r", vec![
        node("conjugated", "ConjugatedPortDefinition", json!({"declaredName":"ignored","ownedRelationship":[
            node("z-first", "PortConjugation", json!({"originalPortDefinition":"original"})),
            node("a-second", "PortConjugation", json!({"originalPortDefinition":"other"})),
        ]})),
        node("original", "PortDefinition", json!({"declaredName":"Power"})),
        node("other", "PortDefinition", json!({"declaredName":"Other"})),
    ])]).unwrap();
    assert_eq!(
        resources
            .effective_name(&object("r", "conjugated"), NameKind::Name, LOCAL)
            .unwrap(),
        Some("~Power".into())
    );
    assert_eq!(
        resources
            .find_from(
                &context("r", "original"),
                "~Power",
                C::PortDefinition,
                LOCAL
            )
            .unwrap(),
        Some(object("r", "conjugated"))
    );
}

#[cfg(feature = "raw-factory-candidate")]
#[test]
fn generated_port_local_references_feed_conjugated_effective_name_after_reload() {
    use syson_initializer_graph::initialize_port_definition;
    use syson_model::factory::RawFactory;
    use syson_model::{ContainmentFeature as F, Editor, attach_raw_selected};
    let id = |n: u32| format!("10000000-0000-4000-8000-{n:012x}");
    let mut editor = Editor::new(document(vec![package("pkg", Some("Pkg"), vec![])]));
    let mut n = 0;
    let mut factory = RawFactory::new(move || {
        n += 1;
        Ok(id(n))
    });
    let membership = factory.allocate(C::OwningMembership).unwrap();
    let root = factory.allocate(C::PortDefinition).unwrap();
    let (tx, _) = attach_raw_selected(
        editor.begin(),
        "pkg",
        F::OwnedRelationship,
        &membership,
        "20000000-0000-4000-8000-000000000001",
        &[],
    )
    .unwrap();
    let (tx, _) = attach_raw_selected(
        tx,
        membership.semantic_id(),
        F::OwnedRelatedElement,
        &root,
        "20000000-0000-4000-8000-000000000002",
        &[],
    )
    .unwrap();
    let mut resource_number = 2;
    let mut resources = move |_: C, _: &str| {
        resource_number += 1;
        Ok(format!("20000000-0000-4000-8000-{resource_number:012x}"))
    };
    let (mut tx, report) =
        initialize_port_definition(tx, root.semantic_id(), &mut factory, &mut resources, || {
            false
        })
        .unwrap();
    tx.set_attribute(root.semantic_id(), "declaredName", json!("Power"))
        .unwrap();
    editor.commit(tx).unwrap();
    let reloaded = Document::from_json(&editor.document().to_json().unwrap()).unwrap();
    let set = ResourceSet::new(vec![ResourceSnapshot::new("model", reloaded)]).unwrap();
    let conjugated = report
        .allocations
        .iter()
        .find(|a| a.classifier == C::ConjugatedPortDefinition)
        .unwrap();
    assert_eq!(
        set.effective_name(
            &object("model", &conjugated.semantic_id),
            NameKind::Name,
            LOCAL
        )
        .unwrap(),
        Some("~Power".into())
    );
    assert_eq!(
        set.find_from(
            &context("model", root.semantic_id()),
            "Pkg::Power::~Power",
            C::PortDefinition,
            LOCAL
        )
        .unwrap(),
        Some(object("model", &conjugated.semantic_id))
    );
}

#[test]
fn non_membership_relationships_flatten_for_owned_lookup_and_retain_raw_container_identity() {
    let resources = ResourceSet::new(vec![snapshot(
        "r",
        vec![package(
            "pkg",
            Some("Pkg"),
            vec![node(
                "typing",
                "FeatureTyping",
                json!({"declaredName":"relation",
            "ownedRelatedElement":[node("part", "PartUsage", json!({"declaredName":"part"}))]}),
            )],
        )],
    )])
    .unwrap();
    assert_eq!(
        resources
            .find_from(&context("r", "pkg"), "Pkg::part", C::PartUsage, LOCAL)
            .unwrap(),
        Some(object("r", "part"))
    );
    assert_eq!(
        resources
            .find_from(&context("r", "pkg"), "relation", C::Relationship, LOCAL)
            .unwrap(),
        Some(object("r", "typing"))
    );
    assert_eq!(
        resources
            .find_from(
                &context("r", "pkg"),
                "Pkg::relation",
                C::Relationship,
                LOCAL
            )
            .unwrap(),
        None
    );
    assert_eq!(
        resources.semantic_owner(&object("r", "part")).unwrap(),
        Some(object("r", "pkg"))
    );
    assert_eq!(
        resources.semantic_owner(&object("r", "typing")).unwrap(),
        None
    );
}

#[test]
fn malformed_containment_unknown_classes_and_enum_objects_are_not_silently_projected() {
    for roots in [
        vec![package(
            "pkg",
            Some("Pkg"),
            vec![node("wrong", "PartUsage", json!({}))],
        )],
        vec![node(
            "pkg",
            "Package",
            json!({"ownedRelatedElement":[node("wrong", "PartUsage", json!({}))]}),
        )],
        vec![node(
            "invalid",
            "Package",
            json!({"ownedRelatedElement":[]}),
        )],
        vec![node("unknown", "FutureClass", json!({}))],
        vec![node("enum", "FeatureDirectionKind", json!({}))],
    ] {
        assert!(ResourceSet::new(vec![snapshot("r", roots)]).is_err());
    }
    let mut foreign = node("foreign", "Package", json!({}));
    foreign["eClass"] = json!("foreign:Package");
    assert!(ResourceSet::new(vec![snapshot("r", vec![foreign])]).is_err());
    let resources = ResourceSet::new(vec![snapshot(
        "r",
        vec![package("pkg", Some("Pkg"), vec![])],
    )])
    .unwrap();
    assert!(
        resources
            .find_from(&context("r", "pkg"), "Pkg", C::FeatureDirectionKind, LOCAL)
            .is_err()
    );
}

#[test]
fn eannotation_containment_gap_is_lazy_and_never_disappears_as_known_absence() {
    let resources = ResourceSet::new(vec![snapshot(
        "r",
        vec![node(
            "pkg",
            "Package",
            json!({
                "declaredName":"Pkg", "eAnnotations":[{"source":"synthetic-metadata"}],
            }),
        )],
    )])
    .unwrap();
    // Source checks the root before obtaining eAllContents.
    assert_eq!(
        resources
            .find_from(&context("r", "pkg"), "Pkg", C::Package, LOCAL)
            .unwrap(),
        Some(object("r", "pkg"))
    );
    assert!(
        resources
            .find_from(&context("r", "pkg"), "absent", C::Package, LOCAL)
            .is_err()
    );
}

#[test]
fn successful_first_import_does_not_resolve_missing_later_observation_targets() {
    let resources = ResourceSet::new(vec![
        snapshot("local", vec![package("pkg", Some("Pkg"), vec![])]),
        snapshot(
            "remote",
            vec![node(
                "target",
                "PartUsage",
                json!({"declaredName":"target"}),
            )],
        ),
    ])
    .unwrap()
    .with_imported_memberships(
        object("local", "pkg"),
        vec![
            ImportedMembershipObservation {
                visibility: Visibility::Public,
                related: vec![object("remote", "target"), object("remote", "missing")],
            },
            ImportedMembershipObservation {
                visibility: Visibility::Public,
                related: vec![object("unregistered", "later")],
            },
        ],
    )
    .unwrap();
    assert_eq!(
        resources
            .find_from(&context("local", "pkg"), "Pkg::target", C::Feature, LOCAL)
            .unwrap(),
        Some(object("remote", "target"))
    );
    assert!(matches!(
        resources.find_from(&context("local", "pkg"), "Pkg::absent", C::Feature, LOCAL),
        Err(Error::Unresolved(_))
    ));
}

#[test]
fn naming_references_do_not_resolve_same_semantic_id_from_a_different_resource() {
    let resources = ResourceSet::new(vec![
        snapshot("local", vec![node("feature", "PartUsage", json!({"ownedRelationship":[node("redef", "Redefinition", json!({"redefinedFeature":"target"}))]}))]),
        snapshot("remote", vec![node("target", "PartUsage", json!({"declaredName":"remote-name"}))]),
    ]).unwrap();
    assert!(matches!(
        resources.effective_name(&object("local", "feature"), NameKind::Name, LOCAL),
        Err(Error::Unresolved(_))
    ));
    assert_eq!(
        resources
            .effective_name(&object("remote", "target"), NameKind::Name, LOCAL)
            .unwrap(),
        Some("remote-name".into())
    );
}
