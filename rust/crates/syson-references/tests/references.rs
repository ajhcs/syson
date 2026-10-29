// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use syson_metamodel::ClassifierId as C;
use syson_model::{Document, Editor};
use syson_references::{Decoded, Error, FragmentIdentity as F, Registry, Resource, decode};
const NODE: &str = "aaaaaaaa-0000-4000-8000-000000000001";
const ELEMENT: &str = "bbbbbbbb-0000-4000-8000-000000000001";
const URI: &str = "sysmllibrary:///11111111-1111-4111-8111-111111111111";
const OTHER: &str = "sysmllibrary:///22222222-2222-4222-8222-222222222222";
fn document(class: &str, name: &str) -> Document {
    Document::from_json(
        &json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},
        "content":[{"id":NODE,"eClass":class,"data":{"elementId":ELEMENT,"declaredName":name}}]})
        .to_string(),
    )
    .unwrap()
}
fn resource(key: &str, document: Document, fragments: F) -> Resource {
    Resource {
        key: key.into(),
        document,
        fragments,
    }
}
fn one(key: &str, doc: Document, policy: F) -> Registry {
    Registry::new(vec![resource(key, doc, policy)]).unwrap()
}
fn external(id: &str) -> Value {
    json!(format!("{URI}#{id}"))
}
#[test]
fn missing_and_explicit_null_are_absent() {
    for value in [None, Some(&Value::Null)] {
        assert_eq!(decode(value), Ok(Decoded::Absent));
        assert!(
            Registry::new(vec![])
                .unwrap()
                .resolve("absent", value, C::Element)
                .unwrap()
                .is_none()
        );
    }
}
#[test]
fn bare_local_id_and_library_type_prefix_decode_without_normalization() {
    assert_eq!(
        decode(Some(&json!(NODE))),
        Ok(Decoded::LocalNode(NODE.into()))
    );
    assert_eq!(
        decode(Some(&json!(format!("sysml:Element {URI}#{ELEMENT}")))),
        Ok(Decoded::External {
            resource: URI.into(),
            fragment: ELEMENT.into(),
            class: Some(C::Element)
        })
    );
    let kerml = format!("kermllibrary:///11111111-1111-4111-8111-111111111111#{ELEMENT}");
    assert!(matches!(
        decode(Some(&json!(kerml))),
        Ok(Decoded::External { class: None, .. })
    ));
}
#[test]
fn malformed_empty_control_and_multiple_fragments_are_invalid() {
    for text in [
        "".to_string(),
        format!("{NODE}\n"),
        format!("{URI}#"),
        format!("{URI}#{ELEMENT}#{NODE}"),
        format!("sysml:Element  {URI}#{ELEMENT}"),
    ] {
        assert!(
            matches!(decode(Some(&json!(text))), Err(Error::Invalid(_))),
            "{text}"
        );
    }
}
#[test]
fn unadmitted_proxy_path_relative_scheme_and_encoding_families_are_unsupported() {
    for value in [
        json!({"$ref":NODE}),
        json!(42),
        json!([NODE]),
        json!(format!("../doc#{NODE}")),
        json!(format!("https://example.test/model#{NODE}")),
        json!("syson-import:qualifiedName#A::B"),
        json!(format!("{URI}#//@ownedRelationship.0")),
        json!(format!("{URI}#%61aaaaaaa-0000-4000-8000-000000000001")),
        json!(format!("sysml:PartUsage {NODE}")),
        json!(format!("ecore:EObject {URI}#{ELEMENT}")),
        json!(format!("sysmllibrary:///nested/file#{NODE}")),
    ] {
        assert!(
            matches!(decode(Some(&value)), Err(Error::Unsupported(_))),
            "{value}"
        );
    }
}
#[test]
fn actual_batmobile_local_reference_resolves_node_identity_only() {
    let doc = Document::from_json(include_str!("../../../../backend/application/syson-application-configuration/src/main/resources/templates/Batmobile.json")).unwrap();
    let r = one("vehicle", doc, F::SemanticElementId);
    let target = r
        .resolve(
            "vehicle",
            Some(&json!("b966a2dd-cd51-4996-8449-d4e9d5b48588")),
            C::PortDefinition,
        )
        .unwrap()
        .unwrap();
    assert_eq!(target.resource(), "vehicle");
    assert_eq!(target.node_id(), "b966a2dd-cd51-4996-8449-d4e9d5b48588");
    assert_eq!(target.element_id(), "80078007-9a1f-42eb-9456-1d0b8d63703d");
    assert!(matches!(
        r.resolve(
            "vehicle",
            Some(&json!(target.element_id())),
            C::PortDefinition
        ),
        Err(Error::Unresolved { .. })
    ));
}
#[test]
fn actual_native_library_wire_decodes_but_missing_registry_target_stays_unresolved() {
    let doc = Document::from_json(include_str!("../../../../backend/application/syson-application-configuration/src/main/resources/sysml.libraries/StandardViewDefinitions.json")).unwrap();
    let node = doc.element("2f4dccd0-6a18-535f-8ab2-2a36a621ccc3").unwrap();
    let wire = &node["data"]["importedNamespace"];
    assert_eq!(
        decode(Some(wire)),
        Ok(Decoded::External {
            resource: "sysmllibrary:///389615e8-f636-3405-928f-411d874ba837".into(),
            fragment: "f8861c88-124d-527d-9946-d707fb209b47".into(),
            class: Some(C::Namespace)
        })
    );
    assert!(matches!(
        one("views", doc.clone(), F::SemanticElementId).resolve("views", Some(wire), C::Namespace),
        Err(Error::Unresolved { .. })
    ));
}
#[test]
fn actual_general_view_object_resolves_under_explicit_synthetic_registry_uri() {
    let doc = Document::from_json(include_str!("../../../../backend/application/syson-application-configuration/src/main/resources/sysml.libraries/StandardViewDefinitions.json")).unwrap();
    // URI is an explicit test registry key, not the native filename-derived URI.
    let r = one(URI, doc, F::SemanticElementId);
    let wire = json!(format!(
        "sysml:ViewDefinition {URI}#03904fdf-d6f2-57b1-92d5-95d36b8208dc"
    ));
    let target = r.resolve(URI, Some(&wire), C::Element).unwrap().unwrap();
    assert_eq!(target.node_id(), "d7955624-057b-45d2-98f9-671776192420");
    assert_eq!(target.element_id(), "03904fdf-d6f2-57b1-92d5-95d36b8208dc");
    assert_eq!(
        r.target(&target).unwrap()["data"]["declaredName"],
        "GeneralView"
    );
}
#[test]
fn external_fragment_policy_is_explicit_without_identity_fallback() {
    let a = one(URI, document("sysml:PartDefinition", "A"), F::JsonNodeId);
    assert_eq!(
        a.resolve(URI, Some(&external(NODE)), C::Type)
            .unwrap()
            .unwrap()
            .element_id(),
        ELEMENT
    );
    assert!(matches!(
        a.resolve(URI, Some(&external(ELEMENT)), C::Element),
        Err(Error::Unresolved { .. })
    ));
    let b = one(
        URI,
        document("sysml:PartDefinition", "A"),
        F::SemanticElementId,
    );
    assert_eq!(
        b.resolve(URI, Some(&external(ELEMENT)), C::Type)
            .unwrap()
            .unwrap()
            .node_id(),
        NODE
    );
    assert!(matches!(
        b.resolve(URI, Some(&external(NODE)), C::Element),
        Err(Error::Unresolved { .. })
    ));
}
#[test]
fn equal_ids_in_two_resources_do_not_alias_or_fall_back() {
    let r = Registry::new(vec![
        resource(
            URI,
            document("sysml:PartDefinition", "A"),
            F::SemanticElementId,
        ),
        resource(
            OTHER,
            document("sysml:PartDefinition", "B"),
            F::SemanticElementId,
        ),
    ])
    .unwrap();
    let a = r
        .resolve(URI, Some(&external(ELEMENT)), C::Element)
        .unwrap()
        .unwrap();
    let b = r
        .resolve(URI, Some(&json!(format!("{OTHER}#{ELEMENT}"))), C::Element)
        .unwrap()
        .unwrap();
    assert_eq!(r.target(&a).unwrap()["data"]["declaredName"], "A");
    assert_eq!(r.target(&b).unwrap()["data"]["declaredName"], "B");
    let local = r
        .resolve(OTHER, Some(&json!(NODE)), C::Element)
        .unwrap()
        .unwrap();
    assert_eq!(local.resource(), OTHER);
    assert!(matches!(
        r.resolve("missing", Some(&json!(NODE)), C::Element),
        Err(Error::Unresolved { .. })
    ));
}
#[test]
fn unresolved_resource_and_fragment_preserve_exact_context() {
    let r = one(
        "local",
        document("sysml:PartDefinition", "A"),
        F::JsonNodeId,
    );
    assert_eq!(
        r.resolve("local", Some(&external(ELEMENT)), C::Element)
            .unwrap_err(),
        Error::Unresolved {
            resource: URI.into(),
            fragment: ELEMENT.into()
        }
    );
    assert_eq!(
        r.resolve("local", Some(&json!(ELEMENT)), C::Element)
            .unwrap_err(),
        Error::Unresolved {
            resource: "local".into(),
            fragment: ELEMENT.into()
        }
    );
}
#[test]
fn wire_type_and_caller_type_both_use_real_assignability() {
    let r = one(
        URI,
        document("sysml:PartDefinition", "A"),
        F::SemanticElementId,
    );
    let target = r
        .resolve(
            URI,
            Some(&json!(format!("sysml:Type {URI}#{ELEMENT}"))),
            C::Definition,
        )
        .unwrap()
        .unwrap();
    assert_eq!(target.element_id(), ELEMENT);
    assert_eq!(
        r.resolve(URI, Some(&external(ELEMENT)), C::PartUsage)
            .unwrap_err(),
        Error::TypeMismatch {
            expected: C::PartUsage,
            actual: C::PartDefinition
        }
    );
    assert_eq!(
        r.resolve(
            URI,
            Some(&json!(format!("sysml:PartUsage {URI}#{ELEMENT}"))),
            C::Element
        )
        .unwrap_err(),
        Error::TypeMismatch {
            expected: C::PartUsage,
            actual: C::PartDefinition
        }
    );
}
#[test]
fn nonclass_or_unknown_class_is_explicitly_unsupported() {
    let r = one(
        URI,
        document("sysml:PartDefinition", "A"),
        F::SemanticElementId,
    );
    assert!(matches!(
        r.resolve(URI, Some(&external(ELEMENT)), C::VisibilityKind),
        Err(Error::Unsupported(_))
    ));
    for class in [
        "external:PartDefinition",
        "sysml:Missing",
        "sysml:VisibilityKind",
    ] {
        assert!(matches!(
            one(URI, document(class, "A"), F::SemanticElementId).resolve(
                URI,
                Some(&external(ELEMENT)),
                C::Element
            ),
            Err(Error::Unsupported(_))
        ));
    }
}
#[test]
fn spelling_case_is_preserved_with_no_uri_or_id_normalization() {
    let r = one(
        URI,
        document("sysml:PartDefinition", "A"),
        F::SemanticElementId,
    );
    let upper = NODE.to_ascii_uppercase();
    assert_eq!(
        decode(Some(&json!(upper))),
        Ok(Decoded::LocalNode(upper.clone()))
    );
    assert!(matches!(
        r.resolve(URI, Some(&json!(upper)), C::Element),
        Err(Error::Unresolved { .. })
    ));
    assert!(matches!(
        decode(Some(&json!(format!(
            "SYSMLLIBRARY:///11111111-1111-4111-8111-111111111111#{ELEMENT}"
        )))),
        Err(Error::Unsupported(_))
    ));
}
#[test]
fn targets_cannot_be_replayed_against_identical_foreign_snapshots() {
    let a = one(
        URI,
        document("sysml:PartDefinition", "A"),
        F::SemanticElementId,
    );
    let b = one(
        URI,
        document("sysml:PartDefinition", "A"),
        F::SemanticElementId,
    );
    let target = a
        .resolve(URI, Some(&external(ELEMENT)), C::Element)
        .unwrap()
        .unwrap();
    assert!(a.target(&target.clone()).is_ok());
    assert_eq!(b.target(&target), Err(Error::StaleSnapshot));
}
#[test]
fn registry_owns_document_snapshot_after_caller_edit() {
    let doc = document("sysml:PartDefinition", "before");
    let r = one(URI, doc.clone(), F::SemanticElementId);
    let target = r
        .resolve(URI, Some(&external(ELEMENT)), C::Element)
        .unwrap()
        .unwrap();
    let mut editor = Editor::new(doc);
    let mut tx = editor.begin();
    tx.rename(ELEMENT, "after").unwrap();
    editor.commit(tx).unwrap();
    assert_eq!(
        editor.document().element(ELEMENT).unwrap()["data"]["declaredName"],
        "after"
    );
    assert_eq!(r.target(&target).unwrap()["data"]["declaredName"], "before");
}
#[test]
fn duplicate_and_empty_registry_keys_fail_admission() {
    let doc = document("sysml:PartDefinition", "A");
    assert!(matches!(
        Registry::new(vec![resource("", doc.clone(), F::JsonNodeId)]),
        Err(Error::Invalid(_))
    ));
    assert!(matches!(
        Registry::new(vec![
            resource("x", doc.clone(), F::JsonNodeId),
            resource("x", doc, F::SemanticElementId)
        ]),
        Err(Error::Invalid(_))
    ));
}

