// SPDX-License-Identifier: EPL-2.0
//! Repeatable pure planner benchmark; no filesystem, database or native runtime.
use std::{hint::black_box, time::Instant};
use syson_document_reconcile::{Document, DocumentPlan, plan_documents};

fn main() {
    for count in [1_000usize, 5_000, 10_000] {
        let documents: Vec<_> = (0..count)
            .map(|index| {
                let mut id = [0u8; 16];
                id[..8].copy_from_slice(&(index as u64).to_le_bytes());
                Document {
                    id,
                    name: format!("document-{index}"),
                    content: "{}".to_owned(),
                    is_read_only: false,
                    created_on: (),
                    last_modified_on: (),
                }
            })
            .collect();
        let references: Vec<_> = documents.iter().collect();
        let run = || {
            let plan = plan_documents(black_box(&references), black_box(&references), &[]);
            assert!(matches!(black_box(plan), DocumentPlan::Unchanged));
        };
        run();
        let mut nanos = Vec::with_capacity(5);
        for _ in 0..5 {
            let started = Instant::now();
            run();
            nanos.push(started.elapsed().as_nanos());
        }
        nanos.sort_unstable();
        println!(
            "{{\"documents\":{count},\"samples\":5,\"min_ns\":{},\"median_ns\":{},\"max_ns\":{},\"outcome\":\"unchanged\"}}",
            nanos[0], nanos[2], nanos[4]
        );
    }
}
