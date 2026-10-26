// SPDX-License-Identifier: EPL-2.0
use std::ptr;
use syson_document_reconcile::*;

fn doc(id: u8, name: &str, content: &str) -> Document<u64> {
    Document {
        id: [id; 16],
        name: name.into(),
        content: content.into(),
        is_read_only: true,
        created_on: 10,
        last_modified_on: 20,
    }
}
fn domains() -> Vec<String> {
    vec!["new-domain".into(), "new-domain".into()]
}
fn replaced<'a, T>(
    plan: DocumentPlan<'a, T>,
) -> (
    Vec<DocumentAction<'a, T>>,
    std::collections::BTreeSet<&'a str>,
) {
    match plan {
        DocumentPlan::Replace {
            documents,
            domain_uris,
        } => (documents, domain_uris),
        DocumentPlan::Unchanged => panic!("expected replacement"),
    }
}

#[test]
fn unchanged_reordering_flag_timestamps_and_domains_do_not_replace() {
    let a = doc(1, "A", "a");
    let b = doc(2, "B", "b");
    let mut incoming_a = doc(1, "A", "a");
    incoming_a.is_read_only = false;
    incoming_a.created_on = 100;
    incoming_a.last_modified_on = 200;
    let d = domains();
    let plan = plan_documents(&[&a, &b], &[&b, &incoming_a], &d);
    assert!(matches!(plan, DocumentPlan::Unchanged));
    assert!(plan.effects().is_empty());
}
#[test]
fn rename_rebuilds_resets_readonly_and_requests_fresh_timestamps() {
    let a = doc(1, "old", "body");
    let incoming = doc(1, "new", "body");
    let d = domains();
    let plan = plan_documents(&[&a], &[&incoming], &d);
    assert_eq!(
        plan.effects(),
        vec![
            Effect::RebuiltDocumentTimestamp { output_index: 0 },
            Effect::AggregateTimestamp,
            Effect::UpdatedEvent
        ]
    );
    let (actions, ds) = replaced(plan);
    assert_eq!(ds.into_iter().collect::<Vec<_>>(), vec!["new-domain"]);
    assert!(
        matches!(actions.as_slice(), [DocumentAction::Rebuild { id, name: "new", content: "body", is_read_only: false }] if *id == [1;16])
    );
    assert!(a.is_read_only);
    assert_eq!(a.last_modified_on, 20);
}
#[test]
fn content_change_is_not_ignored() {
    let a = doc(1, "a", "one");
    let b = doc(1, "a", "two");
    let (actions, _) = replaced(plan_documents(&[&a], &[&b], &[]));
    assert!(matches!(
        actions.as_slice(),
        [DocumentAction::Rebuild { content: "two", .. }]
    ));
}
#[test]
fn unchanged_existing_and_new_incoming_retain_instances_and_timestamps() {
    let a = doc(1, "A", "a");
    let a_copy = doc(1, "A", "a");
    let b = doc(2, "B", "b");
    let plan = plan_documents(&[&a], &[&a_copy, &b], &[]);
    assert_eq!(
        plan.effects(),
        vec![Effect::AggregateTimestamp, Effect::UpdatedEvent]
    );
    let (actions, _) = replaced(plan);
    assert!(
        matches!(&actions[0], DocumentAction::Keep { document, origin: Origin::Existing } if ptr::eq(*document, &a))
    );
    assert!(
        matches!(&actions[1], DocumentAction::Keep { document, origin: Origin::Incoming } if ptr::eq(*document, &b))
    );
    assert_eq!(b.created_on, 10);
}
#[test]
fn id_set_detects_removal_and_empty_collections() {
    let a = doc(1, "A", "a");
    let b = doc(2, "B", "b");
    let (actions, _) = replaced(plan_documents(&[&a, &b], &[&b], &[]));
    assert!(
        matches!(actions.as_slice(), [DocumentAction::Keep { document, .. }] if ptr::eq(*document, &b))
    );
    let (actions, _) = replaced(plan_documents(&[&a], &[], &[]));
    assert!(actions.is_empty());
    assert!(matches!(
        plan_documents::<u64>(&[], &[], &[]),
        DocumentPlan::Unchanged
    ));
}
#[test]
fn distinct_same_uuid_inputs_are_not_deduplicated_by_values() {
    let a = doc(1, "A", "a");
    let b = doc(1, "A", "a");
    let (actions, _) = replaced(plan_documents(&[], &[&a, &b, &a], &[]));
    assert_eq!(actions.len(), 2);
    assert!(matches!(&actions[0], DocumentAction::Keep { document, .. } if ptr::eq(*document, &a)));
    assert!(matches!(&actions[1], DocumentAction::Keep { document, .. } if ptr::eq(*document, &b)));
}
#[test]
fn duplicate_ids_consult_first_existing_and_can_leave_old_duplicates_untouched() {
    let first = doc(1, "first", "x");
    let second = doc(1, "second", "y");
    let incoming = doc(1, "first", "x");
    assert!(matches!(
        plan_documents(&[&first, &second], &[&incoming], &[]),
        DocumentPlan::Unchanged
    ));
    let (actions, _) = replaced(plan_documents(&[&second, &first], &[&incoming], &[]));
    assert!(matches!(
        actions.as_slice(),
        [DocumentAction::Rebuild {
            name: "first",
            content: "x",
            ..
        }]
    ));
}
#[test]
fn repeated_reuse_deduplicates_but_rebuilds_remain_distinct() {
    let old = doc(1, "old", "x");
    let same_a = doc(1, "old", "x");
    let same_b = doc(1, "old", "x");
    let changed_a = doc(1, "new", "x");
    let changed_b = doc(1, "new", "x");
    let plan = plan_documents(&[&old], &[&same_a, &changed_a, &same_b, &changed_b], &[]);
    assert_eq!(
        plan.effects(),
        vec![
            Effect::RebuiltDocumentTimestamp { output_index: 1 },
            Effect::RebuiltDocumentTimestamp { output_index: 2 },
            Effect::AggregateTimestamp,
            Effect::UpdatedEvent
        ]
    );
    let (actions, _) = replaced(plan);
    assert_eq!(actions.len(), 3);
    assert!(
        matches!(&actions[0], DocumentAction::Keep { document, .. } if ptr::eq(*document, &old))
    );
}
#[test]
fn empty_nonnull_names_and_content_are_valid_builder_values() {
    let a = doc(0, "", "");
    let requested_domains = vec![String::new()];
    let (actions, domain_uris) = replaced(plan_documents(&[], &[&a], &requested_domains));
    assert_eq!(domain_uris.into_iter().collect::<Vec<_>>(), vec![""]);
    assert!(
        matches!(actions.as_slice(), [DocumentAction::Keep { document, .. }] if ptr::eq(*document, &a))
    );
}