#[test]
fn actual_batmobile_wires_resolve_local_and_standard_view_targets() {
    let batmobile = Document::from_json(include_str!("../../../../backend/application/syson-application-configuration/src/main/resources/templates/Batmobile.json")).unwrap();
    let views = Document::from_json(include_str!("../../../../backend/application/syson-application-configuration/src/main/resources/sysml.libraries/StandardViewDefinitions.json")).unwrap();
    let local_wire = batmobile
        .element("775ea645-9c42-4e52-8cd8-de30ed8b865b")
        .unwrap()["data"]["type"]
        .clone();
    let view_wire = batmobile
        .element("5dba8007-30b4-4988-9d3f-d5687ba94faa")
        .unwrap()["data"]["type"]
        .clone();
    // This exact URI occurs in Batmobile, rather than being synthesized from a
    // filename or substituted with a test URI. Policy remains caller-supplied.
    let view_uri = "sysmllibrary:///faf517ae-dbcd-30a4-b3b9-3d9cb3bbf5c1";
    assert_eq!(local_wire, "b966a2dd-cd51-4996-8449-d4e9d5b48588");
    assert_eq!(
        view_wire,
        format!("sysml:ViewDefinition {view_uri}#03904fdf-d6f2-57b1-92d5-95d36b8208dc")
    );
    let r = Registry::new(vec![
        resource("vehicle", batmobile, F::SemanticElementId),
        resource(view_uri, views.clone(), F::SemanticElementId),
    ])
    .unwrap();
    let port = r
        .resolve("vehicle", Some(&local_wire), C::PortDefinition)
        .unwrap()
        .unwrap();
    assert_eq!(port.resource(), "vehicle");
    assert_eq!(port.element_id(), "80078007-9a1f-42eb-9456-1d0b8d63703d");
    let view = r
        .resolve("vehicle", Some(&view_wire), C::Type)
        .unwrap()
        .unwrap();
    assert_eq!(view.resource(), view_uri);
    assert_eq!(view.node_id(), "d7955624-057b-45d2-98f9-671776192420");
    assert_eq!(view.element_id(), "03904fdf-d6f2-57b1-92d5-95d36b8208dc");
    assert_eq!(
        r.target(&view).unwrap()["data"]["declaredName"],
        "GeneralView"
    );
    assert!(matches!(
        one(view_uri, views, F::JsonNodeId).resolve("vehicle", Some(&view_wire), C::Type),
        Err(Error::Unresolved { .. })
    ));
}

