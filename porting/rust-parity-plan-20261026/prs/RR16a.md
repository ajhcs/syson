# RR16a: Bind explicitly selected integration checks to immutable candidate source inventories

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Adopted completion contract

SYSON-RUST-COMPLETION/1.0.9: [requirements](../COMPLETION-CONTRACT.md), [machine binding](../completion-contract.json), [acceptance packet](../acceptance-packet.schema.json). Primary acceptance family: repairs. Common gates and cross-family caller/dependency obligations apply.

## Source preparation prerequisites

No new-plan prerequisite.

Existing PR prerequisites: [#19](https://github.com/ajhcs/syson/pull/19)

## Implementation readiness

The unit must compile and pass its local behavior tests against its frozen existing prerequisites.

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- porting/run_integration_checks.py
- porting/tests/test_rr16_integration_receipts.py
- porting/rust-parity-evidence/RR16a/

## Tasks

- Replace fixed lists/partial generic PASS with complete candidate Cargo/input inventory, committed blob+disk before/after binding, explicit selected/held/notselected matrix, exact command scope and immutable failure logs. This foundation NEVER emits broad/full PASS. Bound known Git generator inputs and declared external/toolchain dependencies; unresolved closure remains explicit.

## Verification

- Adversarial owned Git/stub fixtures must catch missing tool/fixture/manifests/locks/localdeps/pins, source drift, stale generation/check failure, dirty/untracked/mode/target mismatch, command failure/timeout and unsafe resume. Run fresh admitted real subset with HELD_NOT_RUN/NOT_SELECTED scopes and full_qualification=false.
- One root-owned serialized offline execution lease; no held initializer-graph/default or raw-factory/factory validator execution. Preserve original counterexamples and independently review exact source/evidence.

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

- rereview-20261026/model-review/findings.json
- rereview-20261026/parser-review/findings.json
- rereview-20261026/platform-review/findings.json
- rereview-20261026/ui-findings.json

Original lane record: [lanes/rereview.json](../lanes/rereview.json). Source snapshots stay immutable; plan.json adds the explicit coordinator dependency overlay.
