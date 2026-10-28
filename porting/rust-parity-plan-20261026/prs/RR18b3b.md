# RR18b3b: Reduce native textarea layout while preserving literal text and local scrolling

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Source preparation prerequisites

- RR18b1
- RR18b2

Existing PR prerequisites: [#61](https://github.com/ajhcs/syson/pull/61), [#65](https://github.com/ajhcs/syson/pull/65), [#112](https://github.com/ajhcs/syson/pull/112)

## Implementation readiness

- RR18b1
- RR18b2

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- rust/crates/syson-web/web/style.css
- porting/rust-parity-evidence/RR18b3b/

## Tasks

- Use recorded real Chromium Layout traces to scope whitespace-preserving local scrolling to machine JSON and rename textareas. Keep literal newlines/tabs, caret, keyboard reachability, accessible names/values and bounded body width. No visibility/AX shortcuts or input scheduler.
- Keep CJK glyph coverage explicitly OPEN until a real adequate font baseline/asset is qualified; naming unavailable fonts does not fix it. Scalar textarea changes require their own observed large-text mechanism.

## Verification

- Compare current and final actual320/1440 traces and ordinary journeys using exact700258 import/900381 export/2701179URI fixtures. Verify exact text values, local scroll, keyboard Home/End/selection/backward caret, labels/readonly AX and screenshots; collect console and whole-task observations.
- Rerun affected recovery/boundary/cancellation/long-ID and canvas journeys on exact final CSS and reused qualified WASM, plus exact a+b composition. Keep whole-task<50ms acceptance OPEN if any required scenario fails. [TIMING POLICY SUPERSEDED 2026-10-28: Measure workload-specific latency ranges and outliers under performance-policy.json; report distinct event-to-visible latency, main-thread task durations and allocation metrics. Preserve raw timing evidence; no single50ms veto or automatic parity acceptance. Keep actual responsiveness, cancellation and exact-data checks.]

Current timing policy: [workload-specific target ranges](../performance-policy.json) supersede legacy timing cutoffs; original observations remain historical.

## Commit plan

- Focused source implementation and meaningful regression/caller fixtures
- Independently reviewed exact verification evidence
- Data-only addition of exact sealed HTTP logs omitted by Git ignore; preserve earlier local commit and verify every artifact against Git

## Review budget

- **production lines max:** 400
- **authored lines max:** 900
- **expected commits:** 3
- **split rule:** Split into numbered behavior-family children before exceeding either limit; generated/evidence data is a separate review block.

## Completion

All named counterexamples/regressions and actual consuming caller gates pass on exact reviewed source; unsupported or unrun acceptance remains OPEN.

## Source evidence

- rereview-20261026/model-review/findings.json
- rereview-20261026/parser-review/findings.json
- rereview-20261026/platform-review/findings.json
- rereview-20261026/ui-findings.json

Original lane record: [lanes/rereview.json](../lanes/rereview.json). Source snapshots stay immutable; plan.json adds the explicit coordinator dependency overlay.
