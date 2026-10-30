// SPDX-License-Identifier: EPL-2.0
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};
use syson_model::{Document, Editor, ModelError, NodeIds};
use syson_text_import::{Error, Importer};
fn importer() -> &'static Importer {
    static I: OnceLock<Importer> = OnceLock::new();
    I.get_or_init(|| Importer::pinned().unwrap())
}
fn ids(n: u32) -> NodeIds {
    NodeIds::new(
        &format!("10000000-0000-4000-8000-{n:012x}"),
        &format!("20000000-0000-4000-8000-{n:012x}"),
    )
    .unwrap()
}
fn supply() -> impl Iterator<Item = NodeIds> {
    (1..100).map(ids)
}
fn cancel() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}
fn named(doc: &Document, name: &str) -> String {
    doc.elements()
        .iter()
        .find(|e| e.name.as_deref() == Some(name))
        .unwrap()
        .element_id
        .clone()
}
fn type_target(doc: &Document, usage: &str) -> String {
    doc.element(usage).unwrap()["data"]["ownedRelationship"][0]["data"]["type"]
        .as_str()
        .unwrap()
        .into()
}
#[test]
fn source_stages_and_commits_one_model_with_distinct_ids_and_actual_linked_types() {
    let mut e = Editor::new(Document::empty());
    let before = e.document().clone();
    let s = importer()
        .stage(
            e.begin(),
            "package Vehicle { part def Wheel; part wheel: Wheel; }",
            None,
            supply(),
            cancel(),
        )
        .unwrap();
    assert_eq!(e.document(), &before);
    assert_eq!(s.roots(), &[ids(1).element.clone()]);
    assert_eq!(s.allocations(), 6);
    let definition = named(s.document(), "Wheel");
    let usage = named(s.document(), "wheel");
    assert_eq!(type_target(s.document(), &usage), definition);
    let typing = &s.document().element(&usage).unwrap()["data"]["ownedRelationship"][0];
    assert_eq!(typing["eClass"], "sysml:FeatureTyping");
    assert_eq!(typing["data"]["typedFeature"], usage);
    assert_ne!(
        typing["data"]["type"],
        s.document().element(&definition).unwrap()["id"]
    );
    for node in s.document().elements() {
        assert_ne!(node.resource_id, node.element_id);
    }
    assert!(s.commit(&mut e).unwrap());
    let imported = e.document().clone();
    assert!(e.undo());
    assert_eq!(e.document(), &before);
    assert!(!e.undo());
    assert!(e.redo());
    assert_eq!(e.document(), &imported);
}
#[test]
fn forward_and_cross_package_references_use_actual_f13() {
    let e = Editor::new(Document::empty());
    let s=importer().stage(e.begin(),"package V { part a: Lib::Wheel; part b: Local; part def Local; } package Lib { part def Wheel; }",None,supply(),cancel()).unwrap();
    for (usage, def) in [("a", "Wheel"), ("b", "Local")] {
        assert_eq!(
            type_target(s.document(), &named(s.document(), usage)),
            named(s.document(), def)
        );
    }
}
#[test]
fn constructor_selects_definition_body_feature_membership() {
    let e = Editor::new(Document::empty());
    let s = importer()
        .stage(
            e.begin(),
            "package P { part def D { part child; } }",
            None,
            supply(),
            cancel(),
        )
        .unwrap();
    assert_eq!(
        s.document().element(&named(s.document(), "D")).unwrap()["data"]["ownedRelationship"][0]["eClass"],
        "sysml:FeatureMembership"
    );
}
#[test]
fn explicit_existing_parent_attaches_parts_without_injecting_existing_symbols() {
    let mut e = Editor::new(Document::empty());
    let mut tx = e.begin();
    tx.create_package(None, ids(90), None, "Existing").unwrap();
    e.commit(tx).unwrap();
    let before = e.document().clone();
    let s = importer()
        .stage(
            e.begin(),
            "part def D; part item:D;",
            Some(&ids(90).element),
            supply(),
            cancel(),
        )
        .unwrap();
    assert_eq!(e.document(), &before);
    s.commit(&mut e).unwrap();
    assert_eq!(
        e.document()
            .elements()
            .iter()
            .filter(|n| n.name.as_deref() == Some("Existing"))
            .count(),
        1
    );
    assert!(matches!(
        importer().stage(
            e.begin(),
            "part item:D;",
            Some(&ids(90).element),
            (30..50).map(ids),
            cancel()
        ),
        Err(Error::Link(syson_linker::Error::Unresolved(_)))
    ));
}
#[test]
fn pinned_java_quote_stripping_is_distinct_from_ast_lookup() {
    let e = Editor::new(Document::empty());
    let s = importer()
        .stage(
            e.begin(),
            "package 'Road vehicle' { part def 'W\\'heel'; part axle:'W\\'heel'; }",
            None,
            supply(),
            cancel(),
        )
        .unwrap();
    assert_eq!(
        type_target(s.document(), &named(s.document(), "axle")),
        named(s.document(), "W\\heel")
    );
    assert!(!s.document().to_json().unwrap().contains("'Road vehicle'"));
}
#[test]
fn malformed_suffix_and_unsupported_input_leave_data_and_history_unchanged() {
    let mut e = Editor::new(Document::empty());
    let before = e.document().clone();
    for source in [
        "package P; !",
        "package P {",
        "action A;",
        "package 'bad",
        "package P; @",
    ] {
        assert!(
            matches!(
                importer().stage(e.begin(), source, None, supply(), cancel()),
                Err(Error::Parse(_))
            ),
            "{source}"
        );
        assert_eq!(e.document(), &before);
        assert!(!e.undo());
    }
}
#[test]
fn missing_wrong_type_duplicate_and_outer_fallback_references_refuse_atomically() {
    let e = Editor::new(Document::empty());
    let before = e.document().clone();
    for source in [
        "package P {part x:Missing;}",
        "package P {package T;part x:T;}",
        "package P {part def T;part def T;}",
        "package P {part def T;package Q;part x:Q::T;}",
    ] {
        assert!(
            matches!(
                importer().stage(e.begin(), source, None, supply(), cancel()),
                Err(Error::Link(_))
            ),
            "{source}"
        );
        assert_eq!(e.document(), &before);
    }
}
#[test]
fn missing_parent_and_unnamed_constructs_are_explicitly_unsupported() {
    let e = Editor::new(Document::empty());
    for source in ["part def T;", "part x;", "package;", "package P {part;}"] {
        assert!(
            matches!(
                importer().stage(e.begin(), source, None, supply(), cancel()),
                Err(Error::Unsupported(_))
            ),
            "{source}"
        );
    }
}
#[test]
fn exhausted_colliding_and_late_typing_identities_publish_no_prefix() {
    let e = Editor::new(Document::empty());
    let before = e.document().clone();
    let source = "package P {part def T;part x:T;}";
    assert_eq!(
        importer()
            .stage(e.begin(), source, None, vec![ids(1)], cancel())
            .unwrap_err(),
        Error::IdentitiesExhausted
    );
    for given in [
        vec![ids(1), ids(2), ids(1), ids(4), ids(5), ids(6)],
        vec![ids(1), ids(2), ids(3), ids(4), ids(5), ids(1)],
    ] {
        assert!(matches!(
            importer().stage(e.begin(), source, None, given, cancel()),
            Err(Error::Model(ModelError::DuplicateId(_)))
        ));
    }
    assert_eq!(e.document(), &before);
}
#[test]
fn explicit_and_mid_creation_cancellation_drop_the_whole_owned_transaction() {
    let mut e = Editor::new(Document::empty());
    let before = e.document().clone();
    let s = importer()
        .stage(e.begin(), "package P;", None, supply(), cancel())
        .unwrap();
    s.cancel();
    assert_eq!(e.document(), &before);
    assert!(!e.undo());
    let flag = cancel();
    let mut count = 0;
    let iter = supply().inspect(|_| {
        count += 1;
        if count == 4 {
            flag.store(true, Ordering::Relaxed);
        }
    });
    assert_eq!(
        importer()
            .stage(
                e.begin(),
                "package P {part def T;part x:T;}",
                None,
                iter,
                flag.clone()
            )
            .unwrap_err(),
        Error::Cancelled
    );
    assert_eq!(e.document(), &before);
    assert_eq!(
        importer()
            .stage(e.begin(), "package P;", None, supply(), flag)
            .unwrap_err(),
        Error::Cancelled
    );
}
#[test]
fn cancellation_after_staging_prevents_commit() {
    let mut e = Editor::new(Document::empty());
    let before = e.document().clone();
    let flag = cancel();
    let s = importer()
        .stage(e.begin(), "package P;", None, supply(), flag.clone())
        .unwrap();
    flag.store(true, Ordering::Relaxed);
    assert_eq!(s.commit(&mut e), Err(Error::Cancelled));
    assert_eq!(e.document(), &before);
}
#[test]
fn stale_and_foreign_editor_commit_keep_current_documents() {
    let mut e = Editor::new(Document::empty());
    let s = importer()
        .stage(e.begin(), "package P;", None, supply(), cancel())
        .unwrap();
    let mut tx = e.begin();
    tx.create_package(None, ids(90), None, "Concurrent")
        .unwrap();
    e.commit(tx).unwrap();
    let before = e.document().clone();
    assert_eq!(
        s.commit(&mut e),
        Err(Error::Model(ModelError::StaleTransaction))
    );
    assert_eq!(e.document(), &before);
    let s = importer()
        .stage(e.begin(), "package Q;", None, supply(), cancel())
        .unwrap();
    let mut foreign = Editor::new(before.clone());
    assert_eq!(
        s.commit(&mut foreign),
        Err(Error::Model(ModelError::ForeignTransaction))
    );
    assert_eq!(foreign.document(), &before);
}
#[test]
fn empty_input_is_noop_and_parser_limits_remain_authoritative() {
    let mut e = Editor::new(Document::empty());
    let s = importer()
        .stage(e.begin(), "  ", None, supply(), cancel())
        .unwrap();
    assert!(s.roots().is_empty());
    assert_eq!(s.allocations(), 0);
    assert!(!s.commit(&mut e).unwrap());
    assert_eq!(
        importer()
            .stage(e.begin(), &" ".repeat(65_537), None, supply(), cancel())
            .unwrap_err(),
        Error::Parse(syson_ast_adapter::Error::Limit)
    );
}
