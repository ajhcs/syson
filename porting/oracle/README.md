# Differential oracle (Phase 0)

Runs the same SysML text through the reference SysON and the Rust importer and
diffs the resulting EMF JSON.

Reference: the official SysON `v2026.9.3` release jar (sha256 `85c03135…`, built
from upstream `b380dada`), Java 21, PostgreSQL 15.19 extracted from the official
image. It is not built from the pinned `6ba2ac64` source; the 125 differing paths
are recorded in `porting/audits/20261026/native-reference.json`. Building from
source needs a GitHub token with `read:packages` (Phase 1).

## Use

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
(source, both documents, diff) plus `summary.json` and `SUMMARY.md`.

## What a verdict means

| Verdict | Meaning |
|---|---|
| MATCH | Same containment tree, classes, attributes and resolved references after normalization |
| DIFFER | Both imported; the first structural differences are listed |
| RUST_REFUSES | Reference imported (possibly with warnings); Rust returned an error |
| REFERENCE_ONLY_REFUSES | Rust imported; the reference failed |
| BOTH_REFUSE | Neither imported |

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
- `corpus.json`: inline synthetic cases plus upstream SysON `.sysml` test inputs
- `results/`: committed summaries of selected runs
- `rust/crates/syson-text-import/examples/oracle_import.rs`: stdin SysML → EMF JSON via the Rust importer
