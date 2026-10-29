# RR16c: Publish exact omitted batch10 HTTP logs as a bound archival supplement

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Adopted completion contract

SYSON-RUST-COMPLETION/1.0.5: [requirements](../COMPLETION-CONTRACT.md), [machine binding](../completion-contract.json), [acceptance packet](../acceptance-packet.schema.json). Primary acceptance family: repairs. Common gates and cross-family caller/dependency obligations apply.

## Source preparation prerequisites

No new-plan prerequisite.

Existing PR prerequisites: [#113](https://github.com/ajhcs/syson/pull/113), [#114](https://github.com/ajhcs/syson/pull/114)

## Implementation readiness

The unit must compile and pass its local behavior tests against its frozen existing prerequisites.

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- porting/rust-parity-artifact-supplements/batch10-20261027/

## Tasks

- Recover the four and twelve omitted HTTP logs from retained RR18b1/RR18b2 packets only when their bytes match the historical committed seal. Publish a separate supplement with original PR/head/path/hash, copied location and honest later-publication provenance; preserve original heads and dependent bases.
- Keep every log as a .log payload with no redaction, reconstruction, newline conversion or .gitignore change. Force-add only the manifest-listed logs after strict static-asset GET/privacy checks; original directories remain read-only.

## Verification

- Verify every manifest mapping against SHA256.json loaded from the exact old Git head, read retained original bytes and check all16 hashes. Inventory exact output paths, plain regular file modes and byte identity; no product execution or test replay.
- Independent source/data review, synthetic/static-only HTTP payload audit, complete committed artifact inventory including ignored files, pushed exact head and dependency-free main landing-tree review are required.
- Original feature heads still lack these blobs directly. A declared committed supplement resolves archival availability; do not label the old heads self-contained or infer feature/runtime acceptance.

Current timing policy: [workload-specific target ranges](../performance-policy.json) supersede legacy timing cutoffs; original observations remain historical.

## Commit plan

- Focused implementation and meaningful regression/caller fixtures
- Exact verification evidence, independent review and remaining limits

## Review budget

- **production lines max:** 400
- **authored lines max:** 900
- **expected commits:** 2
- **split rule:** Split into numbered behavior-family children before exceeding either limit; generated/evidence data is a separate review block.

## Completion

All named counterexamples/regressions and actual consuming caller gates pass on exact reviewed source; unsupported or unrun acceptance remains OPEN.

## Source evidence

- rereview-batch11-findings.json

Original lane record: [lanes/rereview.json](../lanes/rereview.json). Source snapshots stay immutable; plan.json adds the explicit coordinator dependency overlay.