#[derive(Debug)]
struct Ref {
    id: Uuid,
    wrapper: u8,
}
#[test]
fn adding_filters_only_against_old_ids_and_retains_batch_duplicates() {
    let old = Ref {
        id: [1; 16],
        wrapper: 0,
    };
    let equivalent = Ref {
        id: [1; 16],
        wrapper: 1,
    };
    let new_a = Ref {
        id: [2; 16],
        wrapper: 2,
    };
    let new_b = Ref {
        id: [2; 16],
        wrapper: 3,
    };
    let plan = plan_add_dependencies(
        &[&old],
        &[&equivalent, &new_a, &new_b, &new_a],
        |r| r.id,
        MutationAdmission::Allowed,
    )
    .unwrap();
    assert_eq!(plan.references.len(), 4);
    assert!(ptr::eq(plan.references[0], &old));
    assert!(ptr::eq(plan.references[1], &new_a));
    assert!(ptr::eq(plan.references[2], &new_b));
    assert!(ptr::eq(plan.references[3], &new_a));
    assert_eq!(
        plan.effects(),
        vec![Effect::AggregateTimestamp, Effect::UpdatedEvent]
    );
}
#[test]
fn adding_nothing_does_not_emit_but_admitted_noop_removal_requests_event() {
    let old = Ref {
        id: [1; 16],
        wrapper: 0,
    };
    let add =
        plan_add_dependencies(&[&old], &[&old], |r| r.id, MutationAdmission::Unresolved).unwrap();
    assert!(!add.request_updated_event);
    assert!(add.effects().is_empty());
    let remove = plan_remove_dependencies(
        &[&old],
        &[],
        |a, b| a.wrapper == b.wrapper,
        MutationAdmission::Allowed,
    )
    .unwrap();
    assert_eq!(
        remove.effects(),
        vec![Effect::AggregateTimestamp, Effect::UpdatedEvent]
    );
    assert!(ptr::eq(remove.references[0], &old));
}
#[test]
fn removal_uses_explicit_wrapper_equality_not_uuid_equality() {
    let a = Ref {
        id: [1; 16],
        wrapper: 1,
    };
    let b = Ref {
        id: [1; 16],
        wrapper: 2,
    };
    let p = plan_remove_dependencies(
        &[&a, &b, &a],
        &[&a],
        |x, y| x.wrapper == y.wrapper,
        MutationAdmission::Allowed,
    )
    .unwrap();
    assert_eq!(p.references.len(), 1);
    assert!(ptr::eq(p.references[0], &b));
}
#[test]
fn removal_preserves_contains_argument_direction() {
    let a = Ref {
        id: [1; 16],
        wrapper: 1,
    };
    let b = Ref {
        id: [2; 16],
        wrapper: 2,
    };
    let p = plan_remove_dependencies(
        &[&a],
        &[&b],
        |existing, searched_in| existing.wrapper < searched_in.wrapper,
        MutationAdmission::Allowed,
    )
    .unwrap();
    assert!(p.references.is_empty());
}
#[test]
fn rejected_and_unknown_add_mutations_do_not_produce_effect_plans() {
    let a = Ref {
        id: [1; 16],
        wrapper: 1,
    };
    assert!(matches!(
        plan_add_dependencies(
            &[],
            &[&a],
            |r| r.id,
            MutationAdmission::ThrowsUnsupportedOperation
        ),
        Err(DependencyError::UnsupportedMutation)
    ));
    assert!(matches!(
        plan_add_dependencies(&[], &[&a], |r| r.id, MutationAdmission::Unresolved),
        Err(DependencyError::UnresolvedMutationCapability)
    ));
    assert_eq!(a.wrapper, 1);
}
#[test]
fn even_noop_removal_requires_successful_remove_if_admission() {
    let a = Ref {
        id: [1; 16],
        wrapper: 1,
    };
    for existing in [&[][..], &[&a][..]] {
        assert!(matches!(
            plan_remove_dependencies(
                existing,
                &[],
                |_, _| panic!("predicate must not run"),
                MutationAdmission::ThrowsUnsupportedOperation
            ),
            Err(DependencyError::UnsupportedMutation)
        ));
        assert!(matches!(
            plan_remove_dependencies(
                existing,
                &[],
                |_, _| panic!("predicate must not run"),
                MutationAdmission::Unresolved
            ),
            Err(DependencyError::UnresolvedMutationCapability)
        ));
    }
}
