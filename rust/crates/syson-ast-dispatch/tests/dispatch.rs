// Copyright (c) 2022-2025 Sensmetry UAB and others.
// SPDX-License-Identifier: EPL-2.0 OR GPL-2.0 WITH Classpath-exception-2.0
use std::sync::Arc;
use syson_ast_dispatch::{
    ChainOrder, ConstructInput, DispatchError, Entry, Falsey, Postprocessor, Registry,
    parser_registry,
};
use syson_reflection::{OrderingQualification, ReflectionDb, ReflectionError, TypeHandle};
fn db() -> Arc<ReflectionDb> {
    Arc::new(ReflectionDb::pinned())
}
fn ty(db: &ReflectionDb, name: &str) -> TypeHandle {
    db.resolve_type(name).unwrap()
}
fn entries<T>(db: &ReflectionDb, values: Vec<(&str, T)>) -> Vec<(TypeHandle, Entry<T>)> {
    values
        .into_iter()
        .map(|(name, value)| (ty(db, name), Entry::Truthy(value)))
        .collect()
}
#[test]
fn retained_source_most_derived_expectations() {
    let db = db();
    let r = Registry::new(
        db.clone(),
        entries(
            &db,
            vec![
                ("Type", "Type"),
                ("Element", "Element"),
                ("Definition", "Definition"),
            ],
        ),
        None,
    )
    .unwrap();
    // type-index.test.ts:83-99: exact retained source assertions, not JS execution.
    for (name, expected) in [
        ("Element", "Element"),
        ("Namespace", "Element"),
        ("Type", "Type"),
        ("Feature", "Type"),
        ("Definition", "Definition"),
        ("ConnectionDefinition", "Definition"),
    ] {
        assert_eq!(r.most_derived(&ty(&db, name)).unwrap(), Some(&expected));
    }
}
#[test]
fn retained_source_array_expectations() {
    let db = db();
    let r = Registry::new(
        db.clone(),
        entries(
            &db,
            vec![("Type", vec!["Type"]), ("Element", vec!["Element"])],
        ),
        None,
    )
    .unwrap();
    assert_eq!(
        r.merge_blocks(&ty(&db, "Element"), false).unwrap(),
        vec![&"Element"]
    );
    assert_eq!(
        r.merge_blocks(&ty(&db, "Namespace"), false).unwrap(),
        vec![&"Element"]
    );
    for name in ["Type", "Feature"] {
        let result = r.merge_blocks(&ty(&db, name), false).unwrap();
        assert!(result.contains(&&"Type") && result.contains(&&"Element"));
    }
}
#[test]
fn explicit_falsey_direct_values_fall_back_to_ancestor() {
    for value in [
        Falsey::Undefined,
        Falsey::Null,
        Falsey::False,
        Falsey::Zero,
        Falsey::NaN,
        Falsey::EmptyString,
    ] {
        let db = db();
        let r = Registry::new(
            db.clone(),
            vec![
                (ty(&db, "Feature"), Entry::Falsey(value)),
                (ty(&db, "Element"), Entry::Truthy("ancestor")),
            ],
            Some(Entry::Truthy("default")),
        )
        .unwrap();
        assert_eq!(
            r.most_derived(&ty(&db, "Feature")).unwrap(),
            Some(&"ancestor")
        );
    }
}
#[test]
fn falsey_ancestors_skip_to_default_and_falsey_defaults_are_omitted() {
    let db = db();
    let values = vec![(ty(&db, "Type"), Entry::Falsey(Falsey::False))];
    let r = Registry::new(db.clone(), values.clone(), Some(Entry::Truthy("fallback"))).unwrap();
    assert_eq!(
        r.most_derived(&ty(&db, "Feature")).unwrap(),
        Some(&"fallback")
    );
    let r = Registry::<&str>::new(db.clone(), values, Some(Entry::Falsey(Falsey::EmptyString)))
        .unwrap();
    assert_eq!(r.most_derived(&ty(&db, "Feature")).unwrap(), None);
    assert!(r.expand_most_derived().unwrap().is_empty());
    assert!(
        r.chain(&ty(&db, "Feature"), ChainOrder::SubtypeFirst)
            .unwrap()
            .is_empty()
    );
}
#[test]
fn truthy_default_expands_to_every_generated_name_in_source_order() {
    let db = db();
    let r = Registry::new(db.clone(), vec![], Some(Entry::Truthy("default"))).unwrap();
    let result = r.expand_most_derived().unwrap();
    assert_eq!(result.len(), 189);
    for ((actual, value), expected) in result.iter().zip(db.all_generated_types()) {
        assert_eq!(actual, expected);
        assert_eq!(**value, "default");
    }
    assert_eq!(
        r.ordering_qualification(),
        OrderingQualification::SourceDerivedStableShortSortCandidate
    );
}
#[test]
fn empty_array_is_truthy_but_merge_still_collects_ancestor_block() {
    let db = db();
    let r = Registry::new(
        db.clone(),
        entries(
            &db,
            vec![
                ("Feature", Vec::<&str>::new()),
                ("Element", vec!["ancestor"]),
            ],
        ),
        Some(Entry::Truthy(vec!["ignored-default"])),
    )
    .unwrap();
    assert_eq!(
        r.most_derived(&ty(&db, "Feature")).unwrap(),
        Some(&Vec::new())
    );
    assert_eq!(
        r.merge_blocks(&ty(&db, "Feature"), false).unwrap(),
        vec![&"ancestor"]
    );
}
#[test]
fn reverse_merge_reverses_blocks_without_reversing_elements() {
    let db = db();
    let r = Registry::new(
        db.clone(),
        entries(
            &db,
            vec![
                ("Feature", vec!["f1", "f2"]),
                ("Type", vec!["t1", "t2"]),
                ("Element", vec!["e1", "e2"]),
            ],
        ),
        Some(Entry::Truthy(vec!["ignored"])),
    )
    .unwrap();
    assert_eq!(
        r.merge_blocks(&ty(&db, "Feature"), false).unwrap(),
        vec![&"f1", &"f2", &"t1", &"t2", &"e1", &"e2"]
    );
    assert_eq!(
        r.merge_blocks(&ty(&db, "Feature"), true).unwrap(),
        vec![&"e1", &"e2", &"t1", &"t2", &"f1", &"f2"]
    );
}
#[test]
fn full_chain_default_is_last_or_first_and_payloads_are_not_flattened() {
    let db = db();
    let r = Registry::new(
        db.clone(),
        entries(
            &db,
            vec![
                ("Feature", vec![1, 2]),
                ("Type", vec![3, 4]),
                ("Element", vec![5, 6]),
            ],
        ),
        Some(Entry::Truthy(vec![7, 8])),
    )
    .unwrap();
    assert_eq!(
        r.chain(&ty(&db, "Feature"), ChainOrder::SubtypeFirst)
            .unwrap(),
        vec![&vec![1, 2], &vec![3, 4], &vec![5, 6], &vec![7, 8]]
    );
    assert_eq!(
        r.chain(&ty(&db, "Feature"), ChainOrder::SupertypeFirst)
            .unwrap(),
        vec![&vec![7, 8], &vec![5, 6], &vec![3, 4], &vec![1, 2]]
    );
}
#[test]
fn repeated_payload_identity_is_retained_for_distinct_ancestors_and_default() {
    let db = db();
    let shared = Arc::new("same processor");
    let r = Registry::new(
        db.clone(),
        entries(
            &db,
            vec![
                ("Type", shared.clone()),
                ("Namespace", shared.clone()),
                ("Element", shared.clone()),
            ],
        ),
        Some(Entry::Truthy(shared.clone())),
    )
    .unwrap();
    let result = r
        .chain(&ty(&db, "OperatorExpression"), ChainOrder::SubtypeFirst)
        .unwrap();
    assert_eq!(result.len(), 4);
    assert!(result.into_iter().all(|p| Arc::ptr_eq(p, &shared)));
}
#[test]
fn foreign_owner_is_rejected_at_registration_and_every_lookup() {
    let a = db();
    let b = db();
    let foreign = ty(&b, "Feature");
    let err = DispatchError::Reflection(ReflectionError::ForeignTable);
    assert_eq!(
        Registry::new(a.clone(), vec![(foreign.clone(), Entry::Truthy(1))], None).unwrap_err(),
        err
    );
    let r = Registry::new(a.clone(), entries(&a, vec![("Element", vec![1])]), None).unwrap();
    assert_eq!(r.most_derived(&foreign), Err(err.clone()));
    assert_eq!(
        r.chain(&foreign, ChainOrder::SubtypeFirst),
        Err(err.clone())
    );
    assert_eq!(r.merge_blocks(&foreign, true), Err(err));
}
#[test]
fn duplicate_registration_is_restricted_but_default_is_not_a_type() {
    let db = db();
    assert_eq!(
        Registry::new(
            db.clone(),
            entries(&db, vec![("Element", 1), ("Element", 2)]),
            None
        )
        .unwrap_err(),
        DispatchError::DuplicateRegistration
    );
    assert_eq!(
        db.resolve_type("default"),
        Err(ReflectionError::UnknownType("default".into()))
    );
}
#[test]
fn parser_registry_covers_direct_and_inherited_processors_without_invoking_them() {
    let db = db();
    let r = parser_registry(db).unwrap();
    for (name, processor) in [
        ("OperatorExpression", Postprocessor::FixOperatorExpression),
        ("WhileLoopActionUsage", Postprocessor::AddLoopMember),
        ("Import", Postprocessor::FinalizeImport),
        ("NamespaceImport", Postprocessor::FinalizeImport),
        ("MembershipImport", Postprocessor::FinalizeImport),
        (
            "TransitionUsage",
            Postprocessor::CreateEmptyTransitionParameters,
        ),
        (
            "SuccessionAsUsage",
            Postprocessor::CreateMissingSuccessionEnds,
        ),
    ] {
        assert_eq!(
            r.construct_plan(ConstructInput::String(name)).unwrap(),
            Some(&processor)
        );
    }
    assert_eq!(
        r.construct_plan(ConstructInput::String("Element")).unwrap(),
        None
    );
}
#[test]
fn construct_skips_nonstrings_and_unknown_strings_even_with_default() {
    let db = db();
    let r = Registry::new(db, vec![], Some(Entry::Truthy(9))).unwrap();
    for input in [
        ConstructInput::Symbol,
        ConstructInput::Undefined,
        ConstructInput::String("default"),
        ConstructInput::String("not-an-AST-type"),
    ] {
        assert_eq!(r.construct_plan(input).unwrap(), None);
    }
    assert_eq!(
        r.construct_plan(ConstructInput::String("Feature")).unwrap(),
        Some(&9)
    );
}
#[test]
fn registries_and_prior_borrowed_plans_do_not_change_each_other() {
    let db = db();
    let first = Registry::new(db.clone(), entries(&db, vec![("Element", 1)]), None).unwrap();
    let before = first.most_derived(&ty(&db, "Feature")).unwrap();
    let second = Registry::new(db.clone(), entries(&db, vec![("Element", 2)]), None).unwrap();
    assert_eq!(before, Some(&1));
    assert_eq!(second.most_derived(&ty(&db, "Feature")).unwrap(), Some(&2));
}

#[test]
fn merged_blocks_retain_repeated_items_within_and_across_ancestors() {
    let db = db();
    let r = Registry::new(
        db.clone(),
        entries(&db, vec![("Feature", vec![1, 1]), ("Type", vec![1])]),
        Some(Entry::Truthy(vec![99])),
    )
    .unwrap();
    assert_eq!(
        r.merge_blocks(&ty(&db, "Feature"), false).unwrap(),
        vec![&1, &1, &1]
    );
    assert_eq!(
        r.merge_blocks(&ty(&db, "Feature"), true).unwrap(),
        vec![&1, &1, &1]
    );
}
