// Copyright (c) 2024, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
// Source fixtures: pinned sysml.ecore and sysml-customnodes.ecore.
use ClassifierId::*;
use syson_metamodel::Package as SchemaPackage;
use syson_metamodel::*;
#[test]
fn complete_domain_and_separate_custom_package_counts() {
    assert_eq!(DOMAIN_CLASSIFIERS.len(), 182);
    assert_eq!(
        DOMAIN_CLASSIFIERS
            .iter()
            .filter(|c| c.kind == ClassifierKind::Class)
            .count(),
        175
    );
    assert_eq!(
        DOMAIN_CLASSIFIERS
            .iter()
            .filter(|c| c.kind == ClassifierKind::Enum)
            .count(),
        7
    );
    assert_eq!(
        DOMAIN_CLASSIFIERS
            .iter()
            .filter(|c| c.is_abstract())
            .count(),
        8
    );
    assert_eq!(CUSTOM_NODE_CLASSIFIERS.len(), 4);
    assert_eq!(
        DOMAIN_CLASSIFIERS.iter().flat_map(|c| c.features()).count(),
        415
    );
    assert_eq!(
        DOMAIN_CLASSIFIERS
            .iter()
            .flat_map(|c| c.operations())
            .count(),
        70
    );
    assert_eq!(
        DOMAIN_CLASSIFIERS
            .iter()
            .flat_map(|c| c.operations())
            .flat_map(|o| o.parameters())
            .count(),
        57
    );
    assert_eq!(
        DOMAIN_CLASSIFIERS
            .iter()
            .flat_map(|c| c.enum_literals())
            .count(),
        19
    );
}
#[test]
fn exact_multi_level_part_usage_inheritance_is_transitive() {
    assert_eq!(
        PartUsage
            .metadata()
            .supertypes()
            .map(TypeReference::local)
            .collect::<Vec<_>>(),
        vec![Some(ItemUsage)]
    );
    for parent in [
        PartUsage,
        ItemUsage,
        OccurrenceUsage,
        Usage,
        Feature,
        Type,
        Namespace,
        Element,
    ] {
        assert!(is_subtype(PartUsage, parent), "{parent:?}");
    }
    assert!(!is_subtype(PartUsage, PartDefinition));
    assert!(!is_subtype(PartDefinition, Feature));
    assert!(!is_subtype(FeatureDirectionKind, FeatureDirectionKind));
}
#[test]
fn multiple_inheritance_and_diamond_paths_are_preserved() {
    let supers: Vec<_> = ActionDefinition
        .metadata()
        .supertypes()
        .filter_map(TypeReference::local)
        .collect();
    assert_eq!(supers, vec![OccurrenceDefinition, Behavior]);
    assert!(is_subtype(ActionDefinition, Definition));
    assert!(is_subtype(ActionDefinition, Behavior));
    let inherited = inherited_feature(ActionDefinition, "declaredName");
    assert_eq!(inherited.candidates.len(), 1);
    assert_eq!(inherited.unique().unwrap().owner, Element);
}
#[test]
fn all_local_type_references_resolve_without_external_conflation() {
    for classifier in all() {
        for supertype in classifier.supertypes() {
            assert!(!matches!(supertype, TypeReference::UnresolvedLocal { .. }));
        }
        for typed in classifier
            .features()
            .map(|f| f.typed())
            .chain(classifier.operations().map(|o| o.typed()))
        {
            assert!(!matches!(
                typed.type_reference(),
                Some(TypeReference::UnresolvedLocal { .. })
            ));
        }
    }
    assert_eq!(
        lookup(SchemaPackage::Domain, "SysMLPackageNodeStyleDescription"),
        None
    );
    assert_eq!(lookup(SchemaPackage::CustomNodes, "PartUsage"), None);
}
#[test]
fn external_emf_ancestor_and_sirius_identities_remain_explicit() {
    let ancestors = unresolved_supertypes(PartUsage);
    assert_eq!(
        ancestors,
        vec![TypeReference::External {
            raw: "platform:/plugin/org.eclipse.emf.ecore/model/Ecore.ecore#//EModelElement"
        }]
    );
    let custom = SysMLPackageNodeStyleDescription.metadata();
    assert_eq!(
        custom.supertypes().next().unwrap().raw(),
        "../../../../../sirius-components-view-diagram/src/main/resources/model/diagram.ecore#//NodeStyleDescription"
    );
    assert!(!is_subtype(custom.id, Element));
    let background = custom.features().next().unwrap();
    assert_eq!(
        background.typed().type_reference(),
        Some(TypeReference::External {
            raw: "ecore:EClass ../../../../../sirius-components-view/src/main/resources/model/view.ecore#//UserColor"
        })
    );
    let absent = inherited_feature(custom.id, "label");
    assert!(absent.unique().is_none());
    assert_eq!(absent.unresolved_supertypes.len(), 1);
}
#[test]
fn inherited_defaults_distinguish_schema_from_initializer_behavior() {
    let composite = inherited_feature(PartUsage, "isComposite")
        .unique()
        .unwrap();
    assert_eq!(composite.owner, Feature);
    // Ecore says false; ElementInitializerSwitch.caseUsage sets true at creation.
    assert_eq!(composite.explicit_default_literal(), Some("false"));
    let unique = inherited_feature(PartUsage, "isUnique").unique().unwrap();
    assert_eq!(unique.explicit_default_literal(), Some("true"));
    let name = inherited_feature(PartUsage, "declaredName")
        .unique()
        .unwrap();
    assert_eq!(name.explicit_default_literal(), None);
    assert!(name.changeable());
    assert!(!name.derived());
    assert_eq!(
        name.typed().type_reference(),
        Some(TypeReference::External {
            raw: "ecore:EDataType http://www.eclipse.org/emf/2002/Ecore#//EString"
        })
    );
}
#[test]
fn containment_opposites_and_multiplicity_are_not_erased() {
    let owned = inherited_feature(Element, "ownedRelationship")
        .unique()
        .unwrap();
    assert_eq!(owned.kind(), FeatureKind::Reference);
    assert!(owned.containment());
    assert_eq!(
        owned.opposite(),
        Some("#//Relationship/owningRelatedElement")
    );
    assert_eq!(owned.typed().lower_bound(), 0);
    assert_eq!(owned.typed().upper_bound(), -1);
    let parent = inherited_feature(Element, "owner").unique().unwrap();
    assert!(!parent.changeable());
    assert!(parent.derived());
    assert!(parent.volatile());
    assert!(parent.transient());
    let direction = inherited_feature(Feature, "direction").unique().unwrap();
    assert!(direction.unsettable());
}
#[test]
fn enums_preserve_declared_order_values_and_implicit_zero() {
    let literals: Vec<_> = VisibilityKind.metadata().enum_literals().collect();
    assert_eq!(
        literals,
        vec![
            EnumLiteral {
                name: "private",
                value: 0,
                literal: "private"
            },
            EnumLiteral {
                name: "protected",
                value: 1,
                literal: "protected"
            },
            EnumLiteral {
                name: "public",
                value: 2,
                literal: "public"
            }
        ]
    );
    assert_eq!(
        FeatureDirectionKind
            .metadata()
            .enum_literals()
            .next()
            .unwrap()
            .name,
        "in"
    );
    assert!(Element.metadata().is_abstract());
    assert!(!PartUsage.metadata().is_abstract());
}
#[test]
fn operation_parameters_and_return_metadata_are_preserved_without_execution() {
    let argument = ActionUsage
        .metadata()
        .operations()
        .find(|o| o.name() == "argument")
        .unwrap();
    assert_eq!(
        argument.typed().type_reference().unwrap().local(),
        Some(Expression)
    );
    assert_eq!(argument.typed().lower_bound(), 0);
    assert_eq!(argument.typed().upper_bound(), 1);
    let params: Vec<_> = argument.parameters().collect();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name(), "i");
    assert_eq!(params[0].lower_bound(), 1);
    assert!(!params[0].ordered());
    assert_eq!(
        params[0].type_reference(),
        Some(TypeReference::External {
            raw: "ecore:EDataType http://www.eclipse.org/emf/2002/Ecore#//EInt"
        })
    );
}
#[test]
fn redefines_and_subsets_annotations_retain_every_reference() {
    let feature = ActionUsage
        .metadata()
        .features()
        .find(|f| f.name() == "actionDefinition")
        .unwrap();
    let annotation = feature.node.annotations().next().unwrap();
    assert_eq!(annotation.attribute("source"), Some("redefines"));
    assert_eq!(
        annotation.attribute("references"),
        Some("#//Step/behavior #//OccurrenceUsage/occurrenceDefinition")
    );
    let annotations = DOMAIN_CLASSIFIERS
        .iter()
        .flat_map(|c| c.node.children.iter())
        .flat_map(|n| n.annotations())
        .count();
    assert_eq!(annotations, 309);
    let payload = AcceptActionUsage
        .metadata()
        .features()
        .find(|f| f.name() == "payloadParameter")
        .unwrap();
    assert_eq!(
        payload
            .node
            .annotations()
            .next()
            .unwrap()
            .attribute("references"),
        Some("#//Usage/nestedReference #//Step/parameter")
    );
}
#[test]
fn source_provenance_pins_both_ecore_blobs() {
    assert_eq!(SOURCES.len(), 2);
    for source in SOURCES {
        assert_eq!(source.revision, "6ba2ac64e6b16ee2b05752ad55395eebb87d4910");
        assert_eq!(source.sha256.len(), 64);
        assert!(source.attributes.iter().any(|(key, _)| *key == "nsURI"));
    }
}
