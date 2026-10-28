# RR23c: Preserve literal JSON at native-http-contract observed-contract boundary

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Source preparation prerequisites

- RR01
- C01f
- RR24a

Existing PR prerequisites: [#83](https://github.com/ajhcs/syson/pull/83), [#90](https://github.com/ajhcs/syson/pull/90), [#100](https://github.com/ajhcs/syson/pull/100)

## Implementation readiness

- RR01
- C01f
- RR24a

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- rust/tools/native-http-contract/src/lib.rs
- rust/tools/native-http-contract/tests/rr23c_literal.rs
- porting/rust-parity-evidence/RR23c/

## Coordinator-owned wiring

- **paths:** - rust/tools/native-http-contract/Cargo.toml
- rust/tools/native-http-contract/Cargo.lock
- **policy:** Root-only local dependency and existing registry tuple lock wiring; source author does not edit manifests/locks.

## Tasks

- Adopt RR01 at the native-http-contract parse boundary using one shared literal authority. Preserve existing captures, typed shape and comparison policies; reject malformed/private-marker-shaped objects in scalar slots without token magic. Physical depth/value/key/scalar admission must be explicit and tested. Use qualified RR24a LastWins to preserve generic-parser duplicate compatibility, charging all physical occurrences before collapse.

## Verification

- Use independent raw/manual-object expectations for both marker keys, numeric extremes and strings at every reader. Recheck the actual captured corpus, typed shapes, budgets and refusal behavior.
- Freeze exact prerequisite/source/input hashes, preserve original failing evidence, run all affected final-source tests and independent review; no old PASS attribution to changed source.
- Use one serialized offline compiler job and fresh owned runtime/file/DB/browser evidence only where required; no shared production state.

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
