# RR18b3d: Compose bounded complete HTML and qualify final responsiveness and glyph usability

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Source preparation prerequisites

- RR18b3a
- RR18b3b
- RR18b3c
- RR18b3e
- RR18b3f

Existing PR prerequisites: [#61](https://github.com/ajhcs/syson/pull/61), [#65](https://github.com/ajhcs/syson/pull/65), [#112](https://github.com/ajhcs/syson/pull/112)

## Implementation readiness

- RR18b3a
- RR18b3b
- RR18b3c
- RR18b3e
- RR18b3f

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- rust/crates/syson-web/src/lib.rs
- rust/crates/syson-web/src/layout_controller.rs
- rust/crates/syson-web/src/bounded_render.rs
- rust/crates/syson-web/web/loader.js
- rust/crates/syson-web/web/style.css
- rust/crates/syson-web/tests/rr18b3_regression.rs
- porting/rust-parity-evidence/RR18b3/
- rust/crates/syson-web/web/fonts/
- rust/crates/syson-web/src/layout_geometry.rs

## Tasks

- Replace full-render-then-length-check and independently allocating helpers with bounded HTML/escaping/model/geometry sinks or counted admission before materialization. Cover navigation/options/SVG/scalar inspectors/export and all fallback paths without weakening atomic import recovery.
- Preserve unrelated unsubmitted form drafts across recovery-shell removal and restoration using bounded presentation-only loader storage, with focus/caret restored. RR18a preserves drafts on normal refused-import DOM only; its fallback shell lacks prior form fields.
- Compose exact RR18b1/RR18b2 with complete bounded HTML/helper emission and actual performance qualification. Any request-transfer/chunk scheduler change needs explicit request ownership, cancellation and stale callback gates before mutation. Partial sibling success never closes this child or parent.
- Use the recorded phase baseline to address the whole browser task, including native text/DOM/layout work, rather than treating Rust dispatch alone as latency. Changes to large JSON/rename field wrapping or local scrolling must preserve accessible text, exact whitespace and keyboard navigation, with bounded body overflow. Source-only phase inference is insufficient.
- Replace the inherited user-facing canvas refusal jargon (file/ABI byte budget) with an actionable explanation of the reached limit, unchanged model data and currently available safe export/recovery. Do not suggest unsupported view-selection features. Preserve typed admission guards and separately verify user-facing copy.
- Verify Unicode glyph rendering on the pinned actual browser/native font baseline: the batch10 mobile screenshot renders a CJK draft glyph as tofu although exact DOM-string assertions pass. Provide adequate application font fallback or explicitly record an unmet visual gate; byte preservation alone cannot close visual Unicode usability.
- Adopt bounded scalar/helper sinks in complete navigation/options/SVG/export/model/geometry paths and fallback; compose exact children on one landing candidate. Never merge snapshot branches. Split further before budget exhaustion.
- Audit remaining owned Document.elements() consumers before finite caller limits, specifically layout_geometry::measured before its32-node refusal and canvas import/selection. Adopt shared borrowed views or pre-copy admission where appropriate; renderer-only RR18b3a cannot close these amplification paths.
- Retain dense ownership UI performance as a separate workload: initial isolated A observed204/185ms and a+b joint165/178ms at320/1440 on the192234-byte/1025-row fixture. Profile remaining owned-summary callers and DOM scale; smaller phase observations cannot replace this evidence.

## Verification

- Use under-1 MiB dense/deep/escaped inputs that expand beyond 16 MiB. Verify bounded rejection and retained prior state/drafts through actual WASM UI at desktop/mobile widths, keyboard access, console and long-task observation.
- Freeze exact prerequisite/source/input hashes, preserve original failing evidence, run all affected final-source tests and independent review; no old PASS attribution to changed source.
- Use one serialized offline compiler job and fresh owned runtime/file/DB/browser evidence only where required; no shared production state.
- Prove pre-copy refusal/high-water allocation bounds, escaped exact-fit and +1 boundaries and independently observed actual browser responsiveness. RR18a lifecycle-only success cannot close this child.
- Actual WASM fallback -> recover must retain permitted prior form drafts/caret, bound stored draft bytes, and make explicit discard clear model and presentation drafts with keyboard confirmation.
- Use the exact RR18a browser fixtures:700258-byte escaped import and900381-byte committed export/2701179-byte URI. Current same-host Chromium observation has longtasks up to111ms at320px and71ms at1440px. Rendering responsiveness remains OPEN. On the pinned owned reference profile, target no main-thread task >=50ms in these journeys through pre-admission/counting/chunking; record phase timing and any unmet budget explicitly, and do not claim performance acceptance while this required budget fails. Physical mobile/other-browser acceptance remains separate.
- Retain both instrumented and ordinary browser observations. The batch10 baseline reaches111/69ms whole tasks while measured Rust dispatch peaks40.8/43ms and full-journey ABI byte transfer maxima8.0/10.3ms at320/1440 (expanding-import-only7.7/7.9ms); wrapper overhead and unmeasured native work prevent exact causal attribution. Run trace/layout evidence before choosing a fix; required final whole-task budget remains unchanged.

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
