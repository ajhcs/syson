// SPDX-License-Identifier: EPL-2.0
use serde_json::json;
use syson_api_types::*;

fn attr<'a>(name: &'a str, raw: &str) -> Feature<'a> {
    Feature {
        name,
        value: FeatureValue::Attribute(Json::parse(raw).unwrap()),
    }
}
fn reference<'a>(name: &'a str, many: bool, value: Reference<'a>) -> Feature<'a> {
    Feature {
        name,
        value: FeatureValue::Reference {
            many,
            eannotations_identity: false,
            value,
        },
    }
}
fn target(id: Option<&str>, library: bool, virtual_link: bool) -> Target<'_> {
    Target {
        is_element: true,
        element_id: id,
        standard_library: library,
        virtual_link,
    }
}
#[test]
fn upstream_element_fixture_is_retained_without_claiming_executed_golden() {
    let raw = include_str!("fixtures/package1.json");
    let dto = ElementDto::parse(raw).unwrap();
    assert_eq!(dto.json().wire(), raw);
    assert_eq!(dto.json().value()["@type"], "Package");
    assert!(dto.json().value()["declaredShortName"].is_null());
}
#[test]
fn upstream_relationship_fixture_is_a_bare_array() {
    let raw = include_str!("fixtures/part_relationships.json");
    let dto = RelationshipsDto::parse(raw).unwrap();
    assert_eq!(dto.json().wire(), raw);
    assert_eq!(dto.json().value().as_array().unwrap().len(), 2);
    assert_eq!(dto.json().value()[0]["@type"], "FeatureTyping");
}
#[test]
fn unknown_properties_and_numeric_spellings_survive_wire_adapter() {
    let raw = r#"{ "@type":"Part", "foreign":{"x":1E2}, "negativeZero":-0, "huge":900719925474099312345678901 }"#;
    let dto = ElementDto::parse(raw).unwrap();
    assert_eq!(dto.json().wire(), raw);
    assert_eq!(dto.json().value()["foreign"]["x"].to_string(), "1e+2");
    assert_eq!(dto.json().value()["negativeZero"], json!(0));
    assert!(dto.json().wire().contains("900719925474099312345678901"));
}
#[test]
fn malformed_dto_shapes_are_explicit_gaps() {
    assert!(matches!(Json::parse("{"), Err(Gap::InvalidJson(_))));
    assert!(matches!(
        ElementDto::parse("null"),
        Err(Gap::ExpectedObject)
    ));
    assert!(matches!(
        ElementDto::parse(r#"{"@type":null}"#),
        Err(Gap::MissingType)
    ));
    assert!(matches!(
        ElementDto::parse(r#"{"@type":"Part","@id":null}"#),
        Err(Gap::InvalidId)
    ));
    assert!(matches!(
        RelationshipsDto::parse("null"),
        Err(Gap::ExpectedArray)
    ));
}
#[test]
fn direction_defaults_only_after_observed_binding() {
    assert_eq!(Direction::observed_query(None), Direction::Both);
    assert_eq!(
        Direction::observed_query(Some(Direction::In)).source_name(),
        "IN"
    );
    assert_eq!(Direction::Out.source_name(), "OUT");
    assert_eq!(QUERY_TIMEOUT_SECONDS, 20);
    assert!(ELEMENT_PATH.ends_with("/elements/{elementId}"));
    assert!(RELATIONSHIPS_PATH.ends_with("/elements/{relatedElementId}/relationships"));
}
#[test]
fn element_http_body_has_no_internal_payload_envelope() {
    let raw = r#"{"@type":"Part","@id":"element","extra":1E2}"#;
    let response = controller_response(
        Operation::ElementById,
        Context::Present,
        Dispatch::ElementSuccess(Json::parse(raw).unwrap()),
    )
    .unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.body.unwrap().wire(), raw);
}
#[test]
fn absent_context_and_non_success_payloads_have_no_controller_error_body() {
    for operation in [Operation::ElementById, Operation::Relationships] {
        for dispatch in [Dispatch::NonSuccess, Dispatch::CompletedWithoutPayload] {
            let response = controller_response(operation, Context::Present, dispatch).unwrap();
            assert_eq!(response.status, 404);
            assert!(response.body.is_none());
        }
        let response =
            controller_response(operation, Context::Absent, Dispatch::UnqualifiedFailure).unwrap();
        assert_eq!(response.status, 404);
        assert!(response.body.is_none());
    }
}
#[test]
fn matching_success_payload_type_is_required() {
    let response = controller_response(
        Operation::ElementById,
        Context::Present,
        Dispatch::RelationshipsSuccess(RelationshipsDto::parse("[]").unwrap()),
    )
    .unwrap();
    assert_eq!(response.status, 404);
    assert!(response.body.is_none());
}
#[test]
fn missing_related_target_is_empty_success_not_missing_element_failure() {
    let members = relationship_members(
        false,
        Direction::Both,
        &[],
        InverseRelationships::Unresolved,
    )
    .unwrap();
    assert!(members.0.is_empty());
    let response = controller_response(
        Operation::Relationships,
        Context::Present,
        Dispatch::RelationshipsSuccess(RelationshipsDto::parse("[]").unwrap()),
    )
    .unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.body.unwrap().wire(), "[]");
}
#[test]
fn timeout_context_and_null_success_are_not_invented_http_statuses() {
    assert!(matches!(
        controller_response(
            Operation::ElementById,
            Context::Unresolved,
            Dispatch::NonSuccess
        ),
        Err(Gap::ContextUnresolved)
    ));
    assert!(matches!(
        controller_response(
            Operation::Relationships,
            Context::Present,
            Dispatch::UnqualifiedFailure
        ),
        Err(Gap::UnmappedDispatchFailure)
    ));
    assert!(matches!(
        controller_response(
            Operation::ElementById,
            Context::Present,
            Dispatch::ElementSuccess(Json::parse("null").unwrap())
        ),
        Err(Gap::NullSuccessElement)
    ));
}
#[test]
fn attributes_keep_raw_numbers_enum_literals_and_nulls() {
    let dto = serialize_element(
        Some("id"),
        "Part",
        &[
            attr("n", "-0"),
            attr("transientProperty", "1E2"),
            Feature {
                name: "visibility",
                value: FeatureValue::EnumeratorLiteral(Some("public")),
            },
            Feature {
                name: "missing",
                value: FeatureValue::EnumeratorLiteral(None),
            },
        ],
    )
    .unwrap();
    assert!(dto.json().wire().contains(r#""n":-0"#));
    assert!(dto.json().wire().contains(r#""transientProperty":1E2"#));
    assert_eq!(dto.json().value()["visibility"], "public");
    assert!(dto.json().value()["missing"].is_null());
}
#[test]
fn single_library_target_is_included_while_virtual_and_null_id_are_omitted() {
    let dto = serialize_element(
        Some("id"),
        "Part",
        &[
            reference(
                "library",
                false,
                Reference::Target(target(Some("lib"), true, false)),
            ),
            reference(
                "virtual",
                false,
                Reference::Target(target(Some("implicit"), false, true)),
            ),
            reference(
                "nullId",
                false,
                Reference::Target(target(None, false, false)),
            ),
        ],
    )
    .unwrap();
    assert_eq!(dto.json().value()["library"], json!({"@id":"lib"}));
    assert!(dto.json().value().get("virtual").is_none());
    assert!(dto.json().value().get("nullId").is_none());
}
#[test]
fn many_references_filter_individually_but_preserve_duplicates() {
    let mut foreign = target(Some("other"), false, false);
    foreign.is_element = false;
    let dto = serialize_element(
        Some("id"),
        "Part",
        &[reference(
            "refs",
            true,
            Reference::List(vec![
                target(Some("keep"), false, false),
                target(Some("lib"), true, false),
                target(Some("virtual"), false, true),
                target(None, false, false),
                foreign,
                target(Some("keep"), false, false),
            ]),
        )],
    )
    .unwrap();
    assert_eq!(
        dto.json().value()["refs"],
        json!([{"@id":"keep"},{"@id":"keep"}])
    );
}
#[test]
fn null_empty_and_omitted_references_remain_distinct() {
    let dto = serialize_element(
        Some("id"),
        "Part",
        &[
            reference("nullMany", true, Reference::Null),
            reference("nonListMany", true, Reference::OtherNonNull),
            reference("otherSingle", false, Reference::OtherNonNull),
        ],
    )
    .unwrap();
    assert!(dto.json().value()["nullMany"].is_null());
    assert_eq!(dto.json().value()["nonListMany"], json!([]));
    assert!(dto.json().value().get("otherSingle").is_none());
}
#[test]
fn eannotations_filter_uses_reference_identity_not_its_spelling() {
    let dto = serialize_element(
        Some("id"),
        "Part",
        &[
            Feature {
                name: "differentName",
                value: FeatureValue::Reference {
                    many: true,
                    eannotations_identity: true,
                    value: Reference::Null,
                },
            },
            reference("eAnnotations", false, Reference::Null),
        ],
    )
    .unwrap();
    assert!(dto.json().value().get("differentName").is_none());
    assert!(dto.json().value()["eAnnotations"].is_null());
}
#[test]
fn source_feature_sort_uses_utf16_and_does_not_claim_jackson_byte_parity() {
    let dto = serialize_element(
        Some("id"),
        "Part",
        &[attr("\u{e000}", "null"), attr("\u{10000}", "null")],
    )
    .unwrap();
    assert!(
        dto.json().wire().find('\u{10000}').unwrap() < dto.json().wire().find('\u{e000}').unwrap()
    );
}
#[test]
fn invalid_source_observations_are_explicit_gaps() {
    assert!(matches!(
        serialize_element(None, "Part", &[]),
        Err(Gap::NullElementId)
    ));
    assert!(matches!(
        serialize_element(Some("id"), "Part", &[attr("@type", "1")]),
        Err(Gap::DuplicateOrReservedField(_))
    ));
    assert!(matches!(
        serialize_element(
            Some("id"),
            "Part",
            &[Feature {
                name: "x",
                value: FeatureValue::Unresolved
            }]
        ),
        Err(Gap::UnresolvedFeature(_))
    ));
}
#[test]
fn omitting_an_implied_descendant_does_not_prune_its_children() {
    let node = TraversalNode {
        token: 1,
        is_element: true,
        implied_relationship: true,
        children: vec![TraversalNode {
            token: 2,
            is_element: true,
            implied_relationship: true,
            children: vec![TraversalNode {
                token: 3,
                is_element: false,
                implied_relationship: false,
                children: vec![],
            }],
        }],
    };
    assert_eq!(visible_root_contents(&[node]), vec![1, 3]);
    assert!(
        visible_root_contents(&[TraversalNode {
            token: 9,
            is_element: false,
            implied_relationship: false,
            children: vec![]
        }])
        .is_empty()
    );
}
#[test]
fn relationship_union_deduplicates_object_identity_filters_implied_and_has_no_order_claim() {
    let a = RelationshipFact {
        object_identity: 1,
        implied: false,
    };
    let b = RelationshipFact {
        object_identity: 2,
        implied: true,
    };
    let c = RelationshipFact {
        object_identity: 3,
        implied: false,
    };
    let incoming = [&a, &c];
    let members = relationship_members(
        true,
        Direction::Both,
        &[&a, &b],
        InverseRelationships::Observed(&incoming),
    )
    .unwrap();
    assert_eq!(
        members
            .0
            .iter()
            .map(|r| r.object_identity)
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([1, 3])
    );
}
#[test]
fn known_adapter_absence_and_unresolved_inverse_facts_are_distinct() {
    let a = RelationshipFact {
        object_identity: 1,
        implied: false,
    };
    assert!(
        relationship_members(
            true,
            Direction::In,
            &[&a],
            InverseRelationships::KnownAdapterAbsent
        )
        .unwrap()
        .0
        .is_empty()
    );
    assert!(matches!(
        relationship_members(true, Direction::In, &[], InverseRelationships::Unresolved),
        Err(Gap::InverseReferencesUnresolved)
    ));
    assert_eq!(
        relationship_members(
            true,
            Direction::Out,
            &[&a],
            InverseRelationships::Unresolved
        )
        .unwrap()
        .0
        .len(),
        1
    );
}
#[test]
fn contradictory_same_object_facts_are_rejected() {
    let a = RelationshipFact {
        object_identity: 1,
        implied: false,
    };
    let b = RelationshipFact {
        object_identity: 1,
        implied: true,
    };
    assert!(matches!(
        relationship_members(
            true,
            Direction::Out,
            &[&a, &b],
            InverseRelationships::KnownAdapterAbsent
        ),
        Err(Gap::ConflictingRelationshipObservation)
    ));
}
