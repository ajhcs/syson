# F31: isolated immutable snapshot publication failures

This change adds deterministic I/O fault tests to the existing Rust local snapshot adapter. The adapter is a development facility, separate from upstream PostgreSQL persistence. The tests exercise the real save sequence, serialization, staged files, hard-link publication and fresh-store reloads in caller-owned workspace directories.

`SnapshotStore::save_new` delegates to one shared `save_new_with_io` implementation. `FsSnapshotIo` performs the original standard-library operations; the injectable `SnapshotIo` seam supplies trusted instrumentation. The key validation, 16 MiB serialized-size bound, `create_new` staging, ordered write/sync/link/directory-sync/remove operations and no-replace publication are preserved. Directory sync remains Unix-only. No manifest, dependency or feature changes are needed.

The source contract is the existing Rust adapter at `ad814c232ee24109639c1d52239cbeec6f4a3e92`, preserved in [source/](source/). This work derives no PostgreSQL, upstream application, power-loss or cross-filesystem guarantees from that source.

## Failure observations

[cases.json](cases.json) enumerates all thirteen tests. Errors before publication leave the destination absent. A partial-write fault retains its exact prefix in staging. Real collisions prove that neither existing staged bytes nor an existing final snapshot are overwritten. The no-fault path proves the shared algorithm loads through a new store and removes staging.

Errors after publication are recorded separately: the hard link may already exist when an operation reports `Io`. The tests cover a successful link followed by an injected error, a directory-sync error and a before-removal error. In those three cases the final file is readable, staging is retained and an immediate retry collides. An additional after-removal error leaves no staging: retry creates a new staged file but still cannot replace the published destination. No cleanup is attempted after an error; an I/O operation may already have completed its side effect. File visibility and a successful process-local reload do not establish crash durability.

`StoreError::Io` remains compatible and carries no publication-state guarantee. Callers must inspect/reconcile the destination before deciding what recovery requires. This PR adds no retry, automatic cleanup, new success acknowledgement or recovery policy. A caller-supplied `SnapshotIo` implementation is trusted to honor the documented filesystem operation semantics; arbitrary implementations can violate them.

## Execution and retention

The runner must set `SYSON_TEST_DIR` to an existing directory on the approved `/workspace` volume. Each test canonicalizes that directory, refuses a root-disk fallback, creates its own unique fixture and retains it. Every fixture checks an unrelated original snapshot byte-for-byte and through a fresh-store load. Each successful case writes `observed.txt`. Libtest suppresses successful `println!` output by default, so the captured stdout does not list fixture paths. [validation/retained-fixtures.json](validation/retained-fixtures.json) inventories the thirteen retained directories, observations and file hashes after the run. No test mutates environment variables or deletes fixture directories.

The first coordinator run (`f31-01`) passed full default workflow formatting, 15 tests (13 new and 2 existing connected tests) and all-target Clippy offline with the existing locked dependencies and one compiler/test thread. No failure attempt occurred. Candidate-feature and non-Unix tests were not run. [VALIDATION.json](VALIDATION.json) records actual command receipts and limitations. Raw compiler streams and source hashes are preserved under `validation/`; failures, if encountered, are retained as separate attempts.

## Scope

The production seam and tests cover one local publication sequence. They do not qualify PostgreSQL recovery, power loss, kernel writeback, hardware failure, multi-process races, unsupported filesystems, Windows behavior or upstream SysON parity. The synthetic after-link error demonstrates an ambiguous outcome contract; it is not evidence that the local hard-link syscall naturally reports an error after success. No whole-area completion claim is made.
