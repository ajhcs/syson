# Differential oracle (Phase 0)

Runs the same SysML text through the reference SysON and the Rust importer and
diffs the resulting EMF JSON.

Reference: the official SysON `v2026.9.3` release jar (sha256 `85c03135…`, built
from upstream `b380dada`), Java 21, PostgreSQL 15.19 extracted from the official
image. It is not built from the pinned `6ba2ac64` source; the 125 differing paths
are recorded in `porting/audits/20261026/native-reference.json`. Building from
source needs a GitHub token with `read:packages` (Phase 1).

## Use

For a new Linux environment, follow the [native reference setup](native-setup.md)
to select your own runtime paths and run directory.

```bash
python3 -I porting/oracle/oracle.py up
```

```bash
cargo build --offline --locked --example oracle_import --manifest-path rust/crates/syson-text-import/Cargo.toml --target-dir /workspace/build/builderbob/syson-oracle/target
```

```bash
python3 -I porting/oracle/differential.py
```

```bash
python3 -I porting/oracle/oracle.py down
```

`up` starts a fresh database and SysON on random loopback ports, records the
processes in `/workspace/build/builderbob/syson-oracle/current.json`, and returns
once the app answers (about 12 s). Nothing is registered as a service, and `down`
stops only the processes `up` started. Logs, the database directory and every
differential run stay under `/workspace/build/builderbob/syson-oracle/runs/`.

`differential.py --only <text>` runs a subset. Each run writes one JSON per case
(source, both documents, diff) plus `summary.json` and `SUMMARY.md`. The reference
app is restarted every 15 cases (`--restart-every`), because one JVM runs out of
heap after a few dozen imported projects.

## Replay without a running reference

Build the Rust importer from the repository root, then replay the committed
reference results with Python 3. The default binary location is the text-import
crate's Cargo target directory.

~~~bash
cargo build --locked --example oracle_import --manifest-path rust/crates/syson-text-import/Cargo.toml
python3 porting/oracle/differential.py --golden
~~~

Set the same CARGO_TARGET_DIR for both commands when using an alternate build
directory, or pass --rust-bin with the exact binary path. Use --only to select
cases and --out to select the report directory. Golden mode reads the selected
reference records in porting/oracle/golden and runs the same comparator as live
mode. A missing, stale, corrupt or differently pinned record stops the run
before comparison.

To refresh the reference records, start the pinned release as described above
and record the complete corpus into a fresh directory. Set CARGO_TARGET_DIR for
both runs, or use the same explicit --rust-bin path to compare the exact same
Rust executable.

~~~bash
python3 porting/oracle/differential.py --record --golden-dir /path/to/fresh-golden
python3 porting/oracle/oracle.py down
python3 porting/oracle/differential.py --golden --golden-dir /path/to/fresh-golden
~~~

Review the per-case live and replay verdicts before replacing committed records.
Each record contains the reference document, its normalized tree, the import
report and digests for the served release and the case text sent as UTF-8.
Protocol errors and harness failures never count as recorded reference outcomes.
If disk I/O interrupts output, discard the incomplete draft directory and
record again at a fresh path.

## What a verdict means

| Verdict | Meaning |
|---|---|
| MATCH | Same containment tree, classes, attributes and resolved references after normalization |
| DIFFER | Both imported; the first structural differences are listed |
| RUST_REFUSES | Reference imported (possibly with warnings); Rust returned an error |
| REFERENCE_ONLY_REFUSES | Rust imported; the reference failed |
| BOTH_REFUSE | Neither imported |
| HARNESS_ERROR | The reference app or the Rust binary failed (crash, timeout, unreadable response); not a verdict, rerun the case |

Normalization replaces generated `id`/`elementId` values with positional labels
and resolves references to whichever node they point at. Two wire-format
differences are reported separately instead of being hidden: the reference wraps
every import in a root `sysml:Namespace` with an `org.eclipse.syson.sysml.imported`
annotation, and its cross-references point at the target's `id`, where Rust's
point at `elementId`.

## Files

- `oracle.py`: start/stop/status for the isolated reference app
- `reference.py`: GraphQL client (create blank project, multipart `uploadDocument`, project download, delete)
- `compare.py`: normalization and diff
- `differential.py`: corpus runner and report
- `corpus.json`: inline synthetic cases (including one minimal repro per language construct, whose IDs the [Phase 1 issues](https://github.com/ajhcs/syson/issues/269) use) plus upstream SysON `.sysml` test inputs
- `results/`: committed summaries of selected runs
- `rust/crates/syson-text-import/examples/oracle_import.rs`: stdin SysML → EMF JSON via the Rust importer
