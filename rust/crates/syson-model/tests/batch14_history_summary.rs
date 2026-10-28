// SPDX-License-Identifier: EPL-2.0
//! Existing RR20 retention and A borrowed traversal through the same live Editor.
use syson_model::{
    Document, Editor, HistoryAvailability, HistoryLimits, HistoryTruncation, ModelError,
};
const DENSE: &str = include_str!(
    "../../../../porting/rust-parity-evidence/RR18b3a/fixtures/ownership-amplification.json"
);
const CHILD: &str = "child-0511";
const OPAQUE: &str = r#""jointOpaque":{"minus":-0,"exponent":1e+0003,"huge":12345678901234567890123456789012345678901234567890,"marker":{"$serde_json::private::Number":"not-a-number"},"text":"\u0000雪🧭\r\n<&>"}"#;
fn document() -> Document {
    let raw = format!(
        "{},{}{}",
        DENSE.trim_end().strip_suffix('}').unwrap(),
        OPAQUE,
        "}"
    );
    Document::from_json(&raw).unwrap()
}
fn check(editor: &Editor, expected_name: &str) {
    let doc = editor.document();
    let rows = doc.element_summaries();
    assert_eq!(rows.len(), 1025);
    let owner = doc.value()["content"][0]["data"]["elementId"]
        .as_str()
        .unwrap();
    assert_eq!(owner.len(), 65542);
    assert_eq!(
        (
            rows[0].element_id,
            rows[0].class,
            rows[0].name,
            rows[0].owner
        ),
        (owner, "Package", Some("Owner"), None)
    );
    for index in 0..512 {
        let membership = &rows[1 + 2 * index];
        let child = &rows[2 + 2 * index];
        assert_eq!(membership.element_id, format!("membership-{index:04}"));
        assert_eq!(
            (membership.class, membership.owner),
            ("OwningMembership", None)
        );
        assert_eq!(child.element_id, format!("child-{index:04}"));
        assert_eq!((child.class, child.owner), ("PartUsage", Some(owner)));
        assert_eq!(child.owner.unwrap().as_ptr(), owner.as_ptr());
        let source = &doc.value()["content"][0]["data"]["ownedRelationship"][index];
        assert_eq!(membership.resource_id, source["id"].as_str().unwrap());
        assert_eq!(
            child.resource_id,
            source["data"]["ownedRelatedElement"][0]["id"]
                .as_str()
                .unwrap()
        );
        if index == 511 {
            assert_eq!(child.name, Some(expected_name));
        } else {
            assert_eq!(child.name.unwrap(), format!("part{index:04}"));
        }
    }
    let opaque = &doc.value()["jointOpaque"];
    for (name, spelling) in [
        ("minus", "-0"),
        ("exponent", "1e+0003"),
        ("huge", "12345678901234567890123456789012345678901234567890"),
    ] {
        assert_eq!(opaque[name].as_number().unwrap().as_str(), spelling);
    }
    let marker = opaque["marker"].as_object().unwrap();
    assert_eq!(marker.len(), 1);
    assert_eq!(
        marker["$serde_json::private::Number"].as_str(),
        Some("not-a-number")
    );
    assert_eq!(
        doc.value()["jointOpaque"]["text"].as_str().unwrap(),
        "\0雪🧭\r\n<&>"
    );
    let state = editor.history_state();
    assert!(state.undo_entries + state.redo_entries <= state.limits.entries());
    assert!(state.retained_charge_bytes <= state.limits.charge_bytes());
}
fn rename(editor: &mut Editor, name: &str) {
    let mut transaction = editor.begin();
    transaction.rename(CHILD, name).unwrap();
    assert!(editor.commit(transaction).unwrap());
    check(editor, name);
}
#[test]
fn borrowed_dense_identity_and_literal_authority_survive_retention_and_refusals() {
    let mut editor = Editor::with_history_limits(
        document(),
        HistoryLimits::new(2, 128 * 1024 * 1024).unwrap(),
    );
    check(&editor, "part0511");
    let mut stale = editor.begin();
    stale.rename(CHILD, "stale").unwrap();
    let mut foreign = Editor::new(document()).begin();
    foreign.rename(CHILD, "foreign").unwrap();
    rename(&mut editor, "First 雪🧭 &\r\n");
    let first = editor.document().to_json().unwrap();
    rename(&mut editor, "Second");
    let second = editor.document().to_json().unwrap();
    rename(&mut editor, "Third");
    let third = editor.document().to_json().unwrap();
    assert_eq!(editor.history_state().undo_entries, 2);
    assert_eq!(editor.history_state().discarded_undo, 1);
    assert_eq!(
        editor.history_state().last_truncation,
        Some(HistoryTruncation::EntryBudget)
    );
    let state = editor.history_state();
    assert_eq!(editor.commit(stale), Err(ModelError::StaleTransaction));
    assert_eq!(editor.commit(foreign), Err(ModelError::ForeignTransaction));
    let noop = editor.begin();
    assert!(!editor.commit(noop).unwrap());
    let mut cancelled = editor.begin();
    cancelled.rename(CHILD, "cancelled").unwrap();
    cancelled.cancel();
    let mut failed = editor.begin();
    assert!(failed.rename("missing", "bad").is_err());
    assert_eq!(editor.commit(failed), Err(ModelError::FailedTransaction));
    assert_eq!(editor.history_state(), state);
    assert_eq!(editor.document().to_json().unwrap(), third);
    check(&editor, "Third");
    assert!(editor.undo());
    assert_eq!(editor.document().to_json().unwrap(), second);
    check(&editor, "Second");
    assert!(editor.undo());
    assert_eq!(editor.document().to_json().unwrap(), first);
    check(&editor, "First 雪🧭 &\r\n");
    assert!(!editor.undo());
    assert_eq!(
        editor.history_state().undo,
        HistoryAvailability::RetentionBoundary
    );
    assert_eq!(editor.document().to_json().unwrap(), first);
    assert!(editor.redo());
    assert_eq!(editor.document().to_json().unwrap(), second);
    check(&editor, "Second");
    assert!(editor.redo());
    assert_eq!(editor.document().to_json().unwrap(), third);
    check(&editor, "Third");
    assert!(editor.undo());
    rename(&mut editor, "Branch");
    assert!(!editor.redo());
    assert_eq!(editor.history_state().redo, HistoryAvailability::Empty);
    let branch = editor.document().to_json().unwrap();
    editor.set_history_limits(HistoryLimits::new(2, 0).unwrap());
    assert_eq!(editor.document().to_json().unwrap(), branch);
    check(&editor, "Branch");
    rename(&mut editor, "Committed without retention");
    assert_eq!(editor.history_state().undo_entries, 0);
    assert_eq!(
        editor.history_state().last_truncation,
        Some(HistoryTruncation::Disabled)
    );
    assert!(!editor.undo());
    check(&editor, "Committed without retention");
}
