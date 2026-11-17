# Default syntax admission for the shipped Syside libraries

[Issue #247](https://github.com/ajhcs/syson/issues/247) tracks chart-state
scaling. The 94 library files in the Syside 0.9.0 extension give that work
realistic inputs across both grammars. They contain 1,338,935 bytes of model
text. The pinned embedded Syside parser admitted every file with zero lexer
and parser errors.

| Recognizer | Accepted | Chart-state budget refusals |
|---|---:|---:|
| Rust baseline `05a4be35d4842272d0249ab43b9e49295591ef1e` | 71 | 23 |
| [PR #290](https://github.com/ajhcs/syson/pull/290), `e9325a5534c88f0d462e2bacc489c655e2db07ac` | 77 | 17 |

Every recorded Rust refusal hit the existing default of 250,000 chart states.
The smallest remaining inputs include `Actions.sysml` at 15,388 bytes,
`KerML.kerml` at 20,711 bytes, and `ShapeItems.sysml` at 23,802 bytes.
`ISQAtomicNuclear.sysml` at 151,892 bytes gives a larger case. The
[inventory](baseline.tsv) records every member's byte count, SHA-256, and
admission status.

## Reproduce the admission sweep

Use the retained or downloaded
[Syside 0.9.0 extension asset](https://github.com/sensmetry/sysml-2ls/releases/download/0.9.0/sensmetry.sysml-2ls-0.9.0.vsix).
The script verifies the archive, member list and per-file hashes. Point
`SYSON_BUILD_DIR` at the existing workspace build directory for this run.

```sh
cargo build --release --offline --locked --jobs 1 \
  --target-dir "${SYSON_BUILD_DIR}/library-admission" \
  --manifest-path rust/validation/syntax/Cargo.toml --example library_admission

python3 -I porting/syntax/library-admission/check.py \
  --vsix "${SYSIDE_VSIX}" \
  --rust-probe "${SYSON_BUILD_DIR}/library-admission/release/examples/library_admission" \
  > "${SYSON_BUILD_DIR}/library-admission.jsonl"
```

The final row gives accepted and refusal totals and the supplied probe's
SHA-256. The rows above it identify each case, its historical outcomes, and
the current Rust diagnostic. A malformed probe response stops the run without
a summary.

The reviewed Linux x86-64 release probes had these SHA-256 values.

| Probe at the Rust revision above | SHA-256 |
|---|---|
| Rust baseline | `7c15a1f794ef0f65b6ce4ae048684b817d22456e6de78fe0d79028a9e1f6d93c` |
| PR #290 | `b6c8cf20ce0434cec53367f157680c867bd84fa2ec790a7500c573e2afee3216` |

`--expect rust_main` or `--expect rust_290` compares the supplied probe's
outcomes with the matching historical column. Build from the corresponding
Rust syntax revision to reproduce a historical run. A matching outcome
column does not establish the source revision of a supplied executable.
Omit `--expect` to measure a new recognizer. A scaling repair should admit
more files without introducing a new refusal. Verify selected CSTs, error
behavior, time and memory separately.

## Replay the historical reference

The official SysON v2026.9.3 release jar from upstream `b380dada` has SHA-256
`85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1`.
Its embedded `syside-cli.js` reports 0.9.0 and has SHA-256
`47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd`.
The embedded script bytes match the fork's pinned upstream source file at
`6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. That identity covers the script
only. The extension supplies the model inputs and the release jar supplies
the parser implementation.

Add `--reference-jar "${SYSON_REFERENCE_JAR}" --node "${NODE_BINARY}"`
and `--launcher "${SYSON_BUILD_DIR}/syside-admission-launcher.js"` to
replay the reference with Node 24. The launcher path must be vacant. The
runner hashes the jar and embedded script, then replaces only the final CLI
entry invocation in its local copy to read the underlying parser's lexer and
parser errors before document loading. The observed gate is the absence
of both types of error. CST, AST, linking, recovery, semantic validation
and SysON model import need their own checks.
