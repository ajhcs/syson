# RR16b: Qualify the complete integration command and input closure after held scopes are admitted

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Adopted completion contract

SYSON-RUST-COMPLETION/1.0.9: [requirements](../COMPLETION-CONTRACT.md), [machine binding](../completion-contract.json), [acceptance packet](../acceptance-packet.schema.json). Primary acceptance family: repairs. Common gates and cross-family caller/dependency obligations apply.

## Source preparation prerequisites

- RR16a
- RR23

Existing PR prerequisites: [#19](https://github.com/ajhcs/syson/pull/19), [#109](https://github.com/ajhcs/syson/pull/109)

## Implementation readiness

- RR16a
- RR23

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- porting/check_acceptance_packets.py
- porting/tests/test_rr16b_acceptance_packets.py
- porting/rust-parity-evidence/RR16b/

## Tasks

- Compose completed repairs on one exact candidate and enumerate every default/feature/target/generator/runtime input and required command. Require explicit owner admission for all held initializer/raw/factory scopes before execution; inventory is never admission. Bind external registry/toolchain/source inputs, generated freshness and all consuming fixtures.
- Run full fresh serialized offline Cargo/WASM/native/browser/fixture matrix and teardown where required; all omissions/held/unrun/stale/failing cases block full integration qualification and RR17 acceptance.
- Consume RR23 joint one-candidate reader evidence as an explicit implementation input; recompose/requalify on the full candidate rather than treating #109 subset results as full integration acceptance.
- Before any release/archive qualification, check EVERY sealed artifact entry against committed Git blobs, including ignored files, or an explicitly declared committed supplement with exact original-head/path/hash binding. Source-input closure alone is not artifact closure. Preserve old packet heads/receipts and refuse unavailable or unbound bytes.
- Implement completion-contract packet enforcement: resolve actual Git candidates and built artifacts, parse receipt outcomes, cover every required obligation/case and reject failed/unrun/unsupported evidence. Structural hashes or accepted labels cannot replace behavioral evidence and independent review. Keep the new checker and regression suite within their named paths; coordinator orders shared runner/CI wiring. Split into reviewed child IDs before exceeding this unit's review budget.

## Verification

- Inject source mutation, omitted crate/fixture, stale generated files and command failure; each must block broad PASS. Qualify a fresh composed run with exact hashes and teardown.
- Freeze exact prerequisite/source/input hashes, preserve original failing evidence, run all affected final-source tests and independent review; no old PASS attribution to changed source.
- Use one serialized offline compiler job and fresh owned runtime/file/DB/browser evidence only where required; no shared production state.
- RR16a selected checks/partial receipt is insufficient. Complete owner-admitted matrix and independent exact closure review are required; current held scopes keep this child OPEN.
- Negative case: a sealed http.log present on disk but ignored by Git must block publication. Positive declared-supplement case must resolve exact bytes and report original-head incompleteness honestly, without replaying the original runtime.
- Reject a non-Git 40-character candidate, stale/substituted artifact, missing obligation or split owner, parsed FAIL/UNRUN, incomplete caller packet and stale independent-review candidate; accept only a complete supported packet for the exact qualification scope.

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
- rereview-batch11-findings.json

Original lane record: [lanes/rereview.json](../lanes/rereview.json). Source snapshots stay immutable; plan.json adds the explicit coordinator dependency overlay.
