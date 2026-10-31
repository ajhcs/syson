# RR18b: Bound HTML preview and intermediate rendering before materialization

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Adopted completion contract

SYSON-RUST-COMPLETION/1.0.11: [requirements](../COMPLETION-CONTRACT.md), [machine binding](../completion-contract.json), [acceptance packet](../acceptance-packet.schema.json). Primary acceptance family: repairs. Common gates and cross-family caller/dependency obligations apply.

## Source preparation prerequisites

- RR18b1
- RR18b2
- RR18b3

Existing PR prerequisites: [#65](https://github.com/ajhcs/syson/pull/65), [#61](https://github.com/ajhcs/syson/pull/61)

## Implementation readiness

- RR18b1
- RR18b2
- RR18b3

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** - RR18b1
- RR18b2
- RR18b3
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- porting/rust-parity-evidence/RR18b/

## Tasks

- Replace full-render-then-length-check and independently allocating helpers with bounded HTML/escaping/model/geometry sinks or counted admission before materialization. Cover navigation/options/SVG/scalar inspectors/export and all fallback paths without weakening atomic import recovery.
- Preserve unrelated unsubmitted form drafts across recovery-shell removal and restoration using bounded presentation-only loader storage, with focus/caret restored. RR18a preserves drafts on normal refused-import DOM only; its fallback shell lacks prior form fields.
- Integrate all three children on one exact candidate. Original allocation, draft isolation, recovery, actual UI and <50 ms performance gates remain required. [TIMING POLICY SUPERSEDED 2026-10-28: Measure workload-specific latency ranges and outliers under performance-policy.json; report distinct event-to-visible latency, main-thread task durations and allocation metrics. Preserve raw timing evidence; no single50ms veto or automatic parity acceptance. Keep actual responsiveness, cancellation and exact-data checks.]

## Verification

- Use under-1 MiB dense/deep/escaped inputs that expand beyond 16 MiB. Verify bounded rejection and retained prior state/drafts through actual WASM UI at desktop/mobile widths, keyboard access, console and long-task observation.
- Freeze exact prerequisite/source/input hashes, preserve original failing evidence, run all affected final-source tests and independent review; no old PASS attribution to changed source.
- Use one serialized offline compiler job and fresh owned runtime/file/DB/browser evidence only where required; no shared production state.
- Prove pre-copy refusal/high-water allocation bounds, escaped exact-fit and +1 boundaries and independently observed actual browser responsiveness. RR18a lifecycle-only success cannot close this child.
- Actual WASM fallback -> recover must retain permitted prior form drafts/caret, bound stored draft bytes, and make explicit discard clear model and presentation drafts with keyboard confirmation.
- Use the exact RR18a browser fixtures:700258-byte escaped import and900381-byte committed export/2701179-byte URI. Current same-host Chromium observation has longtasks up to111ms at320px and71ms at1440px. Rendering responsiveness remains OPEN. On the pinned owned reference profile, target no main-thread task >=50ms in these journeys through pre-admission/counting/chunking; record phase timing and any unmet budget explicitly, and do not claim performance acceptance while this required budget fails. Physical mobile/other-browser acceptance remains separate. [TIMING POLICY SUPERSEDED 2026-10-28: Measure workload-specific latency ranges and outliers under performance-policy.json; report distinct event-to-visible latency, main-thread task durations and allocation metrics. Preserve raw timing evidence; no single50ms veto or automatic parity acceptance. Keep actual responsiveness, cancellation and exact-data checks.]

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
