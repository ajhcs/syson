// SPDX-License-Identifier: EPL-2.0
use syson_extensions::{
    Capability::{Observed, Unavailable},
    Completion, MoveRegistry, MoveStatus, ProviderId, ProviderSlot, RegistryError,
};

fn id(label: &str) -> ProviderId {
    ProviderId::new(label).unwrap()
}
fn registry(names: &[&str]) -> MoveRegistry {
    MoveRegistry::new(
        Some(id("default")),
        names.iter().map(|name| id(name)).collect(),
    )
    .unwrap()
}
fn slot(index: usize, label: &str) -> ProviderSlot {
    ProviderSlot::Custom {
        index,
        id: id(label),
    }
}

#[test]
fn no_custom_providers_uses_default() {
    let plan = registry(&[]).snapshot(vec![]).unwrap().select().unwrap();
    assert_eq!(plan.selected(), &ProviderSlot::Default(id("default")));
}

#[test]
fn all_false_uses_default() {
    let plan = registry(&["a", "b"])
        .snapshot(vec![Observed(false), Observed(false)])
        .unwrap()
        .select()
        .unwrap();
    assert_eq!(plan.selected(), &ProviderSlot::Default(id("default")));
}

#[test]
fn first_true_wins_in_exact_injected_order() {
    let a = registry(&["z", "a"])
        .snapshot(vec![Observed(true), Observed(true)])
        .unwrap()
        .select()
        .unwrap();
    let b = registry(&["a", "z"])
        .snapshot(vec![Observed(true), Observed(true)])
        .unwrap()
        .select()
        .unwrap();
    assert_eq!(a.selected(), &slot(0, "z"));
    assert_eq!(b.selected(), &slot(0, "a"));
}

#[test]
fn rejects_missing_default_before_selection() {
    assert_eq!(
        MoveRegistry::new(None, vec![id("a")]).unwrap_err(),
        RegistryError::MissingDefault
    );
}

#[test]
fn rust_label_admission_boundaries_are_explicit() {
    for bad in ["", " ", "\t\n", "\u{2003}"] {
        assert_eq!(ProviderId::new(bad), Err(RegistryError::InvalidLabel));
    }
    assert!(ProviderId::new("x".repeat(128)).is_ok());
    assert_eq!(
        ProviderId::new("x".repeat(129)),
        Err(RegistryError::InvalidLabel)
    );
    assert!(ProviderId::new("é".repeat(64)).is_ok());
    assert_eq!(
        ProviderId::new("é".repeat(65)),
        Err(RegistryError::InvalidLabel)
    );
    assert_eq!(id(" spaced ").as_str(), " spaced ");
}

#[test]
fn duplicate_labels_are_not_deduplicated() {
    let registry = registry(&["same", "same", "default"]);
    assert_eq!(
        registry.custom_providers(),
        &[id("same"), id("same"), id("default")]
    );
    let plan = registry
        .snapshot(vec![Observed(false), Observed(true)])
        .unwrap()
        .select()
        .unwrap();
    assert_eq!(plan.selected(), &slot(1, "same"));
}

#[test]
fn unavailable_before_match_is_not_a_false_capability() {
    let packet = registry(&["a", "b"])
        .snapshot(vec![Unavailable, Observed(true)])
        .unwrap();
    assert_eq!(
        packet.select(),
        Err(RegistryError::MissingCapability { custom_index: 0 })
    );
}

#[test]
fn short_packet_reports_first_reached_missing_slot() {
    let packet = registry(&["a", "b"])
        .snapshot(vec![Observed(false)])
        .unwrap();
    assert_eq!(
        packet.select(),
        Err(RegistryError::MissingCapability { custom_index: 1 })
    );
}

#[test]
fn later_observations_cannot_change_earlier_match() {
    let registry = registry(&["a", "b"]);
    for observations in [
        vec![Observed(true)],
        vec![Observed(true), Unavailable],
        vec![Observed(true), Observed(false)],
        vec![Observed(true), Observed(true)],
    ] {
        assert_eq!(
            registry
                .snapshot(observations)
                .unwrap()
                .select()
                .unwrap()
                .selected(),
            &slot(0, "a")
        );
    }
}

#[test]
fn excess_observations_do_not_silently_bind_to_unknown_providers() {
    let error = registry(&["a"])
        .snapshot(vec![Observed(true), Observed(false)])
        .unwrap_err();
    assert_eq!(
        error,
        RegistryError::ExcessObservations {
            registered: 1,
            supplied: 2
        }
    );
}

#[test]
fn packets_own_independent_registrations_and_observations() {
    let mut providers = vec![id("first")];
    let registry = MoveRegistry::new(Some(id("fallback")), providers.clone()).unwrap();
    let mut observed = vec![Observed(true)];
    let old = registry.snapshot(observed.clone()).unwrap();
    providers[0] = id("changed");
    observed[0] = Observed(false);
    let next = registry.snapshot(observed).unwrap();
    drop(registry);
    assert_eq!(old.clone().select().unwrap().selected(), &slot(0, "first"));
    assert_eq!(
        next.select().unwrap().selected(),
        &ProviderSlot::Default(id("fallback"))
    );
}

#[test]
fn selected_null_falls_to_default_not_next_custom() {
    let plan = registry(&["a", "b"])
        .snapshot(vec![Observed(true), Observed(true)])
        .unwrap()
        .select()
        .unwrap();
    assert_eq!(
        plan.complete(None),
        Completion::InvokeDefault(id("default"))
    );
}

#[test]
fn selected_success_and_failure_are_both_returned_without_fallback() {
    let plan = registry(&["a"])
        .snapshot(vec![Observed(true)])
        .unwrap()
        .select()
        .unwrap();
    for is_success in [true, false] {
        let status = MoveStatus {
            is_success,
            message: Some("caller result".into()),
        };
        assert_eq!(
            plan.complete(Some(status.clone())),
            Completion::Returned(Some(status))
        );
    }
}

#[test]
fn default_null_is_returned_without_recursive_fallback() {
    let plan = registry(&[]).snapshot(vec![]).unwrap().select().unwrap();
    assert_eq!(plan.complete(None), Completion::Returned(None));
}

#[test]
fn default_failure_is_also_a_completed_result() {
    let plan = registry(&[]).snapshot(vec![]).unwrap().select().unwrap();
    let failure = MoveStatus {
        is_success: false,
        message: Some("denied".into()),
    };
    assert_eq!(
        plan.complete(Some(failure.clone())),
        Completion::Returned(Some(failure))
    );
}

#[test]
fn exhaustive_small_truth_tables_match_first_true_position() {
    for count in 0..=8 {
        let names = vec!["duplicate"; count];
        let registry = registry(&names);
        for mask in 0..(1usize << count) {
            let observations = (0..count)
                .map(|bit| Observed(mask & (1 << bit) != 0))
                .collect();
            let plan = registry.snapshot(observations).unwrap().select().unwrap();
            let expected = (0..count)
                .find(|bit| mask & (1 << bit) != 0)
                .map_or(ProviderSlot::Default(id("default")), |index| {
                    slot(index, "duplicate")
                });
            assert_eq!(plan.selected(), &expected, "count={count} mask={mask}");
        }
    }
}

#[test]
fn null_message_is_a_non_null_status_and_does_not_request_fallback() {
    let plan = registry(&["a"])
        .snapshot(vec![Observed(true)])
        .unwrap()
        .select()
        .unwrap();
    let status = MoveStatus {
        is_success: false,
        message: None,
    };
    assert_eq!(
        plan.complete(Some(status.clone())),
        Completion::Returned(Some(status))
    );
}
