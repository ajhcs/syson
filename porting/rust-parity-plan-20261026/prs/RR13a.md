# RR13a: Adopt the complete regular SnapshotStore and SnapshotIo API

Lane: storage-source. State: PLANNED_SOURCE_PREPARATION_NOT_FEATURE_ACCEPTED.

## Adopted completion contract

SYSON-RUST-COMPLETION/1.0.11: [requirements](../COMPLETION-CONTRACT.md), [machine binding](../completion-contract.json), [acceptance packet](../acceptance-packet.schema.json). Primary acceptance family: repairs. Common gates and cross-family caller/dependency obligations apply.

## Source preparation prerequisites

- RR04

Existing PR prerequisites: [#99](https://github.com/ajhcs/syson/pull/99)

## Implementation readiness

- RR04

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- rust/crates/syson-workflow/src/lib.rs
- rust/crates/syson-workflow/src/snapshot_io.rs
- rust/crates/syson-workflow/tests/rr13_regression.rs
- rust/crates/syson-workflow/tests/connected.rs
- porting/rust-parity-evidence/RR13a/

## Tasks

- Adopt complete regular SnapshotStore, StoreError, SnapshotIo and FsSnapshotIo APIs. Preserve immutable save_new/save_new_with_io/load, trusted operation ordering, no-replace hard-link publication, retained stages and may-have-published Io errors.
- Keep all 341 projected lib lines, including 111 lines of three deterministic races, all 86 SnapshotIo lines, the 332-line eight-case RR13 regression file and the 103-line two-case connected file. Archive the whole 348-line donor and defer only its exact lines 17–23 held raw-factory-candidate initialized_creation declarations; no held API or feature is released.
- Use a regular-only 14-line manifest with existing model/editor/persistence dependencies. Keep held optional initializer/metamodel dependencies and raw feature declaration archived; do not introduce substitutes, native FFI, Python storage or a database facade.

## Verification

- Source review must prove current model to_json_bounded/from_json/elements/projection and transaction APIs plus editor rotation caller compatibility. Recount every formatted source/helper/fixture before execution; planned 337 production/896 authored leaves only four authored lines.
- Root runs all 13 retained cases under verified Linux x86_64 builderbob/Samsung ownership: regular fresh-handle reopen, no-overwrite, symlink/ancestor/root policies, directory replacement, FIFO/socket/UTF8, exact 4 MiB/one-extra refusal, provisional oversized output before I/O and three deterministic admission/open races. Preserve prior FIFO timeout and all historical failures.
- Use offline locked serial fmt/tests/all-target Clippy and applicable target checks. Reserve seven CI lines for the syson-workflow matrix entry, SYSON_TEST_DIR and a conditional ephemeral GitHub Ubuntu setup: sudo install an owned 0700 /workspace/syson-snapshots directory. This reviewed hosted-only setup changes no local administration or test/API guards; distinguish hosted filesystem behavior from local Samsung measurements. Recount actual formatting/repairs/wiring before execution and split before either hard cap is exceeded. Bound child lifetimes, retain cleanup proof and obtain independent source/executed-input review before exact-head publication.

Current timing policy: [workload-specific target ranges](../performance-policy.json) supersede legacy timing cutoffs; original observations remain historical.

## Commit plan

- Complete regular API and all 13 original cases
- Current source-bound qualification and preserved donor/failure evidence

## Review budget

- **production lines max:** 400
- **authored lines max:** 900
- **planned production:** 337
- **planned authored:** 896

## Completion

Complete regular API and all 13 named cases pass on current safe model with actual owned filesystem operations. RR13 still requires RR13b and the full26-case composition; stable owned parents remain required. No hostile-writer, arbitrary-device-race, descriptor-relative openat or physical power-loss guarantee follows; SQL, native, HTTP and UI acceptance stay open.

## Source evidence

- rereview-20261026/model-review/findings.json
- rereview-20261026/platform-review/findings.json

Original lane record: [lanes/storage-source.json](../lanes/storage-source.json). Source snapshots stay immutable; plan.json adds the explicit coordinator dependency overlay.
