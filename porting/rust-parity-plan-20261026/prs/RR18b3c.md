# RR18b3c: Bound scalar and escaped HTML helpers before materialization

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Source preparation prerequisites

- RR18b3a

Existing PR prerequisites: [#61](https://github.com/ajhcs/syson/pull/61), [#65](https://github.com/ajhcs/syson/pull/65), [#112](https://github.com/ajhcs/syson/pull/112)

## Implementation readiness

- RR18b3a

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- rust/crates/syson-web/src/lib.rs
- rust/crates/syson-web/src/bounded_render.rs
- rust/crates/syson-web/tests/rr18b3c_regression.rs
- porting/rust-parity-evidence/RR18b3c/

## Tasks

- Implement counted/limited escaping and scalar-helper emission through the existing model/literal authority; avoid independently allocating oversized escaped inputs, selected scalar JSON and readonly inspectors before admission. Preserve exact accepted wire/HTML bytes.
- Hand off the bounded sink contract to complete renderer composition; do not claim navigation/SVG/geometry/global HTML or latency closure from helper-only tests.

## Verification

- Use under-1 MiB dense/deep/escaped inputs that expand beyond 16 MiB. Verify bounded rejection and retained prior state/drafts through actual WASM UI at desktop/mobile widths, keyboard access, console and long-task observation.
- Freeze exact prerequisite/source/input hashes, preserve original failing evidence, run all affected final-source tests and independent review; no old PASS attribution to changed source.
- Use one serialized offline compiler job and fresh owned runtime/file/DB/browser evidence only where required; no shared production state.
- Prove pre-copy refusal/high-water allocation bounds, escaped exact-fit and +1 boundaries and independently observed actual browser responsiveness. RR18a lifecycle-only success cannot close this child.

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
