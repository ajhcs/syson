// Copyright (c) 2022-2025 Sensmetry UAB and others.
// SPDX-License-Identifier: EPL-2.0 OR GPL-2.0 WITH Classpath-exception-2.0
use syson_reflection::{
    MandatoryKind, OrderingQualification, ReflectionDb, ReflectionError, TypeHandle,
};

fn names(db: &ReflectionDb, handles: &[TypeHandle]) -> Vec<&'static str> {
    handles.iter().map(|h| db.name(h).unwrap()).collect()
}
fn csv(value: &str) -> Vec<&str> {
    if value.is_empty() {
        Vec::new()
    } else {
        value.split(',').collect()
    }
}

#[test]
fn every_generated_name_order_ancestor_descendant_and_metadata_matches_static_source_fixture() {
    // Generated independently by a DFS closure plus CPython stable comparison
    // sort. This is source-derived candidate evidence, not an upstream capture.
    let db = ReflectionDb::pinned();
    let rows: Vec<Vec<&str>> = include_str!("source_tables.tsv")
        .lines()
        .map(|line| line.split('\t').collect())
        .collect();
    assert_eq!(rows.len(), 189);
    assert_eq!(
        names(&db, db.all_generated_types()),
        rows.iter().map(|r| r[0]).collect::<Vec<_>>()
    );
    for row in &rows {
        assert_eq!(row.len(), 4, "fixture shape for {}", row[0]);
        let ty = db.resolve_type(row[0]).unwrap();
        assert_eq!(
            names(&db, db.proper_supertypes(&ty).unwrap()),
            csv(row[1]),
            "ancestors: {}",
            row[0]
        );
        assert_eq!(
            names(&db, db.proper_subtypes(&ty).unwrap()),
            csv(row[2]),
            "descendants: {}",
            row[0]
        );
        let actual: Vec<_> = db
            .effective_mandatory(&ty)
            .unwrap()
            .iter()
            .map(|property| {
                let kind = match property.kind {
                    MandatoryKind::Array => "array",
                    MandatoryKind::Boolean => "boolean",
                };
                let declaring = property
                    .declared_by
                    .as_ref()
                    .map(|h| db.name(h).unwrap())
                    .unwrap_or("-");
                format!("{}:{kind}:{declaring}", property.name)
            })
            .collect();
        assert_eq!(actual, csv(row[3]), "mandatory precedence: {}", row[0]);
        for candidate in db.all_generated_types() {
            let parent = db.name(candidate).unwrap();
            assert_eq!(
                db.is_subtype(&ty, candidate).unwrap(),
                row[0] == parent || csv(row[1]).contains(&parent),
                "{} <: {parent}",
                row[0]
            );
        }
    }
}

#[test]
fn four_retained_strict_source_expectations_remain_literal() {
    let db = ReflectionDb::pinned();
    for (name, expected) in [
        ("Element", vec![]),
        (
            "Comment",
            vec!["TextualAnnotatingElement", "AnnotatingElement", "Element"],
        ),
        ("Feature", vec!["Type", "Namespace", "Element"]),
        (
            "OperatorExpression",
            vec![
                "InvocationExpression",
                "InlineExpression",
                "Expression",
                "Step",
                "Feature",
                "Type",
                "Namespace",
                "Element",
            ],
        ),
    ] {
        let ty = db.resolve_type(name).unwrap();
        assert_eq!(
            names(&db, db.proper_supertypes(&ty).unwrap()),
            expected,
            "{name}"
        );
    }
}

#[test]
fn non_type_markers_are_filtered_only_from_the_ast_list() {
    let db = ReflectionDb::pinned();
    let all = names(&db, db.all_generated_types());
    let ast = names(&db, db.ast_types_filtered());
    assert_eq!((all.len(), ast.len()), (189, 184));
    for marker in [
        "TransparentElement",
        "FeatureRelationship",
        "InlineExpression",
        "NonOwnerType",
        "TypeRelationship",
    ] {
        assert!(all.contains(&marker));
        assert!(!ast.contains(&marker));
        let ty = db.resolve_type(marker).unwrap();
        assert!(db.is_subtype(&ty, &ty).unwrap());
    }
    let operator = db.resolve_type("OperatorExpression").unwrap();
    let inline = db.resolve_type("InlineExpression").unwrap();
    assert!(db.is_subtype(&operator, &inline).unwrap());
    assert!(db.proper_supertypes(&operator).unwrap().contains(&inline));
}

