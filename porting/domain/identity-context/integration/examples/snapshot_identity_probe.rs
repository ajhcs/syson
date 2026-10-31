// SPDX-License-Identifier: EPL-2.0
//! Fresh-process finite import timings; no UI/native or final RSS claim.
use std::{fs, os::unix::fs::DirBuilderExt, path::PathBuf, time::Instant};
use syson_identity_context_integration::{Result, require, sample, workspace_root};
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    require(args.len() == 3, "usage: snapshot_identity_probe CLI ROOT")?;
    let binary = PathBuf::from(&args[1]);
    let root = PathBuf::from(&args[2]);
    workspace_root(&root)?;
    require(
        fs::read_dir(&root)?.next().is_none(),
        "fresh empty probe root required",
    )?;
    let mut samples = Vec::new();
    for repetition in 0..9 {
        let own = root.join(format!("sample-{repetition}"));
        fs::DirBuilder::new().mode(0o700).create(&own)?;
        let started = Instant::now();
        let write = sample(&binary, "write", &own, true)?;
        let read = sample(&binary, "read", &own, true)?;
        let (writer, writer_elapsed) = (write.numbers, write.elapsed_ns);
        let (reader, reader_elapsed) = (read.numbers, read.elapsed_ns);
        require(writer[6..] == [12, 12] && reader[6..] == [12, 12], "identity cardinality")?;
        let whole = started.elapsed().as_nanos();
        require(
            writer[0] != reader[0] && writer[5] == reader[5],
            "fresh reader mismatch",
        )?;
        require(
            writer[4] == 0 && reader[1..4] == [0, 0, 0],
            "phase protocol mismatch",
        )?;
        let row = [
            writer[1],
            writer[2],
            writer[3],
            reader[4],
            writer_elapsed,
            reader_elapsed,
            whole,
        ];
        println!(
            "{{\"workload\":\"finite-import-edit-save-restart\",\"sample\":{repetition},\"warmup\":{},\"writer_pid\":{},\"reader_pid\":{},\"bytes\":{},\"stage_including_parse_ns\":{},\"edit_ns\":{},\"save_ns\":{},\"load_including_parse_ns\":{},\"writer_process_ns\":{writer_elapsed},\"reader_process_ns\":{reader_elapsed},\"whole_pair_ns\":{whole},\"parse_only\":\"UNAVAILABLE_PUBLIC_API\",\"cache\":\"OS-warmth-uncontrolled-fresh-document-directory\",\"RSS\":\"EXTERNAL_SEPARATE\"}}",
            repetition < 2,
            writer[0],
            reader[0],
            writer[5],
            row[0],
            row[1],
            row[2],
            row[3]
        );
        if repetition >= 2 {
            samples.push(row);
        }
    }
    for (index, phase) in [
        "stage_including_parse",
        "edit",
        "save",
        "load_including_parse",
        "writer_process",
        "reader_process",
        "whole_pair",
    ]
    .iter()
    .enumerate()
    {
        let mut values: Vec<_> = samples.iter().map(|row| row[index]).collect();
        values.sort_unstable();
        println!(
            "{{\"phase\":\"{phase}\",\"raw_samples\":7,\"unit\":\"ns\",\"min\":{},\"median\":{},\"max\":{},\"product_performance_claimed\":false}}",
            values[0], values[3], values[6]
        );
    }
    Ok(())
}
