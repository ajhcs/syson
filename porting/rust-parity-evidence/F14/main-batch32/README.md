# Finite text import on the repaired Rust foundation

This adopts the complete finite importer from [PR #52](https://github.com/ajhcs/syson/pull/52), head `574193c29659010bd5c31c1e6a768fb58f712230`, with all 13 original tests retained. It stages named packages, part definitions and typed part usages through the actual Adapter, linker, resource registry and caller-owned model transaction. Commit uses the real in-memory Editor and its history; durable storage remains B06 work.

Three new consuming-caller regressions cover ordinary P01 notes and Unicode, opaque marker-shaped objects and exact numeric values through import/history/JSON reopen, and parse/link/cancel failures with existing redo state. Historical source mappings, original fixture failures and passing captures remain verbatim `.raw` data in the donor archive. Native artifact/importer correspondence and full language acceptance remain open.

`examples/stage_commit_probe.rs` measures the actual stage and commit calls separately. Each group creates one package, N definitions and one typed usage. Every repetition restores the same empty document/history, complete deterministic identity stream and cancellation state; source generation, Editor construction and `begin()` occur outside the timers. Two warmups precede seven measured calls, with every raw nanosecond value and min/median/max retained. The probe attempts N=1/16/64/128/253/254/255/1000/5000/10000, checks committed identities/types/history, and records explicit parse/link refusals with `commit:null`. Source-derived boundary expectations are checked after all group outputs; [performance](performance.json) records five committed groups through 253 definitions/510 elements (stage median 34.18 ms, range 34.11–34.54 ms), a linker-limit refusal at 254, and parser-limit refusals at 255/1000/5000/10000. The 5k/10k sources exceed the byte cap, so those fast refusals establish no import capacity.

From the repository root, use an admitted workspace Cargo target directory:

```sh
cargo test --offline --locked --manifest-path rust/crates/syson-text-import/Cargo.toml -- --test-threads=1
cargo run --offline --locked --release --manifest-path rust/crates/syson-text-import/Cargo.toml --example stage_commit_probe
```

The source-bound [qualification](qualification.json) records 16 passing tests, formatting, Clippy and WASM checks against the merged JSON provider. Refusal timing is not successful import capacity; seven API measurements do not establish p95, UI latency or native speedup. Timings inform review without a hard cutoff. The already-landed F25 qualifier is reused without copying it; trusted coordinator inputs and launcher-owned cleanup retain their documented limits. This adoption grants no ownership handoff, C01/C02/P152 acceptance, complete import, database publication, backend restart/reopen or connected UI qualification.