#[test]
fn local_metadata_precedes_inherited_and_children_is_appended_once() {
    let db = ReflectionDb::pinned();
    let state = db.resolve_type("StateUsage").unwrap();
    let properties = db.effective_mandatory(&state).unwrap();
    assert_eq!(properties[0].name, "isParallel");
    assert_eq!(properties[0].kind, MandatoryKind::Boolean);
    assert_eq!(properties[0].declared_by, Some(state));
    assert_eq!(properties.last().unwrap().name, "$children");
    assert_eq!(properties.last().unwrap().declared_by, None);
    for property in properties {
        assert_eq!(
            properties
                .iter()
                .filter(|p| p.name == property.name)
                .count(),
            1
        );
    }
    let reference = db.resolve_type("TypeReference").unwrap();
    let parts = db
        .effective_mandatory(&reference)
        .unwrap()
        .iter()
        .find(|p| p.name == "parts")
        .unwrap();
    assert_eq!(
        db.name(parts.declared_by.as_ref().unwrap()).unwrap(),
        "ElementReference"
    );
}

#[test]
fn generated_property_ids_and_override_reference_positions_are_distinct() {
    let db = ReflectionDb::pinned();
    let targets = [
        ("ElementReference", "Element"),
        ("TypeReference", "Type"),
        ("ClassifierReference", "Classifier"),
        ("MetaclassReference", "Metaclass"),
        ("MembershipReference", "Membership"),
        ("NamespaceReference", "Namespace"),
        ("FeatureReference", "Feature"),
        ("ConjugatedPortReference", "ConjugatedPortDefinition"),
    ];
    for (reference, target) in targets {
        let container = db.resolve_type(reference).unwrap();
        let generated = db
            .generated_reference_target(&format!("{reference}:parts"))
            .unwrap();
        assert_eq!(db.name(&generated).unwrap(), "Element");
        for length in 1..=4 {
            for index in 0..length {
                let position = db.reference_position(&container, index, length).unwrap();
                let resolved = db.reference_target_type(&position).unwrap();
                assert_eq!(
                    db.name(&resolved).unwrap(),
                    if index == length - 1 {
                        target
                    } else {
                        "Element"
                    }
                );
            }
        }
    }
    let package = db.resolve_type("Package").unwrap();
    let position = db.reference_position(&package, 0, 1).unwrap();
    assert_eq!(
        db.name(&db.reference_target_type(&position).unwrap())
            .unwrap(),
        "Element"
    );
    for unknown in [
        "",
        "TypeReference:other",
        "Package:parts",
        "__proto__:parts",
    ] {
        assert_eq!(
            db.generated_reference_target(unknown),
            Err(ReflectionError::UnknownReferenceProperty(unknown.into()))
        );
    }
}

#[test]
fn malformed_positions_are_rejected_without_arithmetic_overflow() {
    let db = ReflectionDb::pinned();
    let ty = db.resolve_type("TypeReference").unwrap();
    for (index, length) in [(0, 0), (1, 1), (usize::MAX, 1), (usize::MAX, usize::MAX)] {
        assert!(matches!(
            db.reference_position(&ty, index, length),
            Err(ReflectionError::InvalidReferencePosition)
        ));
    }
    let position = db
        .reference_position(&ty, usize::MAX - 1, usize::MAX)
        .unwrap();
    assert_eq!(
        db.name(&db.reference_target_type(&position).unwrap())
            .unwrap(),
        "Type"
    );
}

