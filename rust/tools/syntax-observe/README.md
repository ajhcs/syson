# Native syntax observation tool (prepared, not compiled)

This Rust-only tool accepts one reviewed UTF-8 `.sysml` or `.kerml` fixture of at most 1024 bytes and emits syntax recognition, token/CST counts and source spans. It never invokes the reference CLI. AST and semantic construction remain `NotPerformed`; syntax agreement cannot establish AST or application parity.

The manifest/lock closure matches the existing syson-syntax lock plus this local package. No new registry package is introduced. No compilation or execution has occurred for this tool. A parent-allocated compiler slot is required before the proposed command:

```sh
CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/workspace/build/builderbob/syson/parser-corpus-native-01/target cargo build --offline --locked --jobs 1 --manifest-path rust/tools/syntax-observe/Cargo.toml
```

After a successful approved build, the exact twelve fixture files from `porting/validation/parser-corpus/manifest.json` may be observed serially with raw output, hashes, UTC, exit status and exact source revision preserved. Do not replay the upstream reference observations. Native observation qualification is still pending.
