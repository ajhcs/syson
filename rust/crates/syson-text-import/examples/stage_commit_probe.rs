// SPDX-License-Identifier: EPL-2.0
//! Actual finite importer calls; setup/begin are outside the stage/commit timers.
use serde_json::{Value, json};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};
use syson_ast_adapter::Error as ParseError;
use syson_linker::Error as LinkError;
use syson_model::{Document, Editor, NodeIds};
use syson_text_import::{Error, Importer};
fn identities(n: u32) -> NodeIds {
    NodeIds::new(
        &format!("10000000-0000-4000-8000-{n:012x}"),
        &format!("20000000-0000-4000-8000-{n:012x}"),
    )
    .unwrap()
}
fn samples(raw: &[u128]) -> Value {
    let mut sorted = raw.to_vec();
    sorted.sort_unstable();
    json!({"raw_ns":raw,"min_ns":sorted[0],"median_ns":sorted[3],"max_ns":sorted[6]})
}
fn main() {
    let importer = Importer::pinned().unwrap();
    let mut boundary_failures = 0;
    for definitions in [1u32, 16, 64, 128, 253, 254, 255, 1_000, 5_000, 10_000] {
        let declarations: String = (0..definitions)
            .map(|n| format!("part def T{n};"))
            .collect();
        let source = format!("package P {{ {declarations} part u:T0; }}");
        let baseline = Document::empty();
        let expected = match definitions {
            1 | 16 | 64 | 128 | 253 => "committed",
            254 => "link_limit",
            _ => "parse_limit",
        };
        let mut outcome = None;
        let mut stage_ns = Vec::new();
        let mut commit_ns = Vec::new();
        for repetition in 0..9 {
            let mut editor = Editor::new(baseline.clone());
            let history = editor.history_state();
            let transaction = editor.begin();
            let cancel = Arc::new(AtomicBool::new(false));
            let started = Instant::now();
            let result =
                importer.stage(transaction, &source, None, (1u32..).map(identities), cancel);
            let elapsed = started.elapsed().as_nanos();
            assert_eq!(editor.document(), &baseline);
            assert_eq!(editor.history_state(), history);
            let (observed, committed_ns) = match result {
                Ok(staged) => {
                    assert_eq!(staged.roots().len(), 1);
                    assert_eq!(staged.allocations(), (2 * definitions + 4) as usize);
                    let started = Instant::now();
                    assert!(staged.commit(&mut editor).unwrap());
                    let committed = started.elapsed().as_nanos();
                    assert_eq!(
                        editor.document().elements().len(),
                        (2 * definitions + 4) as usize
                    );
                    let usage = identities(2 * definitions + 2).element;
                    let target = identities(2).element;
                    assert_eq!(
                        editor.document().element(&usage).unwrap()["data"]["ownedRelationship"][0]
                            ["data"]["type"],
                        target
                    );
                    assert_eq!(editor.history_state().undo_entries, 1);
                    assert_eq!(editor.history_state().redo_entries, 0);
                    ("committed", Some(committed))
                }
                Err(Error::Parse(ParseError::Limit)) => ("parse_limit", None),
                Err(Error::Link(LinkError::Limit)) => ("link_limit", None),
                Err(error) => panic!("unexpected refusal at {definitions}: {error:?}"),
            };
            if let Some(prior) = outcome {
                assert_eq!(prior, observed);
            }
            outcome = Some(observed);
            boundary_failures += usize::from(observed != expected);
            if repetition >= 2 {
                stage_ns.push(elapsed);
                if let Some(elapsed) = committed_ns {
                    commit_ns.push(elapsed);
                }
            }
        }
        assert_eq!(stage_ns.len(), 7);
        let commit = (!commit_ns.is_empty()).then(|| {
            assert_eq!(commit_ns.len(), 7);
            samples(&commit_ns)
        });
        println!(
            "{}",
            json!({"definitions":definitions,"declarations":definitions+2,
            "source_bytes":source.len(),"warmups":2,"samples":7,"outcome":outcome.unwrap(),
            "expected_outcome":expected,"stage":samples(&stage_ns),"commit":commit})
        );
    }
    assert_eq!(
        boundary_failures, 0,
        "retain all output: source-derived boundary differed"
    );
}
