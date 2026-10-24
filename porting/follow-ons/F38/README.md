# F38: immutable move-provider registry

This standalone, dependency-free crate plans dispatch for one provider category:
`SysMLMoveElementService`. It executes no provider code and moves no model objects.
It is based on pinned SysON 2026.9.3 source, not a running Java/Spring comparison.

## Frozen behavior

The [source manifest](source-manifest.json) pins six Git blobs, SHA-256 hashes and
reviewed line ranges at `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`.

- `SysMLMoveElementService.java:42-45` requires both injected delegate list and
  default service to be non-null.
- `SysMLMoveElementService.java:63-66` filters the supplied list by
  `canHandle(element, newParent)` and selects its first match. Caller injection
  order is preserved. The source does not sort, key, or deduplicate that list.
- `SysMLMoveElementService.java:57-60` invokes that delegate, or the default.
  Java `Optional.map` turns a selected null result into an empty Optional, so it
  invokes the default rather than trying the next custom provider. This is a
  source-derived Java semantic expectation; no Java runtime was invoked here.
- `MoveStatus.java:20-29` is a success flag and message record. A non-null failure
  status is returned; failure is not a signal to try another provider.
- Upstream tests at `SysMLMoveElementServiceTest.java:188-248` exercise a matching
  delegate, a non-matching delegate, and bypass of default-only move guards.
  Those tests were inspected, not executed or relabeled as Rust test results.

## Rust API and admission policy

`MoveRegistry::new` owns a required default and ordered provider labels. Labels
are diagnostic only: duplicate labels and a custom label equal to the default
are allowed. Custom slots carry indices, retaining duplicate-list identity.
A missing default produces `MissingDefault`. Empty/whitespace-only labels and
labels exceeding 128 UTF-8 bytes are rejected by a **Rust-only proposed admission
policy**, not by an inferred upstream registration-key contract.

`registry.snapshot(observations)` binds an immutable owned registry to supplied
tri-state capability observations. `snapshot.select()` scans in caller order;
only an observed true matches. An unknown or omitted observation is an error
when reached. An earlier match does not require later observations. Excess
observations are rejected as a Rust packet-shape policy. The upstream Java
implementation computes predicates itself; these packets are a proposed Rust
host boundary, not native support for `Element` or `canHandle`.

`plan.selected()` distinguishes the exact custom slot from the default.
`plan.complete(result)` records a supplied result: custom null requests the
captured default; a non-null status is returned unchanged, including a nullable message (`Option<String>`). A default null is
returned unchanged without recursive fallback. Neither method executes a move,
re-evaluates a predicate, or invokes the default automatically.

Arc-backed immutable registry and observation packets preserve prior snapshots.
The observing caller remains responsible for pairing observations with the
correct element, parent, resource state and request revision. This crate does
not discover Spring beans or claim to reproduce Spring ordering. Predicate
exceptions, null delegate entries, concrete move guards, native identities,
mutation, unpaired UTF-16 message surrogates, frontend extension merging and every other provider category remain
unsupported.

## Focused checks

The suite has 17 test functions, including one exhaustive truth-table test over
511 combinations (zero through eight duplicate-label custom providers).
It covers first-match/default ordering, duplicate slots, a missing default,
Rust label/packet admission boundaries, unknown capabilities before and after a
match, immutable snapshots, selected null fallback, success/failure propagation,
default null behavior and a non-null status with a null message.

Run from this worktree using the existing toolchain and offline cache:

```sh
python3 -B porting/follow-ons/F38/check_sources.py
cargo fmt --manifest-path rust/crates/syson-extensions/Cargo.toml -- --check
cargo test --offline --locked --manifest-path rust/crates/syson-extensions/Cargo.toml -- --test-threads=1
cargo clippy --offline --locked --manifest-path rust/crates/syson-extensions/Cargo.toml --all-targets -- -D warnings
```

Actual commands and their original output belong in the validation packet.
No local pass establishes upstream runtime, Spring integration, whole-area
extension parity, third-party plugin acceptance or production readiness.

A first source-pin check failed because three interface/record line ranges
extended past EOF. Its first output exists only in the tool transcript. The
unchanged diagnostic rerun is preserved in `validation/source-check-01`, with
its actual timestamps, failed manifest and raw output; corrected bounds preserve
all six source blob and SHA-256 pins. This was a metadata failure, not a Rust or
upstream runtime failure.

## Source attribution

The dispatch rules derive from SysON Java sources by Obeo, Copyright (c)
2025, 2026 Obeo, under Eclipse Public License v2.0 (EPL-2.0). The interface
and delegation contributions include Arthur Daussy. Exact per-file source
notices remain in the six pinned upstream objects; no upstream file is changed.
The new Rust crate uses EPL-2.0 and has no external dependencies.
