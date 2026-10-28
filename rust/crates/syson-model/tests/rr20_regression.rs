// SPDX-License-Identifier: EPL-2.0
//! Public-API baseline: full candidate setup precedes the retention assertion.
use syson_model::{Document, Editor};
fn fixture(nodes: usize) -> Document {
    let mut content = Vec::with_capacity(nodes);
    for n in 0..nodes {
        content.push(serde_json::json!({"id":format!("10000000-0000-4000-8000-{n:012x}"),"eClass":"sysml:Package","data":{"elementId":format!("20000000-0000-4000-8000-{n:012x}"),"declaredName":format!("Package{n}")}}));
    }
    let value = serde_json::json!({"json":{"version":"1.0","encoding":"utf-8"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":content});
    Document::from_json(&value.to_string()).unwrap()
}
#[test]
fn repeated_commits_do_not_retain_more_than_sixty_four_undo_snapshots() {
    let mut editor = Editor::new(fixture(1000));
    for n in 1..=256 {
        let mut transaction = editor.begin();
        transaction
            .rename(
                "20000000-0000-4000-8000-000000000000",
                &format!("Revision{n}"),
            )
            .unwrap();
        assert!(editor.commit(transaction).unwrap());
    }
    let mut undos = 0;
    while editor.undo() {
        undos += 1;
    }
    eprintln!("RR20 legacy public undo entries retained after256commits={undos}");
    assert!(
        undos <= 64,
        "unbounded history retained {undos} undo snapshots"
    );
}

use syson_model::{
    ContainmentFeature, HistoryAvailability as Available, HistoryLimits, HistoryTruncation,
    ModelError,
};
const ID: &str = "20000000-0000-4000-8000-000000000000";
fn limits(entries: usize, bytes: usize) -> HistoryLimits {
    HistoryLimits::new(entries, bytes).unwrap()
}
fn small(entries: usize) -> Editor {
    Editor::with_history_limits(fixture(1), limits(entries, 128 * 1024 * 1024))
}
fn rename(editor: &mut Editor, name: &str) {
    let mut transaction = editor.begin();
    transaction.rename(ID, name).unwrap();
    assert!(editor.commit(transaction).unwrap());
}
fn name(editor: &Editor) -> &str {
    editor.document().value()["content"][0]["data"]["declaredName"]
        .as_str()
        .unwrap()
}
fn bounded(editor: &Editor) {
    let state = editor.history_state();
    assert!(state.undo_entries + state.redo_entries <= state.limits.entries());
    assert!(state.retained_charge_bytes <= state.limits.charge_bytes());
}
#[test]
fn entry_boundary_evicts_oldest_and_reports_unavailable_undo() {
    let mut editor = small(2);
    for value in ["one", "two", "three"] {
        rename(&mut editor, value);
        bounded(&editor);
    }
    assert_eq!(editor.history_state().discarded_undo, 1);
    assert_eq!(
        editor.history_state().last_truncation,
        Some(HistoryTruncation::EntryBudget)
    );
    assert!(editor.undo());
    assert_eq!(name(&editor), "two");
    bounded(&editor);
    assert!(editor.undo());
    assert_eq!(name(&editor), "one");
    bounded(&editor);
    assert!(!editor.undo());
    assert_eq!(editor.history_state().undo, Available::RetentionBoundary);
    assert!(editor.redo());
    assert_eq!(name(&editor), "two");
    bounded(&editor);
    assert!(editor.redo());
    assert_eq!(name(&editor), "three");
    bounded(&editor);
}
#[test]
fn combined_stacks_evict_oldest_undo_then_farthest_redo() {
    let mut editor = small(4);
    for value in ["one", "two", "three", "four"] {
        rename(&mut editor, value);
    }
    assert!(editor.undo());
    assert!(editor.undo());
    assert_eq!(name(&editor), "two");
    editor.set_history_limits(limits(3, 128 * 1024 * 1024));
    assert_eq!(
        (
            editor.history_state().undo_entries,
            editor.history_state().redo_entries
        ),
        (1, 2)
    );
    editor.set_history_limits(limits(1, 128 * 1024 * 1024));
    bounded(&editor);
    assert_eq!(editor.history_state().discarded_undo, 2);
    assert_eq!(editor.history_state().discarded_redo, 1);
    assert!(!editor.undo());
    assert!(editor.redo());
    assert_eq!(name(&editor), "three");
    assert!(!editor.redo());
    assert_eq!(editor.history_state().redo, Available::RetentionBoundary);
}
#[test]
fn exact_charge_is_retained_and_one_byte_less_refuses_it() {
    let mut editor = small(1);
    rename(&mut editor, "one");
    let exact = editor.history_state().retained_charge_bytes;
    assert!(exact > editor.document().to_json().unwrap().len());
    editor.set_history_limits(limits(1, exact));
    assert_eq!(editor.history_state().retained_charge_bytes, exact);
    assert_eq!(editor.history_state().undo, Available::Available);
    editor.set_history_limits(limits(1, exact - 1));
    assert_eq!(editor.history_state().retained_charge_bytes, 0);
    assert_eq!(
        editor.history_state().last_truncation,
        Some(HistoryTruncation::SnapshotTooLarge)
    );
    assert_eq!(name(&editor), "one");
    assert!(!editor.undo());
}
#[test]
fn zero_limits_disable_retention_without_refusing_document_commits() {
    for policy in [limits(0, 128 * 1024 * 1024), limits(64, 0)] {
        let mut editor = Editor::with_history_limits(fixture(1), policy);
        rename(&mut editor, "one");
        bounded(&editor);
        assert_eq!(
            editor.history_state().last_truncation,
            Some(HistoryTruncation::Disabled)
        );
        assert!(!editor.undo());
        assert_eq!(name(&editor), "one");
    }
    assert!(HistoryLimits::new(65, 1).is_none());
    assert!(HistoryLimits::new(1, 128 * 1024 * 1024 + 1).is_none());
}
#[test]
fn noop_cancel_poison_foreign_and_stale_preserve_history() {
    let mut editor = small(4);
    let mut foreign = small(4).begin();
    foreign.rename(ID, "foreign").unwrap();
    let mut stale = editor.begin();
    stale.rename(ID, "stale").unwrap();
    rename(&mut editor, "one");
    assert!(editor.undo());
    let snapshot = editor.document().clone();
    let state = editor.history_state();
    assert_eq!(editor.commit(foreign), Err(ModelError::ForeignTransaction));
    // Document equality after undo does not revive a transaction's revision.
    assert_eq!(editor.commit(stale), Err(ModelError::StaleTransaction));
    let noop = editor.begin();
    assert!(!editor.commit(noop).unwrap());
    let mut cancelled = editor.begin();
    cancelled.rename(ID, "cancelled").unwrap();
    cancelled.cancel();
    let mut poisoned = editor.begin();
    assert!(poisoned.rename("missing", "bad").is_err());
    assert_eq!(editor.commit(poisoned), Err(ModelError::FailedTransaction));
    assert_eq!(editor.document(), &snapshot);
    assert_eq!(editor.history_state(), state);
    assert!(editor.redo());
    assert_eq!(name(&editor), "one");
    assert!(editor.undo());
    rename(&mut editor, "branch");
    assert!(!editor.redo());
    assert_eq!(editor.history_state().redo, Available::Empty);
}
#[test]
fn policy_change_does_not_stale_a_document_transaction() {
    let mut editor = small(4);
    rename(&mut editor, "one");
    let mut transaction = editor.begin();
    transaction.rename(ID, "two").unwrap();
    editor.set_history_limits(limits(1, 128 * 1024 * 1024));
    assert!(editor.commit(transaction).unwrap());
    bounded(&editor);
    assert!(editor.undo());
    assert_eq!(name(&editor), "one");
}
fn fragment(opaque: serde_json::Value) -> serde_json::Value {
    let mut node = serde_json::json!({"id":"30000000-0000-4000-8000-000000000001","eClass":"sysml:OwningMembership","data":{"elementId":"40000000-0000-4000-8000-000000000001"}});
    node["data"]["opaque"] = opaque;
    node
}
#[test]
fn failed_publication_does_not_change_live_history() {
    let mut editor = small(4);
    rename(&mut editor, "one");
    assert!(editor.undo());
    let state = editor.history_state();
    let original = editor.document().clone();
    let mut opaque = serde_json::Value::Null;
    for _ in 0..128 {
        opaque = serde_json::Value::Array(vec![opaque]);
    }
    let mut transaction = editor.begin();
    transaction
        .attach_preallocated_subtree(ID, ContainmentFeature::OwnedRelationship, fragment(opaque))
        .unwrap();
    assert!(matches!(
        editor.commit(transaction),
        Err(ModelError::JsonAdmission(_))
    ));
    assert_eq!(editor.document(), &original);
    assert_eq!(editor.history_state(), state);
    assert!(editor.redo());
    assert_eq!(name(&editor), "one");
}
#[test]
fn owned_capacity_is_charged_and_numeric_lexemes_survive_moves() {
    let mut rows = Vec::with_capacity(100_000);
    rows.push(serde_json::Value::Null);
    let mut number = String::with_capacity(100_000);
    number.push_str("-0");
    let mut opaque = serde_json::Map::new();
    opaque.insert("rows".into(), serde_json::Value::Array(rows));
    opaque.insert(
        "number".into(),
        serde_json::Value::Number(serde_json::Number::from_string_unchecked(number)),
    );
    let mut marker = serde_json::Map::new();
    marker.insert(
        "$serde_json::private::Number".into(),
        serde_json::Value::String("123".into()),
    );
    opaque.insert("marker".into(), serde_json::Value::Object(marker.clone()));
    for token in ["1.2300", "1e+0002", "18446744073709551616"] {
        opaque.insert(
            token.into(),
            serde_json::Value::Number(serde_json::Number::from_string_unchecked(token.into())),
        );
    }
    let mut editor = small(4);
    let mut transaction = editor.begin();
    transaction
        .attach_preallocated_subtree(
            ID,
            ContainmentFeature::OwnedRelationship,
            fragment(serde_json::Value::Object(opaque)),
        )
        .unwrap();
    assert!(editor.commit(transaction).unwrap());
    let saved = editor.document().clone();
    rename(&mut editor, "one");
    let charge = editor.history_state().retained_charge_bytes;
    assert!(charge >= 100_000 * std::mem::size_of::<serde_json::Value>());
    assert!(editor.undo());
    assert_eq!(editor.document(), &saved);
    let data =
        &editor.document().value()["content"][0]["data"]["ownedRelationship"][0]["data"]["opaque"];
    assert_eq!(data["number"].as_number().unwrap().as_str(), "-0");
    assert_eq!(data["marker"].as_object().unwrap(), &marker);
    for token in ["1.2300", "1e+0002", "18446744073709551616"] {
        assert_eq!(data[token].as_number().unwrap().as_str(), token);
    }
    assert!(editor.redo());
    assert_eq!(name(&editor), "one");
    bounded(&editor);
}
#[test]
fn undo_with_larger_current_drops_oversized_redo_without_losing_restored_data() {
    let mut reference = small(4);
    rename(&mut reference, "one");
    let bytes = reference.history_state().retained_charge_bytes + 1024;
    let mut editor = Editor::with_history_limits(fixture(1), limits(4, bytes));
    let original = editor.document().clone();
    let mut transaction = editor.begin();
    transaction
        .attach_preallocated_subtree(
            ID,
            ContainmentFeature::OwnedRelationship,
            fragment(serde_json::Value::Array(vec![
                serde_json::Value::Null;
                1000
            ])),
        )
        .unwrap();
    assert!(editor.commit(transaction).unwrap());
    bounded(&editor);
    assert_eq!(editor.history_state().undo_entries, 1);
    assert!(editor.undo());
    bounded(&editor);
    assert_eq!(editor.document(), &original);
    assert_eq!(editor.history_state().redo, Available::RetentionBoundary);
    assert_eq!(
        editor.history_state().last_truncation,
        Some(HistoryTruncation::SnapshotTooLarge)
    );
    assert!(!editor.redo());
}
