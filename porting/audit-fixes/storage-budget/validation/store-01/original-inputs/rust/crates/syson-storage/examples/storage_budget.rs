// SPDX-License-Identifier: EPL-2.0
//! Fixed local storage budget probe; the coordinator supplies a fresh owned DB.
use serde_json::json;
use std::{env, fs, path::PathBuf};
use syson_model::Document;
use syson_storage::{Change, Error, Key, Store};

fn main() {
    let args: Vec<_> = env::args_os().collect();
    assert_eq!(
        args.len(),
        2,
        "one fresh absolute database path is required"
    );
    let path = PathBuf::from(&args[1]);
    assert!(path.is_absolute());
    let store = Store::create(&path).expect("initialize fresh coordinator-owned DB");
    let before = fs::read(&path).unwrap();
    let mut value = Document::empty().value().clone();
    value["opaque"] = json!("");
    let overhead = serde_json::to_vec(&value).unwrap().len();
    let document_bytes = 1_048_448;
    value["opaque"] = json!("x".repeat(document_bytes - overhead));
    let document = Document::from_json(&value.to_string()).unwrap();
    assert_eq!(
        serde_json::to_vec(document.value()).unwrap().len(),
        document_bytes
    );
    let changes: Vec<_> = (0u8..64)
        .map(|index| Change::Create {
            key: Key {
                project: [1; 16],
                document: [index; 16],
            },
            name: "aggregate-budget-probe",
            document: &document,
            read_only: false,
        })
        .collect();
    assert_eq!(store.transact(&changes, 1_048_576), Err(Error::Limit));
    assert_eq!(fs::read(&path).unwrap(), before);
    println!(
        "{}",
        json!({"status":"PASS", "operations":64,
        "serialized_document_bytes":document_bytes, "result":"aggregate_limit",
        "database_bytes_unchanged":true})
    );
}
