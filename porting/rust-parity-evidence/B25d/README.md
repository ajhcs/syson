# B25d evidence and reproduction

`native-fixtures.json` contains 19 raw input cases. `source-predictions.json` remains source-derived prediction data; observed results are separate under `observations/`. No prediction file is consumed by either observation harness.

The native harness compiles the exact pinned SysON participant and candidate inherited Sirius interface/service with annotation processing disabled and Java target 21. Seven release JAR hashes and tool hashes are recorded in `observations/native/review.json`; exact historical compile/runtime arguments and class hashes are in `observations/native/receipt.json`. The unused owned editing context satisfies the real service constructor; no application context or database is started.

To repeat the native selected comparison, obtain the seven identified dependencies, verify their hashes, choose Java21-compatible tools and a fresh output directory, then run the following with absolute paths. `EVIDENCE` points to this directory; `JAR_DIR` contains only the identified JARs. These are adaptation instructions, not a claim that a portable rerun was executed.

```sh
javac --release 21 -proc:none -cp "$JAR_DIR/*" -d "$OUT/classes" "$EVIDENCE/observations/native/ActionOracle.java" "$EVIDENCE/observations/native/IRepresentationMigrationParticipant.java" "$EVIDENCE/observations/native/RepresentationMigrationService.java" "$EVIDENCE/observations/native/StartDoneAndControlNodeMigrationParticipant.java"
java -Xmx256m -XX:ActiveProcessorCount=2 -cp "$OUT/classes:$JAR_DIR/*" ActionOracle "$EVIDENCE/native-fixtures.json"
```

The original Rust observation runner is retained at `observations/rust/executed-runner.py`, together with its consumed `action_observe.rs`, manifest, lock and complete observed source. It uses original BuilderBob absolute paths and creates a fresh output directory. For another checkout, copy `main.rs` into a separate scratch Cargo project's `src/main.rs`, copy the manifest/lock, adjust only the manifest's `syson-migrations` path to the checked-out crate, and run `cargo run --offline --locked --manifest-path <scratch/Cargo.toml> -- <absolute/native-fixtures.json>`. Offline execution requires the exact locked packages to be present. Source inspection/metadata alone does not satisfy this runtime step.

Compare result arrays by ID: version, kind and status must agree; successful rows must also match JSON values and `changed`. Two refusal rows preserve `native_partial_tree` only on Java's side; Rust intentionally returns no partially prepared value. Do not compare native exception text/classes with Rust typed errors. The retained `comparison.json` is the coordinator's actual result.

`captures/b25d-01/` contains the exact unit runner, receipt and raw streams. Its upstream-runtime field applies to that local test invocation; the separate actual native/Rust observations establish the selected comparison. Complete external raw artifacts remain under `/workspace/build/builderbob/syson-rust-parity-batch3-20261026`.

Candidate preparation/output limits are 1 MiB, depth 64 and 65536 JSON values; supplied selected watermarks are at most 4096 bytes. These finite Rust policies are not native capacity qualification. No full artifact/source closure, migration-framework history, real read-only editing-context enforcement, SQL, GUI, persistence or broad performance claim is made.
