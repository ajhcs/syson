// SPDX-License-Identifier: EPL-2.0
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use syson_editor::{Diagram, Node, Point, prepare_rotation};
use syson_model::{Document, Editor, NodeIds};
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
fn output() -> PathBuf {
    // The test runner must provide a verified build volume; no root-disk fallback.
    let root = std::env::var_os("SYSON_TEST_DIR")
        .expect("Set SYSON_TEST_DIR on the approved build volume");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from(root).join(format!("workflow-{}-{unique}", std::process::id()))
}
#[test]
fn create_type_cancel_undo_save_restart_and_rotate() {
    let mut editor = Editor::new(Document::empty());
    let mut tx = editor.begin();
    tx.create_package(None, ids(1), None, "Vehicle").unwrap();
    tx.create_part_definition(&eid(1), ids(2), ids(3), "Wheel")
        .unwrap();
    tx.create_part_usage(&eid(1), ids(4), ids(5), "wheel")
        .unwrap();
    tx.type_part_usage(&eid(4), &eid(2), ids(6)).unwrap();
    editor.commit(tx).unwrap();
    let original = editor.document().clone();
    let mut cancelled = editor.begin();
    cancelled.rename(&eid(4), "cancelled").unwrap();
    cancelled.cancel();
    assert_eq!(editor.document(), &original);
    let mut renamed = editor.begin();
    renamed.rename(&eid(4), "frontWheel").unwrap();
    editor.commit(renamed).unwrap();
    assert!(editor.undo());
    assert_eq!(editor.document(), &original);
    assert!(editor.redo());
    let root = output();
    let store = SnapshotStore::open(&root).unwrap();
    store.save_new("revision-1", editor.document()).unwrap();
    assert!(store.save_new("revision-1", &original).is_err());
    let recovered = SnapshotStore::open(&root)
        .unwrap()
        .load("revision-1")
        .unwrap();
    assert_eq!(&recovered, editor.document());
    let projected = recovered.projection().unwrap();
    let local = projected
        .model
        .find_by_name_and_type("Vehicle::frontWheel", |e| e.class == "PartUsage")
        .unwrap()
        .unwrap();
    assert_eq!(projected.stable_id(local), Some(eid(4).as_str()));
    let diagram: Diagram<String, ()> = Diagram {
        nodes: vec![Node {
            id: eid(4),
            position: Point { x: 10., y: 20. },
            width: Some(100.),
            height: Some(60.),
            resized_by_user: false,
            connection_handles: vec![],
            payload: "frontWheel".into(),
        }],
        edges: vec![],
    };
    let rotated = prepare_rotation(&diagram, &[eid(4)]);
    assert_eq!(rotated.nodes[0].position, Point { x: 30., y: 0. });
    assert_eq!(rotated.nodes[0].width, Some(60.));
    assert!(!syson_persistence::should_persist_in_project_download(
        Some("sysmllibrary:/Actions.sysml")
    ));
    assert!(syson_persistence::should_persist_in_project_download(Some(
        "memory:/vehicle.sysml"
    )));
}
#[test]
fn corrupt_partial_and_invalid_keys_do_not_replace_snapshot() {
    let root = output();
    let store = SnapshotStore::open(&root).unwrap();
    store.save_new("good", &Document::empty()).unwrap();
    fs::write(root.join("partial.emf.staged"), b"{unfinished").unwrap();
    fs::write(root.join("corrupt.emf.json"), b"{unfinished").unwrap();
    assert!(store.load("partial").is_err());
    assert!(store.load("corrupt").is_err());
    assert_eq!(store.load("good").unwrap(), Document::empty());
    for key in ["", "../escape", "a/b", "a\\b", "."] {
        assert!(store.load(key).is_err());
    }
}
