// SPDX-License-Identifier: EPL-2.0
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use syson_metamodel::ClassifierId;
use syson_model::{Document, Editor};
use syson_references::{FragmentIdentity, Registry, Resource};
use syson_storage::{Change, Error, Key, Store};
static NEXT: AtomicU64 = AtomicU64::new(1);
const NODE: &str = "10000000-0000-4000-8000-000000000001";
const ELEMENT: &str = "20000000-0000-4000-8000-000000000001";
fn key(project: u8, document: u8) -> Key {
    Key {
        project: [project; 16],
        document: [document; 16],
    }
}
fn path() -> PathBuf {
    let scratch =
        PathBuf::from(std::env::var_os("TMPDIR").expect("Coordinator must set workspace TMPDIR"));
    assert!(scratch.is_absolute() && scratch.starts_with("/workspace"));
    let folder = scratch.join(format!(
        "f27-test-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&folder).unwrap();
    folder.join("store.sqlite3")
}
fn document(name: &str) -> Document {
    Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{"id":NODE,"eClass":"sysml:Package","data":{"elementId":ELEMENT,"declaredName":name,"largeInteger":1234567890123456789u64}}]}).to_string()).unwrap()
}
fn create(store: &Store, key: Key, document: &Document, read_only: bool) {
    store
        .transact(
            &[Change::Create {
                key,
                name: "selected document",
                document,
                read_only,
            }],
            1048576,
        )
        .unwrap();
}
#[test]
fn create_reload_reopen_preserves_all_identity_layers_and_actual_reference_lookup() {
    let file = path();
    let store = Store::create(&file).unwrap();
    let input = document("Vehicle");
    let k = key(1, 2);
    create(&store, k, &input, false);
    let loaded = store.load(k).unwrap().unwrap();
    assert_eq!(loaded.key(), k);
    assert_eq!(loaded.document().value(), input.value());
    assert!(!loaded.is_read_only());
    let reopened = Store::open(&file).unwrap();
    let loaded = reopened.load(k).unwrap().unwrap();
    assert_eq!(loaded.document().element(ELEMENT).unwrap()["id"], NODE);
    let registry = Registry::new(vec![Resource {
        key: "selected-resource".into(),
        document: loaded.document().clone(),
        fragments: FragmentIdentity::SemanticElementId,
    }])
    .unwrap();
    let target = registry
        .resolve(
            "selected-resource",
            Some(&json!(NODE)),
            ClassifierId::Package,
        )
        .unwrap()
        .unwrap();
    assert_eq!(target.node_id(), NODE);
    assert_eq!(target.element_id(), ELEMENT);
    assert!(
        registry
            .resolve(
                "selected-resource",
                Some(&json!(ELEMENT)),
                ClassifierId::Package
            )
            .is_err()
    );
}
#[test]
fn actual_reconciliation_noop_and_content_rebuild_control_revision() {
    let file = path();
    let store = Store::create(&file).unwrap();
    let input = document("Vehicle");
    let k = key(3, 4);
    create(&store, k, &input, false);
    let before = store.load(k).unwrap().unwrap();
    store
        .transact(
            &[Change::Update {
                before: &before,
                name: before.name(),
                document: &input,
            }],
            1048576,
        )
        .unwrap();
    assert_eq!(
        store.load(k).unwrap().unwrap().revision(),
        before.revision()
    );
    let mut editor = Editor::new(input);
    let mut tx = editor.begin();
    tx.rename(ELEMENT, "Renamed").unwrap();
    editor.commit(tx).unwrap();
    store
        .transact(
            &[Change::Update {
                before: &before,
                name: "new name",
                document: editor.document(),
            }],
            1048576,
        )
        .unwrap();
    let after = store.load(k).unwrap().unwrap();
    assert!(after.revision() > before.revision());
    assert_eq!(after.name(), "new name");
    assert_eq!(after.document().value(), editor.document().value());
    assert_eq!(
        store.transact(&[Change::Delete { before: &before }], 1048576),
        Err(Error::Refused("stale".into()))
    );
    assert_eq!(store.load(k).unwrap().unwrap().revision(), after.revision());
}
#[test]
fn delete_is_persistent_and_recreate_does_not_reuse_observation_revision() {
    let file = path();
    let store = Store::create(&file).unwrap();
    let k = key(5, 6);
    let input = document("old");
    create(&store, k, &input, false);
    let old = store.load(k).unwrap().unwrap();
    store
        .transact(&[Change::Delete { before: &old }], 1048576)
        .unwrap();
    assert!(Store::open(&file).unwrap().load(k).unwrap().is_none());
    create(&store, k, &document("new"), false);
    let new = store.load(k).unwrap().unwrap();
    assert!(new.revision() > old.revision());
    assert_eq!(
        store.transact(&[Change::Delete { before: &old }], 1048576),
        Err(Error::Refused("stale".into()))
    );
    assert_eq!(
        store.load(k).unwrap().unwrap().document().value(),
        new.document().value()
    );
}
#[test]
fn stored_read_only_refuses_noop_update_and_delete() {
    let store = Store::create(path()).unwrap();
    let k = key(7, 8);
    let input = document("library");
    create(&store, k, &input, true);
    let before = store.load(k).unwrap().unwrap();
    for change in [
        Change::Update {
            before: &before,
            name: before.name(),
            document: &input,
        },
        Change::Delete { before: &before },
    ] {
        assert_eq!(
            store.transact(&[change], 1048576),
            Err(Error::Refused("read-only".into()))
        );
        let after = store.load(k).unwrap().unwrap();
        assert!(after.is_read_only());
        assert_eq!(after.revision(), before.revision());
    }
}
#[test]
fn late_sql_duplicate_rolls_back_prior_insert_and_clock_update() {
    let store = Store::create(path()).unwrap();
    let a = key(9, 10);
    let b = key(9, 11);
    let input = document("Vehicle");
    create(&store, a, &input, false);
    let revision = store.load(a).unwrap().unwrap().revision();
    assert_eq!(
        store.transact(
            &[
                Change::Create {
                    key: b,
                    name: "first",
                    document: &input,
                    read_only: false
                },
                Change::Create {
                    key: a,
                    name: "duplicate",
                    document: &input,
                    read_only: false
                }
            ],
            1048576
        ),
        Err(Error::Refused("sql".into()))
    );
    assert!(store.load(b).unwrap().is_none());
    assert_eq!(store.load(a).unwrap().unwrap().revision(), revision);
    create(&store, b, &input, false);
    assert_eq!(store.load(b).unwrap().unwrap().revision(), revision + 1);
}
#[test]
fn late_stale_revision_rolls_back_an_earlier_update() {
    let store = Store::create(path()).unwrap();
    let a = key(12, 13);
    let b = key(12, 14);
    let input = document("Vehicle");
    create(&store, a, &input, false);
    create(&store, b, &input, false);
    let first = store.load(a).unwrap().unwrap();
    let stale = store.load(b).unwrap().unwrap();
    store
        .transact(
            &[Change::Update {
                before: &stale,
                name: "new b",
                document: &input,
            }],
            1048576,
        )
        .unwrap();
    assert_eq!(
        store.transact(
            &[
                Change::Update {
                    before: &first,
                    name: "must rollback",
                    document: &input
                },
                Change::Delete { before: &stale }
            ],
            1048576
        ),
        Err(Error::Refused("stale".into()))
    );
    let after = store.load(a).unwrap().unwrap();
    assert_eq!(after.name(), first.name());
    assert_eq!(after.revision(), first.revision());
}
#[test]
fn late_read_only_failure_rolls_back_an_earlier_delete() {
    let file = path();
    let store = Store::create(&file).unwrap();
    let a = key(15, 16);
    let b = key(15, 17);
    let input = document("Vehicle");
    create(&store, a, &input, false);
    create(&store, b, &input, true);
    let writable = store.load(a).unwrap().unwrap();
    let readonly = store.load(b).unwrap().unwrap();
    assert_eq!(
        store.transact(
            &[
                Change::Delete { before: &writable },
                Change::Delete { before: &readonly }
            ],
            1048576
        ),
        Err(Error::Refused("read-only".into()))
    );
    assert_eq!(
        Store::open(&file)
            .unwrap()
            .load(a)
            .unwrap()
            .unwrap()
            .revision(),
        writable.revision()
    );
}
#[test]
fn real_serializer_writer_failure_prevents_all_sql_execution() {
    let file = path();
    let store = Store::create(&file).unwrap();
    let small = Document::empty();
    let large = document(&"x".repeat(2000));
    let before = fs::read(&file).unwrap();
    let budget = small.value().to_string().len();
    assert_eq!(
        store.transact(
            &[
                Change::Create {
                    key: key(18, 19),
                    name: "serialized",
                    document: &small,
                    read_only: false
                },
                Change::Create {
                    key: key(18, 20),
                    name: "fails",
                    document: &large,
                    read_only: false
                }
            ],
            budget
        ),
        Err(Error::Serialization)
    );
    assert_eq!(fs::read(&file).unwrap(), before);
    assert!(store.load(key(18, 19)).unwrap().is_none());
    assert!(store.load(key(18, 20)).unwrap().is_none());
}
#[test]
fn foreign_store_observation_cannot_be_replayed_even_for_identical_database() {
    let file = path();
    let store = Store::create(&file).unwrap();
    let k = key(21, 22);
    let input = document("Vehicle");
    create(&store, k, &input, false);
    let before = store.load(k).unwrap().unwrap();
    let other = Store::open(&file).unwrap();
    assert_eq!(
        other.transact(&[Change::Delete { before: &before }], 1048576),
        Err(Error::ForeignObservation)
    );
    let own = other.load(k).unwrap().unwrap();
    other
        .transact(&[Change::Delete { before: &own }], 1048576)
        .unwrap();
    assert!(store.load(k).unwrap().is_none());
}
#[test]
fn equal_document_and_element_ids_in_separate_projects_never_alias() {
    let store = Store::create(path()).unwrap();
    let a = key(23, 24);
    let b = key(25, 24);
    create(&store, a, &document("ProjectA"), false);
    create(&store, b, &document("ProjectB"), false);
    assert_eq!(
        store
            .load(a)
            .unwrap()
            .unwrap()
            .document()
            .element(ELEMENT)
            .unwrap()["data"]["declaredName"],
        "ProjectA"
    );
    assert_eq!(
        store
            .load(b)
            .unwrap()
            .unwrap()
            .document()
            .element(ELEMENT)
            .unwrap()["data"]["declaredName"],
        "ProjectB"
    );
}
#[test]
fn existing_or_foreign_paths_are_never_overwritten_and_reads_do_not_create() {
    let file = path();
    fs::write(&file, b"").unwrap();
    assert!(matches!(Store::create(&file), Err(Error::Io)));
    let before = fs::read(&file).unwrap();
    assert!(matches!(Store::open(&file), Err(Error::Refused(_))));
    assert_eq!(fs::read(&file).unwrap(), before);
    let missing = path();
    assert!(Store::open(&missing).is_err());
    assert!(!missing.exists());
    assert!(Store::create("relative.sqlite3").is_err());
}
#[test]
fn bounded_batches_and_documents_refuse_before_mutating_the_database() {
    let file = path();
    let store = Store::create(&file).unwrap();
    let input = document("Vehicle");
    let before = fs::read(&file).unwrap();
    assert_eq!(store.transact(&[], 1048576), Err(Error::Limit));
    assert_eq!(
        store.transact(
            &[Change::Create {
                key: key(26, 27),
                name: "large budget",
                document: &input,
                read_only: false
            }],
            1048577
        ),
        Err(Error::Limit)
    );
    let changes = (0..65)
        .map(|_| Change::Create {
            key: key(26, 27),
            name: "many",
            document: &input,
            read_only: false,
        })
        .collect::<Vec<_>>();
    assert_eq!(store.transact(&changes, 1048576), Err(Error::Limit));
    assert_eq!(fs::read(&file).unwrap(), before);
}
#[test]
fn changed_candidate_format_refuses_reopen_and_existing_handle_writes_without_mutation() {
    let file = path();
    let store = Store::create(&file).unwrap();
    let input = document("Vehicle");
    create(&store, key(28, 29), &input, false);
    // Fixed test-only corruption of an owned candidate; no arbitrary SQL API.
    let setup = "import pathlib,sqlite3,sys; db=sqlite3.connect(pathlib.Path(sys.argv[1]).as_uri()+'?mode=rw',uri=True,isolation_level=None); db.execute('PRAGMA user_version=999'); db.close()";
    assert!(
        std::process::Command::new("/usr/bin/python3")
            .args(["-I", "-S", "-B", "-c", setup])
            .arg(&file)
            .env_clear()
            .status()
            .unwrap()
            .success()
    );
    let after_setup = fs::read(&file).unwrap();
    assert!(matches!(Store::open(&file), Err(Error::Refused(code)) if code == "format"));
    assert_eq!(
        store.transact(
            &[Change::Create {
                key: key(28, 30),
                name: "must not insert",
                document: &input,
                read_only: false
            }],
            1048576
        ),
        Err(Error::Refused("format".into()))
    );
    assert_eq!(fs::read(&file).unwrap(), after_setup);
}

