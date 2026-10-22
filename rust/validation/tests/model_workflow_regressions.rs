// SPDX-License-Identifier: EPL-2.0
//! Independently authored local API regressions; no upstream parity claim.
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use syson_model::{Document, Editor, ModelError, NodeIds};
use syson_workflow::SnapshotStore;

fn ids(n: u32) -> NodeIds {
    NodeIds::new(
        &format!("00000000-0000-4000-8000-{n:012x}"),
        &format!("10000000-0000-4000-8000-{n:012x}"),
    )
    .unwrap()
}
fn eid(n: u32) -> String {
    ids(n).element
}
fn populated() -> Editor {
    let mut editor = Editor::new(Document::empty());
    let mut tx = editor.begin();
    tx.create_package(None, ids(1), None, "Vehicle").unwrap();
    editor.commit(tx).unwrap();
    editor
}
fn directory(label: &str) -> PathBuf {
    let root = std::env::var_os("SYSON_TEST_DIR")
        .expect("Set SYSON_TEST_DIR to an approved build directory");
    PathBuf::from(root).join(format!(
        "independent-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
#[test]
fn transaction_cannot_commit_to_another_identical_editor() {
    let source = Editor::new(Document::empty());
    let mut destination = Editor::new(Document::empty());
    let mut tx = source.begin();
    tx.create_package(None, ids(1), None, "SourceOnly").unwrap();
    assert!(
        destination.commit(tx).is_err(),
        "foreign editor transaction was accepted"
    );
    assert_eq!(destination.document(), &Document::empty());
    assert!(!destination.undo());
}
#[test]
fn aba_undo_cannot_revive_an_old_transaction() {
    let mut editor = populated();
    let baseline = editor.document().clone();
    let mut stale = editor.begin();
    stale.rename(&eid(1), "stale").unwrap();
    let mut live = editor.begin();
    live.rename(&eid(1), "changed").unwrap();
    editor.commit(live).unwrap();
    assert!(editor.undo());
    assert_eq!(editor.document(), &baseline);
    assert_eq!(editor.commit(stale), Err(ModelError::StaleTransaction));
    assert!(editor.redo());
}
#[test]
fn no_op_transaction_preserves_redo_branch() {
    let mut editor = populated();
    let complete = editor.document().clone();
    assert!(editor.undo());
    let tx = editor.begin();
    assert!(!editor.commit(tx).unwrap());
    assert!(editor.redo());
    assert_eq!(editor.document(), &complete);
}
#[test]
fn failed_operation_preserves_last_staged_value_and_committed_history() {
    let mut editor = populated();
    let baseline = editor.document().clone();
    let mut tx = editor.begin();
    tx.rename(&eid(1), "staged").unwrap();
    let staged = tx.document().clone();
    assert!(tx.create_package(None, ids(1), None, "duplicate").is_err());
    assert_eq!(tx.document(), &staged);
    assert_eq!(editor.commit(tx), Err(ModelError::FailedTransaction));
    assert_eq!(editor.document(), &baseline);
    assert!(editor.undo());
    assert!(editor.redo());
    assert_eq!(editor.document(), &baseline);
}
#[test]
fn malformed_envelopes_and_containment_return_errors_without_panicking() {
    for input in [
        "null",
        "[]",
        "false",
        "{\"json\":false}",
        r#"{"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[null]}"#,
        r#"{"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{"id":"a","eClass":"sysml:Package","data":{"elementId":"b","ownedRelationship":false}}]}"#,
        r#"{"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{"id":"a","eClass":"sysml:Package","data":{"elementId":"b","ownedRelationship":[42]}}]}"#,
    ] {
        let result = std::panic::catch_unwind(|| Document::from_json(input));
        assert!(result.is_ok(), "input caused a panic: {input}");
        assert!(result.unwrap().is_err());
    }
}
#[test]
fn duplicate_snapshot_save_preserves_committed_bytes() {
    let root = directory("no-clobber");
    let store = SnapshotStore::open(&root).unwrap();
    let before = populated();
    let path = store.save_new("saved", before.document()).unwrap();
    let bytes = fs::read(&path).unwrap();
    assert!(store.save_new("saved", &Document::empty()).is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(store.load("saved").unwrap(), *before.document());
    assert!(root.join("saved.emf.staged").exists());
}
#[test]
fn interrupted_staging_remains_unpublished_and_other_keys_still_work() {
    let root = directory("interrupted");
    let store = SnapshotStore::open(&root).unwrap();
    fs::write(root.join("interrupted.emf.staged"), "{unfinished").unwrap();
    assert!(store.load("interrupted").is_err());
    assert!(store.save_new("interrupted", &Document::empty()).is_err());
    assert!(!root.join("interrupted.emf.json").exists());
    store.save_new("next", &Document::empty()).unwrap();
    assert_eq!(store.load("next").unwrap(), Document::empty());
}