#[test]
fn unknown_names_and_foreign_handles_are_explicit_errors() {
    let db = ReflectionDb::pinned();
    for name in ["", "NotAType", "__proto__", "Type\0", "type"] {
        assert_eq!(
            db.resolve_type(name),
            Err(ReflectionError::UnknownType(name.into()))
        );
    }
    let other = ReflectionDb::pinned();
    let foreign = other.resolve_type("Type").unwrap();
    let local = db.resolve_type("Type").unwrap();
    assert_ne!(local, foreign);
    assert_eq!(db.name(&foreign), Err(ReflectionError::ForeignTable));
    assert_eq!(
        db.proper_supertypes(&foreign),
        Err(ReflectionError::ForeignTable)
    );
    assert_eq!(
        db.proper_subtypes(&foreign),
        Err(ReflectionError::ForeignTable)
    );
    assert_eq!(
        db.effective_mandatory(&foreign),
        Err(ReflectionError::ForeignTable)
    );
    assert_eq!(
        db.is_subtype(&local, &foreign),
        Err(ReflectionError::ForeignTable)
    );
    assert_eq!(
        db.is_subtype(&foreign, &local),
        Err(ReflectionError::ForeignTable)
    );
    assert!(matches!(
        db.reference_position(&foreign, 0, 1),
        Err(ReflectionError::ForeignTable)
    ));
    let position = other.reference_position(&foreign, 0, 1).unwrap();
    assert_eq!(
        db.reference_target_type(&position),
        Err(ReflectionError::ForeignTable)
    );
}

#[test]
fn owner_identity_outlives_its_database_and_tables_are_shareable() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<ReflectionDb>();
    send_sync::<TypeHandle>();
    let old = {
        let db = ReflectionDb::pinned();
        db.resolve_type("Feature").unwrap()
    };
    let fresh = ReflectionDb::pinned();
    assert_eq!(fresh.name(&old), Err(ReflectionError::ForeignTable));
    assert_eq!(
        fresh.ordering_qualification(),
        OrderingQualification::SourceDerivedStableShortSortCandidate
    );
    assert_eq!(ReflectionDb::MAX_PINNED_SORT_PARTITION, 13);
}

#[test]
fn three_retained_source_subtype_examples_exclude_self_and_keep_generated_order() {
    // Literal names from the pinned type-index.test.ts, independently of the
    // generated source_tables.tsv fixture. The upstream examples check set
    // membership; this immutable API also preserves the outer generated order.
    let db = ReflectionDb::pinned();
    for (name, expected) in [
        (
            "AnnotatingElement",
            vec![
                "Comment",
                "Documentation",
                "MetadataFeature",
                "MetadataUsage",
                "TextualAnnotatingElement",
                "TextualRepresentation",
            ],
        ),
        ("Multiplicity", vec!["MultiplicityRange"]),
        (
            "Subsetting",
            vec!["CrossSubsetting", "Redefinition", "ReferenceSubsetting"],
        ),
    ] {
        let ty = db.resolve_type(name).unwrap();
        assert_eq!(names(&db, db.proper_subtypes(&ty).unwrap()), expected);
        assert!(!db.proper_subtypes(&ty).unwrap().contains(&ty));
        assert!(db.is_subtype(&ty, &ty).unwrap());
    }
}

#[test]
fn mixed_namespace_partition_has_nontransitive_incomparability() {
    // These are all real ancestors in AllocationUsage's namespace partition.
    // A subtype comparator treats the two incomparable pairs as equal, but
    // ConnectorAsUsage < Connector: equivalence is therefore not transitive.
    // This preserves the reason a generic total-order sort is inappropriate;
    // it does not establish which JavaScript-runtime ordering is required.
    let db = ReflectionDb::pinned();
    let allocation = db.resolve_type("AllocationUsage").unwrap();
    let namespace = db.resolve_type("Namespace").unwrap();
    let connector_usage = db.resolve_type("ConnectorAsUsage").unwrap();
    let part_usage = db.resolve_type("PartUsage").unwrap();
    let connector = db.resolve_type("Connector").unwrap();
    for ty in [&connector_usage, &part_usage, &connector] {
        assert!(db.proper_supertypes(&allocation).unwrap().contains(ty));
        assert!(db.is_subtype(ty, &namespace).unwrap());
    }
    for (left, right) in [(&connector_usage, &part_usage), (&part_usage, &connector)] {
        assert!(!db.is_subtype(left, right).unwrap());
        assert!(!db.is_subtype(right, left).unwrap());
    }
    assert!(db.is_subtype(&connector_usage, &connector).unwrap());
    assert!(!db.is_subtype(&connector, &connector_usage).unwrap());
}