#[test]
fn aggregate_limit_discards_a_large_borrowed_document_batch_before_sql() {
    let file = path();
    let store = Store::create(&file).unwrap();
    let mut value = Document::empty().value().clone();
    value["opaque"] = json!("x".repeat(1_048_000));
    let input = Document::from_json(&value.to_string()).unwrap();
    let before = fs::read(&file).unwrap();
    let changes: Vec<_> = (0u8..64)
        .map(|index| Change::Create {
            key: key(31, index),
            name: "large batch",
            document: &input,
            read_only: false,
        })
        .collect();
    assert_eq!(store.transact(&changes, 1_048_576), Err(Error::Limit));
    assert_eq!(fs::read(&file).unwrap(), before);
    assert!(store.load(key(31, 0)).unwrap().is_none());
    assert!(store.load(key(31, 63)).unwrap().is_none());
}

#[test]
fn complete_64_operation_packet_preserves_escaped_names_and_document_values() {
    let store = Store::create(path()).unwrap();
    let input = document("quote\" slash\\ newline\n unicode🚗");
    let name = "quote\" slash\\ nul\0 newline\n unicode🚗".repeat(40);
    let changes: Vec<_> = (0u8..64)
        .map(|index| Change::Create {
            key: key(32, index),
            name: &name,
            document: &input,
            read_only: false,
        })
        .collect();
    store.transact(&changes, 1_048_576).unwrap();
    for index in [0, 31, 63] {
        let loaded = store.load(key(32, index)).unwrap().unwrap();
        assert_eq!(loaded.name(), name);
        assert_eq!(loaded.document().value(), input.value());
    }
}
