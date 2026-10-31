# RR13b: Qualify the complete SnapshotIo fault contract and filesystem ranges

Lane: storage-source. State: PLANNED_NOT_IMPLEMENTED.

## Adopted completion contract

SYSON-RUST-COMPLETION/1.0.11: [requirements](../COMPLETION-CONTRACT.md), [machine binding](../completion-contract.json), [acceptance packet](../acceptance-packet.schema.json). Primary acceptance family: repairs. Common gates and cross-family caller/dependency obligations apply.

## Source preparation prerequisites

- RR13a

Existing PR prerequisites: [#99](https://github.com/ajhcs/syson/pull/99)

## Implementation readiness

- RR13a

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- rust/crates/syson-workflow/tests/snapshot_failures.rs
- rust/crates/syson-workflow/examples/snapshot_roundtrip_probe.rs
- porting/rust-parity-evidence/RR13b/

## Tasks

- Adopt the entire 356-line, 13-case snapshot_failures.rs unchanged. Check each create/write/file-sync/publish/directory-sync/remove operation, partial writes, errors after publication/removal, collision/no-overwrite and retained prior/staged bytes through actual SnapshotIo.
- Add a bounded actual release filesystem probe: fresh identical admitted document/IDs/history and owned directory each repetition; save then drop/reopen/load; verify full data and role identities. Keep exact/over 4 MiB refusal controls and add applicable owned read-only/permission-denial controls with modes restored and successful prior bytes checked. Those new controls are separate from the original 26 cases; count any probe/helper growth normally. Keep compilation outside timed work.
- Report save/load separately with two warmups and seven raw samples per named representative workload, raw ranges and independently scoped process memory. State cache warmth and measured operations; no hard timing veto, p95/native speedup/capacity claim or inferred crash durability.

## Verification

- Root reexecutes all 26 original cases on the final RR13a+RR13b source, not just13fault cases; preserve typed may-have-published errors, exact original bytes and operation sequences. An Io error alone never proves nonpublication or safe retry.
- Independent reviewers inspect actual binary correctness assertions, raw save/load samples, limits and whole-process memory method; root owns serial compiler/filesystem subprocess scopes, source-before/after custody, cleanup, hosted checks and exact-head merge.

Current timing policy: [workload-specific target ranges](../performance-policy.json) supersede legacy timing cutoffs; original observations remain historical.

## Commit plan

- Complete original13fault cases and bounded actual filesystem probe
- Source-bound26case execution and raw performance/recovery evidence

## Review budget

- **production lines max:** 400
- **authored lines max:** 900
- **planned production:** 83
- **planned authored:** 453

## Completion

All26 original cases and actual filesystem save/load probe pass on the exact reviewed composition; failure visibility and recovery evidence remain explicit. Parent native/SQL/HTTP/UI and physical power-loss acceptance remain required.

## Source evidence

- rust/crates/syson-workflow/src/snapshot_io.rs
- rereview-20261026/platform-review/findings.json

Original lane record: [lanes/storage-source.json](../lanes/storage-source.json). Source snapshots stay immutable; plan.json adds the explicit coordinator dependency overlay.
