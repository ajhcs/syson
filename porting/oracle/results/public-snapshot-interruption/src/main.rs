// SPDX-License-Identifier: EPL-2.0
#![forbid(unsafe_code)]

mod barrier;

use barrier::{BARRIERS, Observation};
use serde_json::{Value, json};
use std::{
    path::Path,
    process::ExitCode,
    sync::{Arc, atomic::AtomicBool},
};
use syson_model::{Document, Editor, NodeIds};
use syson_text_import::Importer;
use syson_workflow::{SnapshotStore, StoreError};

const SOURCE: &str = include_str!("../fixtures/rig.sysml");
const KEY: &str = "rig";
const ORIGINAL: &str = "original";
const DEFINITION: &str = "72000000-0000-4000-8000-000000000002";

fn ids(sequence: u8) -> NodeIds {
    NodeIds::new(
        &format!("71000000-0000-4000-8000-{sequence:012x}"),
        &format!("72000000-0000-4000-8000-{sequence:012x}"),
    )
    .expect("fixture identity shape")
}

fn fixture(name: &str) -> Result<Document, Value> {
    let mut editor = Editor::new(Document::empty());
    let importer = Importer::pinned().map_err(|e| failure("importer", e))?;
    let staged = importer
        .stage(
            editor.begin(),
            SOURCE,
            None,
            (1..=6).map(ids),
            Arc::new(AtomicBool::new(false)),
        )
        .map_err(|e| failure("stage", e))?;
    if staged.allocations() != 6
        || !staged
            .commit(&mut editor)
            .map_err(|e| failure("import commit", e))?
    {
        return Err(failure("import", "expected six committed identities"));
    }
    let mut amendment = editor.begin();
    amendment
        .rename(DEFINITION, name)
        .map_err(|e| failure("edit", e))?;
    if !editor
        .commit(amendment)
        .map_err(|e| failure("edit commit", e))?
    {
        return Err(failure("edit", "rename made no change"));
    }
    Ok(editor.document().clone())
}

fn failure(operation: &str, error: impl std::fmt::Debug) -> Value {
    json!({"operation":operation, "error":format!("{error:?}")})
}

fn store_failure(operation: &str, error: StoreError) -> Value {
    let mut row = failure(operation, &error);
    if let StoreError::Io(io) = error {
        row["io_kind"] = json!(format!("{:?}", io.kind()));
        row["raw_os_error"] = json!(io.raw_os_error());
    }
    row
}

fn save(store: &SnapshotStore, mode: &str, barrier: Option<&str>) -> Result<Value, Value> {
    let (key, name) = match mode {
        "retry" => (KEY, "Unpublishable alternative"),
        "seed" => (ORIGINAL, "Baseline sentinel"),
        _ => (KEY, "Link Rev A <&> 雪🧭"),
    };
    let document = fixture(name)?;
    let mut observer = Observation {
        barrier,
        ..Observation::default()
    };
    let result = match mode {
        "interrupt" | "observe" | "retry" => store.save_new_with_io(key, &document, &mut observer),
        _ => store.save_new(key, &document),
    };
    let report = match result {
        Ok(path) => {
            json!({"document":document.value(), "snapshot":path.file_name().and_then(|name| name.to_str())})
        }
        Err(error) => store_failure("save", error),
    };
    let mut report = report;
    report["attempted"] = json!(observer.attempted);
    report["completed"] = json!(observer.completed);
    if report.get("error").is_some() {
        Err(report)
    } else {
        Ok(report)
    }
}

fn run() -> Result<Value, Value> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if let [mode, root, point] = args.as_slice()
        && mode == "interrupt"
    {
        let point = point
            .to_str()
            .filter(|p| BARRIERS.contains(p))
            .ok_or_else(|| failure("arguments", "unknown boundary"))?;
        let store = SnapshotStore::open(Path::new(root)).map_err(|e| store_failure("open", e))?;
        return save(&store, "interrupt", Some(point));
    }
    let [mode, root] = args.as_slice() else {
        return Err(failure("arguments", "expected MODE ROOT [BOUNDARY]"));
    };
    let store = SnapshotStore::open(Path::new(root)).map_err(|e| store_failure("open", e))?;
    match mode.to_str() {
        Some(mode @ ("seed" | "observe" | "retry")) => save(&store, mode, None),
        Some(mode @ ("read" | "read-original")) => {
            let key = if mode == "read" { KEY } else { ORIGINAL };
            let document = store.load(key).map_err(|e| store_failure("load", e))?;
            Ok(json!({"document":document.value()}))
        }
        _ => Err(failure("arguments", "unknown mode")),
    }
}

fn main() -> ExitCode {
    let (mut report, ok) = match run() {
        Ok(report) => (report, true),
        Err(report) => (report, false),
    };
    report["pid"] = json!(std::process::id());
    if ok {
        println!("{report}");
        ExitCode::SUCCESS
    } else {
        eprintln!("{report}");
        ExitCode::from(2)
    }
}
