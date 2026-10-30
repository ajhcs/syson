// Copyright (c) 2024, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
// Source-derived branch fixtures, not captured upstream runtime observations.
use Observed::{Known, Unresolved};
use std::collections::VecDeque;
use syson_domain::{
    identity_context::*,
    uuid::{Uuid, uuid_v5},
};

fn fresh(n: u8) -> Uuid {
    let mut bytes = [0; 16];
    bytes[6] = 0x40;
    bytes[8] = 0x80;
    bytes[15] = n;
    Uuid(bytes)
}
struct Provider {
    calls: usize,
    ids: VecDeque<Uuid>,
}
impl FreshIdProvider for Provider {
    type Error = &'static str;
    fn next_v4(&mut self) -> Result<Uuid, Self::Error> {
        self.calls += 1;
        self.ids.pop_front().ok_or("exhausted")
    }
}
fn provider() -> Provider {
    Provider {
        calls: 0,
        ids: VecDeque::from([fresh(1), fresh(2), fresh(3)]),
    }
}
fn ordinary() -> ElementContext<'static> {
    ElementContext {
        class: ElementClass::Other,
        resource: ResourceObservation::Unresolved,
        library_namespace: LibraryNamespaceObservation::Absent,
        declared_name: Unresolved,
        qualified_name: Unresolved,
    }
}
fn root<'a>(uri: Option<&'a str>, name: Option<&'a str>) -> ElementContext<'a> {
    ElementContext {
        class: ElementClass::LibraryPackage { standard: true },
        resource: ResourceObservation::Present { uri: Known(uri) },
        library_namespace: LibraryNamespaceObservation::Unresolved,
        declared_name: Unresolved,
        qualified_name: Known(name),
    }
}
fn member() -> ElementContext<'static> {
    ElementContext {
        class: ElementClass::Other,
        resource: ResourceObservation::Unresolved,
        library_namespace: LibraryNamespaceObservation::LibraryPackage {
            standard: true,
            element_id: Known(Some("cdd5d1e3-fe4b-52bd-8a01-51a53f22ba47")),
        },
        declared_name: Known(Some("Anything")),
        qualified_name: Known(Some("Base::Anything")),
    }
}
#[test]
fn ordinary_context_uses_injected_fresh_id_without_unrelated_getters() {
    let mut p = provider();
    assert_eq!(
        generate_semantic_id(Some(ordinary()), &mut p).unwrap(),
        GeneratedSemanticId {
            id: SemanticId(fresh(1)),
            origin: IdentityOrigin::FreshV4
        }
    );
    assert_eq!(p.calls, 1);
}
#[test]
fn nonstandard_packages_and_nonstandard_library_contexts_stay_fresh() {
    for namespace in [
        LibraryNamespaceObservation::Absent,
        LibraryNamespaceObservation::OtherNamespace,
        LibraryNamespaceObservation::LibraryPackage {
            standard: false,
            element_id: Unresolved,
        },
    ] {
        let mut c = ordinary();
        c.class = ElementClass::LibraryPackage { standard: false };
        c.library_namespace = namespace;
        c.resource = ResourceObservation::Present { uri: Known(None) };
        assert_eq!(
            generate_semantic_id(Some(c), &mut provider())
                .unwrap()
                .origin,
            IdentityOrigin::FreshV4
        );
    }
}
#[test]
fn source_library_root_goldens_are_repeatable_but_consume_fresh_each_time() {
    for (uri, name, expected) in [
        (
            "kermllibrary:///base",
            "Base",
            "cdd5d1e3-fe4b-52bd-8a01-51a53f22ba47",
        ),
        (
            "kermllibrary:///scalar",
            "ScalarValues",
            "40bb440c-5036-58e1-8675-5afccb8b8f1d",
        ),
        (
            "sysmllibrary:///isq",
            "ISQ",
            "3fcc2ef4-a31c-522b-b69d-717d74bccfa6",
        ),
    ] {
        let mut p = provider();
        let c = root(Some(uri), Some(name));
        let a = generate_semantic_id(Some(c), &mut p).unwrap();
        let b = generate_semantic_id(Some(c), &mut p).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.id.0.to_string(), expected);
        assert_eq!(p.calls, 2);
        assert_eq!(a.origin, IdentityOrigin::StandardLibraryRootV5);
    }
}
#[test]
fn only_exact_kerml_colon_prefix_selects_kerml_base() {
    let sysml = generate_semantic_id(
        Some(root(Some("sysmllibrary:/"), Some("Base"))),
        &mut provider(),
    )
    .unwrap();
    for uri in [
        "kermllibrary-other",
        "KERMLLIBRARY:/",
        "file:/ordinary",
        "",
        "sysmllibrary:/",
    ] {
        assert_eq!(
            generate_semantic_id(Some(root(Some(uri), Some("Base"))), &mut provider()).unwrap(),
            sysml
        );
    }
    assert_ne!(
        generate_semantic_id(
            Some(root(Some("kermllibrary:/"), Some("Base"))),
            &mut provider()
        )
        .unwrap(),
        sysml
    );
}
#[test]
fn absent_resource_and_null_name_fall_back_without_resolving_unused_values() {
    let mut c = root(None, None);
    c.resource = ResourceObservation::Absent;
    c.qualified_name = Unresolved;
    assert_eq!(
        generate_semantic_id(Some(c), &mut provider())
            .unwrap()
            .origin,
        IdentityOrigin::FreshV4
    );
    assert_eq!(
        generate_semantic_id(Some(root(Some("kermllibrary:/"), None)), &mut provider())
            .unwrap()
            .origin,
        IdentityOrigin::FreshV4
    );
}
#[test]
fn null_uri_and_unresolved_resource_uri_name_remain_distinct() {
    let mut c = root(None, None);
    c.qualified_name = Unresolved;
    assert_eq!(
        generate_semantic_id(Some(c), &mut provider()),
        Err(IdentityError::NullResourceUri)
    );
    c.resource = ResourceObservation::Unresolved;
    assert_eq!(
        generate_semantic_id(Some(c), &mut provider()),
        Err(IdentityError::Unresolved(ContextField::Resource))
    );
    c.resource = ResourceObservation::Present { uri: Unresolved };
    assert_eq!(
        generate_semantic_id(Some(c), &mut provider()),
        Err(IdentityError::Unresolved(ContextField::ResourceUri))
    );
    c.resource = ResourceObservation::Present {
        uri: Known(Some("kermllibrary:/")),
    };
    assert_eq!(
        generate_semantic_id(Some(c), &mut provider()),
        Err(IdentityError::Unresolved(ContextField::QualifiedName))
    );
}
#[test]
fn named_library_member_matches_source_golden_and_namespace_hex_case() {
    let mut c = member();
    let first = generate_semantic_id(Some(c), &mut provider()).unwrap();
    assert_eq!(
        first.id.0.to_string(),
        "d5b4e7df-e644-5f2f-b95e-cf6f1f6c076d"
    );
    assert_eq!(first.origin, IdentityOrigin::StandardLibraryMemberV5);
    c.library_namespace = LibraryNamespaceObservation::LibraryPackage {
        standard: true,
        element_id: Known(Some("CDD5D1E3-FE4B-52BD-8A01-51A53F22BA47")),
    };
    assert_eq!(
        generate_semantic_id(Some(c), &mut provider()).unwrap(),
        first
    );
}
#[test]
fn excluded_relationship_families_ignore_qualified_name_and_namespace_id() {
    for class in [
        ElementClass::Import,
        ElementClass::Membership,
        ElementClass::Specialization,
        ElementClass::TypeFeaturing,
    ] {
        let mut c = member();
        c.class = class;
        c.qualified_name = Unresolved;
        c.library_namespace = LibraryNamespaceObservation::LibraryPackage {
            standard: true,
            element_id: Unresolved,
        };
        assert_eq!(
            generate_semantic_id(Some(c), &mut provider())
                .unwrap()
                .origin,
            IdentityOrigin::FreshV4
        );
        c.declared_name = Unresolved;
        assert_eq!(
            generate_semantic_id(Some(c), &mut provider()),
            Err(IdentityError::Unresolved(ContextField::DeclaredName))
        );
    }
}
#[test]
fn null_declared_name_skips_while_empty_declared_and_qualified_names_count() {
    let mut c = member();
    c.declared_name = Known(None);
    c.qualified_name = Unresolved;
    assert_eq!(
        generate_semantic_id(Some(c), &mut provider())
            .unwrap()
            .origin,
        IdentityOrigin::FreshV4
    );
    c.declared_name = Known(Some(""));
    c.qualified_name = Known(Some(""));
    let result = generate_semantic_id(Some(c), &mut provider()).unwrap();
    assert_eq!(result.origin, IdentityOrigin::StandardLibraryMemberV5);
    let namespace = generate_semantic_id(
        Some(root(Some("kermllibrary:/"), Some("Base"))),
        &mut provider(),
    )
    .unwrap()
    .id
    .0;
    assert_eq!(result.id.0, uuid_v5(namespace, ""));
}
#[test]
fn unresolved_namespace_precedes_declared_getter_and_null_qname_ignores_id() {
    let mut c = member();
    c.library_namespace = LibraryNamespaceObservation::Unresolved;
    c.declared_name = Unresolved;
    assert_eq!(
        generate_semantic_id(Some(c), &mut provider()),
        Err(IdentityError::Unresolved(ContextField::LibraryNamespace))
    );
    c = member();
    c.qualified_name = Known(None);
    c.library_namespace = LibraryNamespaceObservation::LibraryPackage {
        standard: true,
        element_id: Unresolved,
    };
    assert_eq!(
        generate_semantic_id(Some(c), &mut provider())
            .unwrap()
            .origin,
        IdentityOrigin::FreshV4
    );
}
#[test]
fn reached_namespace_id_null_unresolved_invalid_and_unsupported_are_explicit() {
    for (id, expected) in [
        (Known(None), IdentityError::NullNamespaceElementId),
        (
            Unresolved,
            IdentityError::Unresolved(ContextField::NamespaceElementId),
        ),
        (
            Known(Some("1-1-1-1-1")),
            IdentityError::NamespaceUuid(UuidTextError::NonCanonicalSpelling),
        ),
        (
            Known(Some("gdd5d1e3-fe4b-52bd-8a01-51a53f22ba47")),
            IdentityError::NamespaceUuid(UuidTextError::InvalidHex),
        ),
    ] {
        let mut c = member();
        c.library_namespace = LibraryNamespaceObservation::LibraryPackage {
            standard: true,
            element_id: id,
        };
        assert_eq!(
            generate_semantic_id(Some(c), &mut provider()),
            Err(expected)
        );
    }
}
#[test]
fn provider_exhaustion_does_not_retry_or_bypass_deterministic_branch() {
    let mut p = Provider {
        calls: 0,
        ids: VecDeque::new(),
    };
    assert_eq!(
        generate_semantic_id(Some(root(Some("kermllibrary:/"), Some("Base"))), &mut p),
        Err(IdentityError::Fresh("exhausted"))
    );
    assert_eq!(p.calls, 1);
}
#[test]
fn eager_provider_precedes_null_element_and_enforces_v4_input_contract() {
    let mut p = provider();
    assert_eq!(
        generate_semantic_id(None, &mut p),
        Err(IdentityError::NullElement)
    );
    assert_eq!(p.calls, 1);
    let mut invalid = fresh(9);
    invalid.0[6] = 0x50;
    let mut p = Provider {
        calls: 0,
        ids: VecDeque::from([invalid]),
    };
    assert_eq!(
        generate_semantic_id(None, &mut p),
        Err(IdentityError::ProviderNotV4)
    );
    assert_eq!(p.calls, 1);
}
fn pair(resource: Uuid, semantic: Uuid) -> (String, String) {
    (resource.to_string(), semantic.to_string())
}
fn selected(pair: &(String, String)) -> SelectedIdentity<'_> {
    SelectedIdentity::from_fields(Some(&pair.0), Some(&pair.1)).unwrap()
}
#[test]
fn selected_roles_remain_separate_allow_equal_values_and_preserve_field_spelling() {
    let fields = (fresh(1).to_string().to_uppercase(), fresh(1).to_string());
    let identity = selected(&fields);
    assert_eq!(identity.fields(), (fields.0.as_str(), fields.1.as_str()));
    assert_eq!(identity.resource().0, identity.semantic().0);
    let registry = SelectedIdentityRegistry::restore([identity]).unwrap();
    assert!(registry.contains_resource(ResourceId(fresh(1))));
    assert!(registry.contains_semantic(SemanticId(fresh(1))));
}
#[test]
fn semantic_collision_is_atomic_and_does_not_reserve_new_resource() {
    let a = pair(fresh(1), fresh(2));
    let b = pair(fresh(3), fresh(2));
    let c = pair(fresh(3), fresh(4));
    let mut registry = SelectedIdentityRegistry::restore([selected(&a)]).unwrap();
    assert_eq!(
        registry.admit(selected(&b)),
        Err(Collision::Semantic(SemanticId(fresh(2))))
    );
    assert_eq!(registry.len(), 1);
    assert!(!registry.contains_resource(ResourceId(fresh(3))));
    registry.admit(selected(&c)).unwrap();
    assert_eq!(registry.len(), 2);
}
#[test]
fn resource_collision_does_not_reserve_semantic_id_and_scopes_are_independent() {
    let a = pair(fresh(1), fresh(2));
    let b = pair(fresh(1), fresh(3));
    let mut registry = SelectedIdentityRegistry::restore([selected(&a)]).unwrap();
    assert_eq!(
        registry.admit(selected(&b)),
        Err(Collision::Resource(ResourceId(fresh(1))))
    );
    assert!(!registry.contains_semantic(SemanticId(fresh(3))));
    assert_eq!(
        SelectedIdentityRegistry::restore([selected(&a)])
            .unwrap()
            .len(),
        1
    );
    let upper = (a.0.to_uppercase(), a.1.to_uppercase());
    assert_eq!(
        registry.admit(selected(&upper)),
        Err(Collision::Resource(ResourceId(fresh(1))))
    );
}
#[test]
fn selected_field_errors_are_role_specific_and_do_not_generate_ids() {
    let s = fresh(1).to_string();
    assert_eq!(
        SelectedIdentity::from_fields(None, Some(&s)),
        Err(SelectedIdentityError::MissingResource)
    );
    assert_eq!(
        SelectedIdentity::from_fields(Some(&s), None),
        Err(SelectedIdentityError::MissingSemantic)
    );
    assert_eq!(
        SelectedIdentity::from_fields(Some("not-a-uuid"), Some(&s)),
        Err(SelectedIdentityError::ResourceText(
            UuidTextError::NonCanonicalSpelling
        ))
    );
    assert!(SelectedIdentityRegistry::default().is_empty());
}

#[test]
fn qualified_name_unicode_quotes_and_whitespace_are_hashed_verbatim() {
    let mut c = member();
    c.declared_name = Known(Some("é😀"));
    c.qualified_name = Known(Some("Base::'é😀'"));
    let a = generate_semantic_id(Some(c), &mut provider()).unwrap();
    // Independently calculated with Python stdlib uuid.uuid5; not Java output.
    assert_eq!(a.id.0.to_string(), "8a23cabb-0bb5-5bd6-8d1b-d86d26765efc");
    c.qualified_name = Known(Some(" Base::'é😀' "));
    assert_ne!(generate_semantic_id(Some(c), &mut provider()).unwrap(), a);
}
