# RR08b: Journal AST mutations and publish atomically

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Adopted completion contract

SYSON-RUST-COMPLETION/1.0.7: [requirements](../COMPLETION-CONTRACT.md), [machine binding](../completion-contract.json), [acceptance packet](../acceptance-packet.schema.json). Primary acceptance family: repairs. Common gates and cross-family caller/dependency obligations apply.

## Source preparation prerequisites

- RR08a

Existing PR prerequisites: [#11](https://github.com/ajhcs/syson/pull/11), [#41](https://github.com/ajhcs/syson/pull/41)

## Implementation readiness

- RR08a

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- rust/crates/syson-ast/src/lib.rs
- rust/crates/syson-ast/tests/rr08_regression.rs
- porting/rust-parity-evidence/RR08b/

## Tasks

- Replace snapshots with complete nested suffix undo, outer commit clearing and cancellation/work admission before publication. Add recording across allocation, frames, append_datatype, adopt_subrule, assign, defaults, merge, action, associate_cst and end_frame together.
- Remove State::Clone only when every transactional mutator records its effects. Preserve spent work/IDs, owner identity, frames, fields, kind and CST on nested failure and cancellation.
- Charge each repeated containment-property name before finish copies it. Restore the complete original nine-case regression file and qualify the combined repair through the parent gates.

## Verification

- Run all nine original RR08 regressions and all current 27 adapter/37 AST cases, for64expected distinct final cases. Preserve original failures and exact original full regression source.
- Run actual 100/1000/5000core and F12 operator workloads separately, including nested rollback, deterministic cancellation, foreign/CST/spent-ID, partial defaults/kind adoption and finish-label refusal.
- Freshly qualify complete combined finite parsing/structural behavior and relevant release timing/memory observations under the declared method.
- Verify final combined kernel bytes equal the reviewed final donor 0a3b7c3a... unless a separately reviewed demonstrable repair is needed. Any difference requires its own source/evidence disposition.
- Freeze exact staged source/base/input hashes and recompute actual added-plus-deleted production and authored counts after ordinary formatting before execution; split again if either cap is exceeded.

Current timing policy: [workload-specific target ranges](../performance-policy.json) supersede legacy timing cutoffs; original observations remain historical.

## Commit plan

- Complete journal/atomic-publication and finish-label changes with the remaining preserved regressions and coordinator wiring
- Fresh complete-composition checks/profiles/independent review, source seals and explicit acceptance limits

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
