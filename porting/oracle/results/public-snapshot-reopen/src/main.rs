// SPDX-License-Identifier: EPL-2.0
#![forbid(unsafe_code)]
use serde_json::{Value, json};
use std::{
    path::Path,
    process::ExitCode,
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};
use syson_model::{Document, Editor, NodeIds};
use syson_text_import::Importer;
use syson_workflow::{SnapshotStore, StoreError};

const SOURCE: &str = include_str!("../fixtures/rig.sysml");
const MALFORMED: &str = include_str!("../fixtures/truncated-model.sysml");
const KEY: &str = "rig";
const DEFINITION: &str = "72000000-0000-4000-8000-000000000002";
const NAME: &str = "Link Rev A <&> 雪🧭";

fn ids(sequence: u8) -> NodeIds {
    NodeIds::new(
        &format!("71000000-0000-4000-8000-{sequence:012x}"),
        &format!("72000000-0000-4000-8000-{sequence:012x}"),
    )
    .expect("fixture identity shape")
}

fn detail(operation: &str, error: impl std::fmt::Debug) -> String {
    format!("{operation}: {error:?}")
}

fn store_failure(operation: &str, error: StoreError) -> String {
    if let StoreError::Io(io) = &error {
        return format!("{operation}: {:?}: {io}", io.kind());
    }
    detail(operation, error)
}

fn write(root: &Path, action: &str) -> Result<Value, String> {
    let source = if action == "malformed" {
        MALFORMED
    } else {
        SOURCE
    };
    let name = if action == "overwrite" {
        "Unpublishable alternative"
    } else {
        NAME
    };
    let mut editor = Editor::new(Document::empty());
    let importer = Importer::pinned().map_err(|e| detail("importer", e))?;
    let start = Instant::now();
    let staged = importer
        .stage(
            editor.begin(),
            source,
            None,
            (1..=6).map(ids),
            Arc::new(AtomicBool::new(false)),
        )
        .map_err(|e| detail("stage", e))?;
    let allocated = staged.allocations();
    if allocated != 6
        || !staged
            .commit(&mut editor)
            .map_err(|e| detail("import commit", e))?
    {
        return Err("unexpected import result".into());
    }
    let import_ns = start.elapsed().as_nanos();
    let start = Instant::now();
    let mut amendment = editor.begin();
    amendment
        .rename(DEFINITION, name)
        .map_err(|e| detail("edit", e))?;
    if !editor
        .commit(amendment)
        .map_err(|e| detail("edit commit", e))?
    {
        return Err("edit made no change".into());
    }
    let edit_ns = start.elapsed().as_nanos();
    let start = Instant::now();
    let store = SnapshotStore::open(root).map_err(|e| store_failure("open", e))?;
    let file = store
        .save_new(KEY, editor.document())
        .map_err(|e| store_failure("save", e))?;
    let save_ns = start.elapsed().as_nanos();
    drop(store);
    Ok(json!({
        "mode":"write", "pid":std::process::id(), "allocations":allocated,
        "snapshot":file.file_name().and_then(|name|name.to_str()),
        "document":editor.document().value(),
        "observed_ns":{"import":import_ns,"edit":edit_ns,"open_save":save_ns}
    }))
}

fn read(root: &Path) -> Result<Value, String> {
    let start = Instant::now();
    let store = SnapshotStore::open(root).map_err(|e| store_failure("open", e))?;
    let doc = store.load(KEY).map_err(|e| store_failure("load", e))?;
    let load_ns = start.elapsed().as_nanos();
    drop(store);
    Ok(
        json!({"mode":"read","pid":std::process::id(),"document":doc.value(),"observed_ns":{"open_load":load_ns}}),
    )
}

fn run() -> Result<Value, String> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let [mode, root] = arguments.as_slice() else {
        return Err("expected MODE ROOT".into());
    };
    match mode.to_str() {
        Some(action @ ("write" | "overwrite" | "malformed")) => write(Path::new(root), action),
        Some("read") => read(Path::new(root)),
        _ => Err("unknown mode".into()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(result) => {
            println!("{result}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}", json!({"pid":std::process::id(),"error":error}));
            ExitCode::from(2)
        }
    }
}
