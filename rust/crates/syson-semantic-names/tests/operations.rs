// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use syson_model::{Document, Editor};
use syson_references::{Error as ReferenceError, FragmentIdentity, Resource};
use syson_semantic_names::{Error, NameKind as K, Snapshot};
fn id(n: u32) -> String {
    format!("00000000-0000-4000-8000-{n:012x}")
}
fn wire(n: u32) -> String {
    format!("10000000-0000-4000-8000-{n:012x}")
}
fn node(n: u32, class: &str, name: Option<&str>, children: Vec<Value>) -> Value {
    let mut value =
        json!({"id":wire(n),"eClass":format!("sysml:{class}"),"data":{"elementId":id(n)}});
    if let Some(name) = name {
        value["data"]["declaredName"] = json!(name);
    }
    if !children.is_empty() {
        value["data"][if class.ends_with("Membership") {
            "ownedRelatedElement"
        } else {
            "ownedRelationship"
        }] = json!(children);
    }
    value
}
fn membership(n: u32, child: Value) -> Value {
    node(n, "OwningMembership", None, vec![child])
}
fn doc(nodes: Vec<Value>) -> Document {
    Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":nodes}).to_string()).unwrap()
}
fn resource(key: &str, document: Document) -> Resource {
    Resource {
        key: key.into(),
        document,
        fragments: FragmentIdentity::SemanticElementId,
    }
}
fn snapshot(nodes: Vec<Value>) -> Snapshot {
    Snapshot::new(vec![resource("local", doc(nodes))]).unwrap()
}
fn root(children: Vec<Value>) -> Value {
    node(1, "Package", Some("Vehicle"), children)
}
fn redefine(n: u32, reference: Value) -> Value {
    let mut value = node(n, "Redefinition", None, vec![]);
    value["data"]["redefinedFeature"] = reference;
    value
}
#[test]
fn semantic_owner_skips_containing_relationship_and_owned_elements_flatten_in_order() {
    let s = snapshot(vec![root(vec![
        membership(2, node(3, "PartUsage", Some("a"), vec![])),
        membership(4, node(5, "PartDefinition", Some("b"), vec![])),
    ])]);
    let root = s.object("local", &id(1)).unwrap();
    let a = s.object("local", &id(3)).unwrap();
    let owner = s.owner(&a).unwrap().unwrap();
    assert_eq!(
        (owner.resource(), owner.element_id()),
        ("local", id(1).as_str())
    );
    assert!(
        s.owner(&s.object("local", &id(2)).unwrap())
            .unwrap()
            .is_none()
    );
    assert_eq!(
        s.owned_elements(&root)
            .unwrap()
            .iter()
            .map(|o| o.element_id().to_owned())
            .collect::<Vec<_>>(),
        vec![id(3), id(5)]
    );
    assert!(
        s.owned_elements(&s.object("local", &id(2)).unwrap())
            .unwrap()
            .is_empty()
    );
}
#[test]
fn qualified_names_use_effective_primary_and_actual_printable_policy() {
    let s = snapshot(vec![root(vec![
        membership(2, node(3, "PartUsage", Some("part"), vec![])),
        membership(4, node(5, "PartUsage", Some("O'Brien"), vec![])),
    ])]);
    assert_eq!(
        s.qualified_name(&s.object("local", &id(3)).unwrap())
            .unwrap()
            .as_deref(),
        Some("Vehicle::'part'")
    );
    assert_eq!(
        s.qualified_name(&s.object("local", &id(5)).unwrap())
            .unwrap()
            .as_deref(),
        Some("Vehicle::'O\\'Brien'")
    );
}
#[test]
fn unnamed_self_is_none_and_unnamed_owner_does_not_invent_ancestor_prefix() {
    let inner = node(
        3,
        "Package",
        None,
        vec![membership(4, node(5, "PartUsage", Some("leaf"), vec![]))],
    );
    let s = snapshot(vec![root(vec![membership(2, inner)])]);
    assert_eq!(
        s.qualified_name(&s.object("local", &id(3)).unwrap())
            .unwrap(),
        None
    );
    assert_eq!(
        s.qualified_name(&s.object("local", &id(5)).unwrap())
            .unwrap()
            .as_deref(),
        Some("leaf")
    );
}
#[test]
fn membership_owner_is_skipped_once_using_semantic_owner_not_econtainer() {
    let mut middle = node(3, "Membership", Some("ignored"), vec![]);
    middle["data"]["ownedRelationship"] =
        json!([membership(4, node(5, "PartUsage", Some("leaf"), vec![]))]);
    let s = snapshot(vec![root(vec![membership(2, middle)])]);
    let leaf = s.object("local", &id(5)).unwrap();
    assert_eq!(s.owner(&leaf).unwrap().unwrap().element_id(), id(3));
    assert_eq!(
        s.qualified_name(&leaf).unwrap().as_deref(),
        Some("Vehicle::leaf")
    );
}
#[test]
fn feature_inherits_first_redefinition_names_and_short_name_blocks_primary_fallback() {
    let mut base = node(20, "PartUsage", Some("baseName"), vec![]);
    base["data"]["declaredShortName"] = json!("bn");
    let inherited = node(
        3,
        "PartUsage",
        None,
        vec![redefine(4, json!(wire(20))), redefine(5, json!(wire(99)))],
    );
    let mut short_only = node(6, "PartUsage", None, vec![redefine(7, json!(wire(99)))]);
    short_only["data"]["declaredShortName"] = json!("s");
    let s = snapshot(vec![
        root(vec![membership(2, inherited), membership(8, short_only)]),
        base,
    ]);
    let a = s.object("local", &id(3)).unwrap();
    assert_eq!(s.name(&a, K::Name).unwrap().as_deref(), Some("baseName"));
    assert_eq!(s.name(&a, K::ShortName).unwrap().as_deref(), Some("bn"));
    assert_eq!(
        s.qualified_name(&a).unwrap().as_deref(),
        Some("Vehicle::baseName")
    );
    let b = s.object("local", &id(6)).unwrap();
    assert_eq!(s.name(&b, K::Name).unwrap(), None);
    assert_eq!(s.name(&b, K::ShortName).unwrap().as_deref(), Some("s"));
    assert_eq!(s.qualified_name(&b).unwrap(), None);
}
#[test]
fn absent_null_and_self_redefinition_are_known_absence() {
    let nodes = vec![
        node(1, "PartUsage", None, vec![]),
        node(2, "PartUsage", None, vec![redefine(3, Value::Null)]),
        node(4, "PartUsage", None, vec![redefine(5, json!(wire(4)))]),
    ];
    let s = snapshot(nodes);
    for n in [1, 2, 4] {
        assert_eq!(
            s.name(&s.object("local", &id(n)).unwrap(), K::Name)
                .unwrap(),
            None
        );
    }
}
#[test]
fn explicit_empty_name_stops_inheritance_and_malformed_name_is_refused() {
    let mut feature = node(1, "PartUsage", Some(""), vec![redefine(2, json!(wire(99)))]);
    let s = snapshot(vec![feature.clone()]);
    let object = s.object("local", &id(1)).unwrap();
    assert_eq!(s.name(&object, K::Name).unwrap().as_deref(), Some(""));
    assert_eq!(s.qualified_name(&object).unwrap(), None);
    feature["data"]["declaredName"] = json!(42);
    let s = snapshot(vec![feature]);
    assert_eq!(
        s.name(&s.object("local", &id(1)).unwrap(), K::Name),
        Err(Error::InvalidName)
    );
}
#[test]
fn cross_resource_library_inheritance_retains_local_owner_and_exact_resource() {
    let uri = "sysmllibrary:///aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    let local = root(vec![membership(
        2,
        node(
            3,
            "PartUsage",
            None,
            vec![redefine(
                4,
                json!(format!("sysml:Feature {uri}#{}", id(20))),
            )],
        ),
    )]);
    let s = Snapshot::new(vec![
        resource(
            "local",
            doc(vec![
                local,
                node(20, "PartUsage", Some("wrongLocal"), vec![]),
            ]),
        ),
        resource(
            uri,
            doc(vec![node(20, "PartUsage", Some("libraryName"), vec![])]),
        ),
    ])
    .unwrap();
    let o = s.object("local", &id(3)).unwrap();
    assert_eq!(s.name(&o, K::Name).unwrap().as_deref(), Some("libraryName"));
    assert_eq!(s.owner(&o).unwrap().unwrap().resource(), "local");
    assert_eq!(
        s.qualified_name(&o).unwrap().as_deref(),
        Some("Vehicle::libraryName")
    );
}
#[test]
fn references_do_not_fall_back_to_semantic_id_or_missing_resource() {
    for reference in [
        json!(id(20)),
        json!(format!(
            "sysmllibrary:///aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa#{}",
            id(20)
        )),
    ] {
        let s = snapshot(vec![
            node(1, "PartUsage", None, vec![redefine(2, reference)]),
            node(20, "PartUsage", Some("base"), vec![]),
        ]);
        assert!(matches!(
            s.name(&s.object("local", &id(1)).unwrap(), K::Name),
            Err(Error::Reference(ReferenceError::Unresolved { .. }))
        ));
    }
}
#[test]
fn wrong_type_and_proxy_forms_fail_without_declared_name_fallback() {
    for reference in [json!(wire(20)), json!({"$ref":wire(20)})] {
        let s = snapshot(vec![
            node(1, "PartUsage", None, vec![redefine(2, reference)]),
            node(20, "Package", Some("wrong"), vec![]),
        ]);
        assert!(matches!(
            s.name(&s.object("local", &id(1)).unwrap(), K::Name),
            Err(Error::Reference(_))
        ));
    }
}
#[test]
fn naming_cycles_are_explicit_and_caller_documents_are_immutable() {
    let d = doc(vec![
        node(1, "PartUsage", None, vec![redefine(2, json!(wire(3)))]),
        node(3, "PartUsage", None, vec![redefine(4, json!(wire(1)))]),
    ]);
    let saved = d.to_json().unwrap();
    let s = Snapshot::new(vec![resource("local", d.clone())]).unwrap();
    assert_eq!(
        s.name(&s.object("local", &id(1)).unwrap(), K::Name),
        Err(Error::Cycle)
    );
    assert_eq!(d.to_json().unwrap(), saved);
}
#[test]
fn conjugated_port_names_use_original_name_and_declared_short_name() {
    let mut c = node(1, "ConjugatedPortDefinition", Some("ignored"), vec![]);
    c["data"]["declaredShortName"] = json!("c");
    let mut relation = node(2, "PortConjugation", None, vec![]);
    relation["data"]["originalPortDefinition"] = json!(wire(3));
    c["data"]["ownedRelationship"] = json!([relation]);
    let s = snapshot(vec![c, node(3, "PortDefinition", Some("Port"), vec![])]);
    let o = s.object("local", &id(1)).unwrap();
    assert_eq!(s.name(&o, K::Name).unwrap().as_deref(), Some("~Port"));
    assert_eq!(s.name(&o, K::ShortName).unwrap().as_deref(), Some("c"));
    assert_eq!(s.qualified_name(&o).unwrap().as_deref(), Some("'~Port'"));
}
#[test]
fn foreign_identical_snapshots_and_caller_edits_cannot_retarget_handles() {
    let d = doc(vec![root(vec![])]);
    let a = Snapshot::new(vec![resource("local", d.clone())]).unwrap();
    let b = Snapshot::new(vec![resource("local", d.clone())]).unwrap();
    let o = a.object("local", &id(1)).unwrap();
    assert_eq!(b.owner(&o).unwrap_err(), Error::StaleSnapshot);
    assert_eq!(b.qualified_name(&o), Err(Error::StaleSnapshot));
    let mut editor = Editor::new(d);
    let mut tx = editor.begin();
    tx.rename(&id(1), "changed").unwrap();
    editor.commit(tx).unwrap();
    assert_eq!(a.name(&o, K::Name).unwrap().as_deref(), Some("Vehicle"));
    assert!(matches!(
        a.object("missing", &id(1)),
        Err(Error::MissingObject(_))
    ));
}
#[test]
fn invalid_containment_and_unknown_classes_refuse_snapshot_admission() {
    let mut bad = root(vec![node(
        2,
        "PartUsage",
        Some("invalidRelationship"),
        vec![],
    )]);
    assert!(Snapshot::new(vec![resource("local", doc(vec![bad.clone()]))]).is_err());
    bad["data"]["ownedRelationship"] = json!([]);
    bad["eClass"] = json!("sysml:UnknownClass");
    assert!(Snapshot::new(vec![resource("local", doc(vec![bad]))]).is_err());
}
