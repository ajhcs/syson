// SPDX-License-Identifier: EPL-2.0
//! Differential-oracle entry point: read SysML text on stdin, import it into an
//! empty document with the Rust importer, and print one JSON line:
//! {"status":"ok","document":<EMF JSON>} or {"status":"error","error":"..."}.
use serde_json::{Value, json};
use std::{
    io::Read,
    sync::{Arc, atomic::AtomicBool},
};
use syson_model::{Document, Editor, NodeIds};
use syson_text_import::Importer;

fn identities(n: u32) -> NodeIds {
    NodeIds::new(
        &format!("10000000-0000-4000-8000-{n:012x}"),
        &format!("20000000-0000-4000-8000-{n:012x}"),
    )
    .unwrap()
}

fn run(source: &str) -> Result<Value, String> {
    let importer = Importer::pinned().map_err(|e| format!("{e:?}"))?;
    let mut editor = Editor::new(Document::empty());
    let staged = importer
        .stage(
            editor.begin(),
            source,
            None,
            (1u32..).map(identities),
            Arc::new(AtomicBool::new(false)),
        )
        .map_err(|e| format!("{e:?}"))?;
    staged.commit(&mut editor).map_err(|e| format!("{e:?}"))?;
    let text = editor.document().to_json().map_err(|e| format!("{e:?}"))?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn main() {
    let mut source = String::new();
    std::io::stdin().read_to_string(&mut source).unwrap();
    let out = match run(&source) {
        Ok(document) => json!({"status": "ok", "document": document}),
        Err(error) => json!({"status": "error", "error": error}),
    };
    println!("{out}");
}
