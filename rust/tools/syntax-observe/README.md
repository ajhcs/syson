# Native syntax observation tool (bounded execution captured)

This Rust-only tool accepts one reviewed UTF-8 `.sysml` or `.kerml` fixture of at most 1024 bytes and emits syntax recognition, token/CST counts and source spans. It never invokes the reference CLI. AST and semantic construction remain `NotPerformed`; syntax agreement cannot establish AST or application parity.

The manifest/lock closure matches the existing syson-syntax lock plus this local package. No new registry package is introduced. The tool was built offline once and observed twelve fixtures under a bounded parent allocation; exact receipts are in `porting/runs/parser-corpus-rust-01`. That allocation is released. A new parent allocation is required before another build:

```sh
CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/workspace/build/builderbob/syson/parser-corpus-native-01/target cargo build --offline --locked --jobs 1 --manifest-path rust/tools/syntax-observe/Cargo.toml
```

The exact twelve fixture files from `porting/validation/parser-corpus/manifest.json` were observed serially with raw output, hashes, UTC, exit status and exact source revision preserved. Nine recognized syntax and three rejected syntax; all twelve observation processes exited0. Distinct independent qualification remains pending. Do not replay unchanged runs or the upstream reference observations.
