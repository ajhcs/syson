# RR18b3c: Bound scalar and escaped HTML helpers before materialization

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Adopted completion contract

SYSON-RUST-COMPLETION/1.0.10: [requirements](../COMPLETION-CONTRACT.md), [machine binding](../completion-contract.json), [acceptance packet](../acceptance-packet.schema.json). Primary acceptance family: repairs. Common gates and cross-family caller/dependency obligations apply.

## Source preparation prerequisites

- RR18b3a

Existing PR prerequisites: [#61](https://github.com/ajhcs/syson/pull/61), [#65](https://github.com/ajhcs/syson/pull/65), [#112](https://github.com/ajhcs/syson/pull/112)

## Implementation readiness

- RR18b3a

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** - RR18b3c2
- RR18b3c3
- **joint implementation inputs:** - RR18b3d
- **expanded implementation inputs:** - I01
- RR01
- RR02
- RR04
- RR18a
- RR18b1
- RR18b2
- RR18b3a
- RR18b3b
- RR18b3c
- RR18b3c2
- RR18b3c3
- RR18b3d
- RR18b3e
- RR18b3f

## Write scope

- rust/crates/syson-web/src/bounded_render.rs
- rust/crates/syson-web/tests/rr18b3c_regression.rs
- rust/crates/syson-web/tests/support/rr18b3c_alloc.rs
- porting/rust-parity-evidence/RR18b3c/

## Coordinator-owned wiring

- **paths:** - rust/crates/syson-web/src/lib.rs
- rust/crates/syson-web/src/bounded_render.rs
- **policy:** Root-only module hook, fallible try_render/preview/navigation/paint caller wiring and test compatibility adapters after explicit source freeze; no parallel writer. Current source is exact a+b snapshot, not actual main or RR20 adoption. Existing Result alias visibility is the only coordinator module change beyond formatting; author acknowledges exact freeze.

## Tasks

- Implement counted/limited escaping and scalar-helper emission through the existing model/literal authority; avoid independently allocating oversized escaped inputs, selected scalar JSON and readonly inspectors before admission. Preserve exact accepted wire/HTML bytes.
- Hand off the bounded sink contract to complete renderer composition; do not claim navigation/SVG/geometry/global HTML or latency closure from helper-only tests.
- Keep actual two-field helper-limit caller qualification in separate verification child RR18b3c2; helper producer acceptance requires that child. No source/helper/global gate is removed by this review-size split.

## Verification

- Use under-1 MiB dense/deep/escaped inputs that expand beyond 16 MiB. Verify bounded rejection and retained prior state/drafts through actual WASM UI at desktop/mobile widths, keyboard access, console and long-task observation.
- Freeze exact prerequisite/source/input hashes, preserve original failing evidence, run all affected final-source tests and independent review; no old PASS attribution to changed source.
- Use one serialized offline compiler job and fresh owned runtime/file/DB/browser evidence only where required; no shared production state.
- Prove pre-copy refusal/high-water allocation bounds, escaped exact-fit and +1 boundaries and independently observed actual browser responsiveness. RR18a lifecycle-only success cannot close this child.
- Distinguish helper-component admission from global renderer closure: the admitted 700258-byte,700000-ampersand Package expands across navigation/options/SVG/export beyond16MiB while import selection is empty. Retain actual UI refusal/model/history/draft evidence, but whole-page preallocation and responsiveness remain RR18b3d requirements; a<=1MiB scalar alone is not falsely labelled >16MiB helper output.
- Use exact frozen legacy bytes plus independent UTF8/CR/NUL/sentinel and raw-number/private-marker-object oracles. Count whole helper wrappers, exact-fit/+1, individually fitting fields whose aggregate refuses, and swallowed-write-error fault injection. Measure successful allocation-request aggregate/largest separately from RSS/live/high-water; disclose owned inspection and the added document-size scan.

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
