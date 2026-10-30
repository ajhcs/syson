# RR08a: Admit selected AST copies before reservation

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Adopted completion contract

SYSON-RUST-COMPLETION/1.0.10: [requirements](../COMPLETION-CONTRACT.md), [machine binding](../completion-contract.json), [acceptance packet](../acceptance-packet.schema.json). Primary acceptance family: repairs. Common gates and cross-family caller/dependency obligations apply.

## Source preparation prerequisites

No new-plan prerequisite.

Existing PR prerequisites: [#11](https://github.com/ajhcs/syson/pull/11), [#41](https://github.com/ajhcs/syson/pull/41)

## Implementation readiness

The unit must compile and pass its local behavior tests against its frozen existing prerequisites.

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- rust/crates/syson-ast/src/lib.rs
- rust/crates/syson-ast/tests/rr08_regression.rs
- porting/rust-parity-evidence/RR08a/

## Tasks

- Charge copied_text/copied_datum, selected kind allocation and reference/merge values before reservation; poll UTF-8-safe scalar chunks. Keep State::Clone and the existing whole-State snapshot transaction.
- Preserve source-first merged arrays, target scalar precedence and target array lexeme. All source fields are copied and charged before shadowed entries are discarded; do not claim only retained fields are copied.
- Do not introduce Undo, remove snapshots, rewrite assign/default/datatype transactions or implement finish-label charging in this child.

## Verification

- Reproduce the known/UnknownType preallocation and reservation-cancellation counterexamples against the admitted base, then run the two actual preserved RR08 cases.
- Run all existing AST and current 27 adapter cases on exact new sources, with all source/case counts verified.
- Verify source-first merge order, target lexeme, foreign refusal and rollback under the unchanged snapshot transaction.
- Keep whole-State growth, large merge preallocation, full nested journal rollback and repeated finish labels explicitly OPEN for the next scope. Do not present selected helper admission as the whole RR08 repair.
- Freeze exact staged source/base/input hashes and recompute actual added-plus-deleted production and authored counts after ordinary formatting before execution; split again if either cap is exceeded.

Current timing policy: [workload-specific target ranges](../performance-policy.json) supersede legacy timing cutoffs; original observations remain historical.

## Commit plan

- Selected copied-value operation changes plus preserved meaningful regression subset and permitted coordinator expectation update
- Exact source/counterexample/verification evidence, independent review and remaining parent gaps

## Review budget

- **production lines max:** 400
- **authored lines max:** 900
- **expected commits:** 2
- **split rule:** Split into numbered behavior-family children before exceeding either limit; generated/evidence data is a separate review block.

## Completion

This bounded source and its preserved original cases pass independent qualification on exact admitted inputs. Whole-parent obligations remain required; no full feature acceptance follows from a child PR.

## Source evidence

- rereview-20261026/model-review/findings.json
- rereview-20261026/parser-review/findings.json
- rereview-20261026/platform-review/findings.json
- rereview-20261026/ui-findings.json
- amendments/1.0.2-rr08-split.md

Original lane record: [lanes/rereview.json](../lanes/rereview.json). Source snapshots stay immutable; plan.json adds the explicit coordinator dependency overlay.
