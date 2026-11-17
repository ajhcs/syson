// SPDX-License-Identifier: EPL-2.0
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use syson_model::{Document, Editor, NodeIds};
use syson_workflow::SnapshotStore;

#[test]
fn long_unicode_package_name_survives_reopen() {
    // Keep this value in sync with long-unicode-name in porting/oracle/corpus.json.
    let name = format!("Long <&> 雪🧭 {} END", "x".repeat(4096));
    let ids = NodeIds::new(
        "00000000-0000-4000-8000-000000000001",
        "10000000-0000-4000-8000-000000000001",
    )
    .unwrap();
    let element_id = ids.element.clone();
    let mut editor = Editor::new(Document::empty());
    let mut transaction = editor.begin();
    transaction.create_package(None, ids, None, &name).unwrap();
    editor.commit(transaction).unwrap();

    let root = PathBuf::from(
        std::env::var_os("SYSON_TEST_DIR")
            .expect("Set SYSON_TEST_DIR on the approved build volume"),
    );
    let root = root.canonicalize().unwrap();
    assert!(root.starts_with("/workspace"));
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = root.join(format!("long-unicode-{}-{nonce}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    {
        let store = SnapshotStore::open(&directory).unwrap();
        store.save_new("project", editor.document()).unwrap();
    }
    let reopened = SnapshotStore::open(&directory).unwrap();
    let document = reopened.load("project").unwrap();
    let package = document
        .element(&element_id)
        .expect("package identity survives");
    let saved_name = package["data"]["declaredName"]
        .as_str()
        .expect("package name survives");
    assert_eq!(saved_name.as_bytes(), name.as_bytes());
}
