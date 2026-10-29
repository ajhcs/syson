# RR24a: Add explicit literal duplicate-key compatibility without weakening strict callers

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Adopted completion contract

SYSON-RUST-COMPLETION/1.0.3: [requirements](../COMPLETION-CONTRACT.md), [machine binding](../completion-contract.json), [acceptance packet](../acceptance-packet.schema.json). Primary acceptance family: repairs. Common gates and cross-family caller/dependency obligations apply.

## Source preparation prerequisites

- RR01

Existing PR prerequisites: [#90](https://github.com/ajhcs/syson/pull/90)

## Implementation readiness

- RR01

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- rust/crates/syson-json-value/src/lib.rs
- rust/crates/syson-json-value/tests/rr24a_duplicates.rs
- porting/rust-parity-evidence/RR24a/
- rust/crates/syson-json-value/README.md

## Tasks

- Keep parse strict by default and add an explicit shared LastWins policy for migration callers. Preserve first decoded-key insertion position and final value, raw bytes, original number lexemes and ordinary marker-named objects. Charge every physical occurrence before collapse; never add a second parser.

## Verification

- Run all strict RR01 regressions plus manual ordered/literal oracles, escaped/nested duplicate keys, overwritten malformed data, exact physical byte/depth/value/key/scalar budgets and cancellation. Verify the actual migration adopter uses only the explicit opt-in policy.
- Freeze exact source, preserve original failed attempts, independently review, and bind executed inputs to committed blobs. A new compatibility API is not a claim that the old strict policy was a bug.

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
