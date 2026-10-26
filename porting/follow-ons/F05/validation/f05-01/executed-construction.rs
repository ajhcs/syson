// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::cell::Cell;
use syson_metamodel::ClassifierId as C;
use syson_model::{Document, Editor, NodeIds};
use syson_references::{FragmentIdentity as F, Registry, Resource};
use syson_view_usage::{Error, Ids, Presence, stage};
const LOCAL: &str = "model";
const LIB: &str = "sysmllibrary:///aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
fn ids(n: u32) -> NodeIds {
    NodeIds::new(
        &format!("10000000-0000-4000-8000-{n:012x}"),
        &format!("00000000-0000-4000-8000-{n:012x}"),
    )
    .unwrap()
}
fn allocation(n: u32) -> Ids {
    Ids {
        membership: ids(n),
        view: ids(n + 1),
        typing: ids(n + 2),
    }
}
fn node(n: u32, class: &str, extra: Value) -> Value {
    let id = ids(n);
    let mut data = extra.as_object().unwrap().clone();
    data.insert("elementId".into(), json!(id.element));
    json!({"id":id.resource,"eClass":format!("sysml:{class}"),"data":data})
}
fn document(roots: Vec<Value>) -> Document {
    Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":roots}).to_string()).unwrap()
}
fn library() -> Resource {
    Resource {
        key: LIB.into(),
        fragments: F::SemanticElementId,
        document: document(vec![node(
            100,
            "Package",
            json!({"declaredName":"StandardViewDefinitions","ownedRelationship":[node(101,"OwningMembership",json!({"ownedRelatedElement":[node(102,"ViewDefinition",json!({"declaredName":"GeneralView"}))]}))]}),
        )]),
    }
}
fn editor() -> Editor {
    Editor::new(document(vec![node(
        1,
        "Package",
        json!({"declaredName":"Vehicle"}),
    )]))
}
fn make(editor: &Editor, n: u32, libraries: Vec<Resource>) -> syson_model::Transaction {
    stage(
        editor.begin(),
        &ids(1).element,
        allocation(n),
        LOCAL,
        libraries,
        Presence::ResourceSet,
        || false,
    )
    .unwrap()
}
fn registry(doc: Document, lib: Resource) -> Registry {
    Registry::new(vec![
        Resource {
            key: LOCAL.into(),
            document: doc,
            fragments: F::JsonNodeId,
        },
        lib,
    ])
    .unwrap()
}
#[test]
fn staged_selected_prefix_constructs_three_distinct_objects_and_actual_external_wire() {
    let mut editor = editor();
    let original = editor.document().to_json().unwrap();
    let lib = library();
    let original_lib = lib.document.to_json().unwrap();
    let tx = make(&editor, 10, vec![lib]);
    assert_eq!(editor.document().to_json().unwrap(), original);
    let view = tx.document().element(&ids(11).element).unwrap();
    assert_eq!(view["id"], ids(11).resource);
    assert_eq!(view["data"]["declaredName"], "view1");
    assert_eq!(view["data"]["isComposite"], true);
    assert_eq!(view["data"]["ownedRelationship"][0]["id"], ids(12).resource);
    let typing = tx.document().element(&ids(12).element).unwrap();
    assert_eq!(typing["data"]["elementId"], ids(12).element);
    assert_eq!(typing["data"]["typedFeature"], ids(11).resource);
    assert_eq!(
        typing["data"]["type"],
        format!("sysml:ViewDefinition {LIB}#{}", ids(102).element)
    );
    let r = registry(tx.document().clone(), library());
    let t = r
        .resolve(LOCAL, typing["data"].get("type"), C::ViewDefinition)
        .unwrap()
        .unwrap();
    assert_eq!(t.resource(), LIB);
    assert_eq!(t.element_id(), ids(102).element);
    let self_ref = r
        .resolve(LOCAL, typing["data"].get("typedFeature"), C::ViewUsage)
        .unwrap()
        .unwrap();
    assert_eq!(self_ref.element_id(), ids(11).element);
    assert!(editor.commit(tx).unwrap());
    assert_eq!(library().document.to_json().unwrap(), original_lib);
    assert!(editor.undo());
    assert_eq!(editor.document().to_json().unwrap(), original);
    assert!(editor.redo());
    assert_eq!(
        editor.document().element(&ids(11).element).unwrap()["data"]["declaredName"],
        "view1"
    );
}
#[test]
fn counting_includes_attached_root_and_only_exact_view_usage_siblings() {
    let mut editor = editor();
    editor.commit(make(&editor, 10, vec![])).unwrap();
    let mut tx = editor.begin();
    tx.create_part_usage(&ids(1).element, ids(31), ids(30), "other")
        .unwrap();
    editor.commit(tx).unwrap();
    let tx = make(&editor, 20, vec![]);
    assert_eq!(
        tx.document().element(&ids(21).element).unwrap()["data"]["declaredName"],
        "view2"
    );
}
#[test]
fn absent_general_view_and_missing_resource_set_assign_null_instead_of_unresolved_guessing() {
    for presence in [Presence::ResourceSet, Presence::WithoutResourceSet] {
        let editor = editor();
        let tx = stage(
            editor.begin(),
            &ids(1).element,
            allocation(10),
            LOCAL,
            vec![],
            presence,
            || false,
        )
        .unwrap();
        let typing = tx.document().element(&ids(12).element).unwrap();
        assert!(typing["data"].get("type").unwrap().is_null());
        assert_eq!(typing["data"]["typedFeature"], ids(11).resource);
    }
}
#[test]
fn local_general_view_uses_json_node_identity_without_semantic_fallback() {
    let mut editor = Editor::new(document(vec![node(
        1,
        "Package",
        json!({"declaredName":"StandardViewDefinitions","ownedRelationship":[node(101,"OwningMembership",json!({"ownedRelatedElement":[node(102,"ViewDefinition",json!({"declaredName":"GeneralView"}))]}))]}),
    )]));
    let tx = make(&editor, 10, vec![]);
    let typing = tx.document().element(&ids(12).element).unwrap();
    assert_eq!(typing["data"]["type"], ids(102).resource);
    assert_ne!(typing["data"]["type"], ids(102).element);
    editor.commit(tx).unwrap();
}
#[test]
fn type_parent_uses_feature_membership_without_changing_owned_feature_count_rule() {
    let editor = Editor::new(document(vec![node(
        1,
        "PartDefinition",
        json!({"declaredName":"TypeParent"}),
    )]));
    let tx = make(&editor, 10, vec![]);
    assert_eq!(
        tx.document().element(&ids(10).element).unwrap()["eClass"],
        "sysml:FeatureMembership"
    );
    assert_eq!(
        tx.document().element(&ids(11).element).unwrap()["data"]["declaredName"],
        "view1"
    );
}
#[test]
fn cancelling_returned_transaction_and_latched_mid_plan_cancellation_preserve_committed_document() {
    let editor = editor();
    let original = editor.document().clone();
    make(&editor, 10, vec![library()]).cancel();
    assert_eq!(editor.document(), &original);
    for after in [0, 3, 8, 14] {
        let polls = Cell::new(0);
        let result = stage(
            editor.begin(),
            &ids(1).element,
            allocation(10),
            LOCAL,
            vec![library()],
            Presence::ResourceSet,
            || {
                let old = polls.get();
                polls.set(old + 1);
                old >= after
            },
        );
        assert!(
            matches!(result, Err(Error::Cancelled)),
            "poll threshold {after}"
        );
        assert_eq!(editor.document(), &original);
    }
}
#[test]
fn resource_and_semantic_collisions_including_late_typing_collision_leave_editor_unchanged() {
    let editor = editor();
    let original = editor.document().clone();
    for field in [0, 1, 2, 3] {
        let mut a = allocation(10);
        match field {
            0 => a.view.resource = ids(1).resource,
            1 => a.view.element = ids(1).element,
            2 => a.typing.resource = ids(1).resource,
            _ => a.typing.element = ids(1).element,
        };
        assert!(
            stage(
                editor.begin(),
                &ids(1).element,
                a,
                LOCAL,
                vec![library()],
                Presence::ResourceSet,
                || false
            )
            .is_err()
        );
        assert_eq!(editor.document(), &original);
    }
}
#[test]
fn failed_construction_consumes_prior_staged_work_and_stale_commit_is_rejected() {
    let mut editor = editor();
    let original = editor.document().clone();
    let mut prior = editor.begin();
    prior.create_package(None, ids(80), None, "prior").unwrap();
    let mut bad = allocation(10);
    bad.typing.element = ids(1).element;
    assert!(
        stage(
            prior,
            &ids(1).element,
            bad,
            LOCAL,
            vec![],
            Presence::ResourceSet,
            || false
        )
        .is_err()
    );
    assert_eq!(editor.document(), &original);
    let stale = make(&editor, 10, vec![]);
    let mut other = editor.begin();
    other.rename(&ids(1).element, "Changed").unwrap();
    editor.commit(other).unwrap();
    assert!(editor.commit(stale).is_err());
    assert!(editor.document().element(&ids(11).element).is_none());
    assert_eq!(
        editor.document().element(&ids(1).element).unwrap()["data"]["declaredName"],
        "Changed"
    );
}
#[test]
fn success_preserves_prior_staged_work_in_one_caller_commit() {
    let mut editor = editor();
    let original = editor.document().clone();
    let mut tx = editor.begin();
    tx.create_package(None, ids(80), None, "prior").unwrap();
    let tx = stage(
        tx,
        &ids(1).element,
        allocation(10),
        LOCAL,
        vec![],
        Presence::ResourceSet,
        || false,
    )
    .unwrap();
    editor.commit(tx).unwrap();
    assert!(editor.document().element(&ids(80).element).is_some());
    assert!(editor.undo());
    assert_eq!(editor.document(), &original);
}
#[test]
fn unresolved_import_observation_is_not_silently_encoded_as_null() {
    let imp = node(110, "NamespaceImport", json!({}));
    let lib = Resource {
        key: LIB.into(),
        fragments: F::SemanticElementId,
        document: document(vec![node(
            100,
            "Package",
            json!({"declaredName":"StandardViewDefinitions","ownedRelationship":[imp]}),
        )]),
    };
    let editor = editor();
    let original = editor.document().clone();
    assert!(matches!(
        stage(
            editor.begin(),
            &ids(1).element,
            allocation(10),
            LOCAL,
            vec![lib],
            Presence::ResourceSet,
            || false
        ),
        Err(Error::Planning(_))
    ));
    assert_eq!(editor.document(), &original);
}
#[test]
fn wrong_resource_fragment_policy_or_unsupported_uri_refuses_after_query_success() {
    for uri in [LIB, "https://example.invalid/library"] {
        let mut lib = library();
        lib.key = uri.into();
        if uri == LIB {
            lib.fragments = F::JsonNodeId;
        }
        let editor = editor();
        let original = editor.document().clone();
        assert!(
            stage(
                editor.begin(),
                &ids(1).element,
                allocation(10),
                LOCAL,
                vec![lib],
                Presence::ResourceSet,
                || false
            )
            .is_err()
        );
        assert_eq!(editor.document(), &original);
    }
}
#[test]
fn resource_context_and_parent_admission_are_explicit() {
    let editor = editor();
    for key in ["", LIB] {
        assert!(matches!(
            stage(
                editor.begin(),
                &ids(1).element,
                allocation(10),
                key,
                vec![library()],
                Presence::ResourceSet,
                || false
            ),
            Err(Error::ResourceIdentity)
        ));
    }
    assert!(
        stage(
            editor.begin(),
            &ids(99).element,
            allocation(10),
            LOCAL,
            vec![],
            Presence::ResourceSet,
            || false
        )
        .is_err()
    );
    let foreign = make(&editor, 10, vec![]);
    let mut other = super_editor();
    assert!(other.commit(foreign).is_err());
}
fn super_editor() -> Editor {
    Editor::new(Document::empty())
}
