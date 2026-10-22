# Domain lane validation checkpoint

Baseline `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`; branch `rust/domain`.
Host BuilderBob; cwd `/workspace/worktrees/builderbob/syson/domain`.
Rust `1.99.1 (58cf8f9c3 2026-10-13)`; Cargo `1.99.1`.
No fetched crate dependencies and no build scripts.
All build outputs: `/workspace/build/builderbob/syson/domain`.
`CARGO_BUILD_JOBS=1` used for each Cargo invocation.

* `cargo fmt --manifest-path rust/crates/syson-domain/Cargo.toml`: passed.
* `cargo test --offline --manifest-path rust/crates/syson-domain/Cargo.toml`:
  20 integration tests passed, 0 failed; no unit/doc tests.
* `cargo clippy --offline --manifest-path rust/crates/syson-domain/Cargo.toml --all-targets -- -D warnings`:
  passed after applying array-chunk style fixes.

Initial test iteration: 18 passed, one inventory assertion failed because the
expected keyword count was wrong (134 versus pinned actual 117). Corrected to
117. Subsequent suite includes 20 tests and passes. First Clippy run rejected
three array-chunk style warnings; code updated and rerun passed.

Tests include 4 exact upstream printable-quote examples and 7 exact UUID goldens
from upstream NameHelperTest and ElementUtilTest, plus independent Python stdlib
uuid.uuid5 Unicode and multiblock fixtures. Other cases are source-derived tests.
Java, UI, API, persistence, full metamodel, resource/proxy behavior and full
application parity remain unrun/unported as recorded in README.md and UNPORTED.md.
Independent source review raised the distinction between raw EMF containment and
the bounded semantic projection; the crate docs now state the adapter invariant
explicitly. It must not be represented as lossless raw EMF ingestion.