#[test]
fn overlapping_node_and_semantic_ids_select_only_the_requested_domain() {
    // Both identities exist in both domains but denote different objects. A
    // resolver that searches either domain first can return a plausible wrong target.
    let doc = Document::from_json(
        &json!({
            "json":{"version":"1.0"},
            "ns":{"sysml":"http://www.eclipse.org/syson/sysml"},
            "content":[
                {"id":NODE,"eClass":"sysml:PartDefinition",
                 "data":{"elementId":ELEMENT,"declaredName":"node-domain"}},
                {"id":ELEMENT,"eClass":"sysml:PartDefinition",
                 "data":{"elementId":NODE,"declaredName":"semantic-domain"}}
            ]
        })
        .to_string(),
    )
    .unwrap();
    let by_node = one(URI, doc.clone(), F::JsonNodeId);
    let by_semantic = one(URI, doc, F::SemanticElementId);
    for registry in [&by_node, &by_semantic] {
        let local = registry
            .resolve(URI, Some(&json!(NODE)), C::Type)
            .unwrap()
            .unwrap();
        assert_eq!(local.node_id(), NODE);
        assert_eq!(local.element_id(), ELEMENT);
        assert_eq!(
            registry.target(&local).unwrap()["data"]["declaredName"],
            "node-domain"
        );
    }
    let node = by_node
        .resolve(URI, Some(&external(NODE)), C::Type)
        .unwrap()
        .unwrap();
    let semantic = by_semantic
        .resolve(URI, Some(&external(NODE)), C::Type)
        .unwrap()
        .unwrap();
    assert_eq!((node.node_id(), node.element_id()), (NODE, ELEMENT));
    assert_eq!((semantic.node_id(), semantic.element_id()), (ELEMENT, NODE));
    assert_eq!(
        by_semantic.target(&semantic).unwrap()["data"]["declaredName"],
        "semantic-domain"
    );
}
