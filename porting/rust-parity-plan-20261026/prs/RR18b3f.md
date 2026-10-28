# RR18b3f: Keep saved scalar and readonly value displays readable with exact full-text access

Lane: rereview-repairs. State: IMPLEMENTED_BOUNDED_NOT_ACCEPTED.

## Adopted completion contract

SYSON-RUST-COMPLETION/1.0.0: [requirements](../COMPLETION-CONTRACT.md), [machine binding](../completion-contract.json), [acceptance packet](../acceptance-packet.schema.json). Primary acceptance family: repairs. Common gates and cross-family caller/dependency obligations apply.

## Source preparation prerequisites

- RR18b3c
- RR18b3c3

Existing PR prerequisites: [#115](https://github.com/ajhcs/syson/pull/115), [#116](https://github.com/ajhcs/syson/pull/116)

## Implementation readiness

- RR18b3c
- RR18b3c3

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** - RR18b3e
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
- RR18b3e

## Write scope

- rust/crates/syson-web/src/bounded_render.rs
- rust/crates/syson-web/tests/rr18b3f_regression.rs
- porting/rust-parity-evidence/RR18b3f/

## Tasks

- Replace unbounded visible saved-state/readonly value paragraphs with a bounded readable summary and usable native full-value disclosure/control. Preserve exact saved JSON/literal semantics, accessible names, keyboard/full-text/copy access and unchanged model/export bytes. Do not silently truncate values or only hide focus offscreen.
- Keep ordinary small values clear; do not build oversized previews before admission or defeat helper limits. Full renderer composition RR18b3d must adopt the final helper contract after this UX repair. No CSS-file author scope is granted; any shared styling requires an explicit coordinator boundary before source edits.

## Verification

- Reproduce actual850kComment body saved-value wall from batch12 helper-scalar320 screenshot; measure bounding rectangles/page scroll extent before automation scroll, then verify bounded initial display, native disclosure/full-text access at320/1440, selection/history/CRLF/Unicode/readonly/AX and exact downloaded model bytes.
- Use actual screenshots and UI interaction/performance observations; retain current missing glyph and global<50ms/renderer admission gates. Requalify exact affected scalar helper oracles on changed source, preserving legacy golden captures as historical data rather than relabelling them. [TIMING POLICY SUPERSEDED 2026-10-28: Measure workload-specific latency ranges and outliers under performance-policy.json; report distinct event-to-visible latency, main-thread task durations and allocation metrics. Preserve raw timing evidence; no single50ms veto or automatic parity acceptance. Keep actual responsiveness, cancellation and exact-data checks.]
- At 320 and 1440 CSS pixels, the initial saved-state display must occupy at most six text lines and 160 CSS pixels per property, without horizontal page overflow. Full-value access must have a labelled keyboard path and a selectable native control with at most twelve visible rows and local scrolling; exact saved JSON and model downloads remain authoritative. Any required broader interaction or lazy-output protocol must be split before exceeding the source budget.
- Retain batch13 actual wide-value counterexamples: compactJSON128-byte body126W and124-byte readonlyalias120W originally occupied7/8lines. Verify the80-byte summary threshold and exact keyboard full-value disclosure on those same inputs at320/1440; preserve originalfailure and unchangedcanonicalmodel.

Current timing policy: [workload-specific target ranges](../performance-policy.json) supersede legacy timing cutoffs; original observations remain historical.

## Commit plan

- Focused saved-value presentation and meaningful boundary fixtures
- Exact browser/screenshots/performance/independent review and limits

## Review budget

- **production lines max:** 400
- **authored lines max:** 900
- **expected commits:** 2
- **split rule:** Split before exceeding budget; full model/renderer/host presentation rewrites require separate behavior families.

## Completion

Bounded readable presentation and exact full-value/model access pass actual keyboard/AX/viewport/caller checks; full parity and glyph/latency closure stay separate.

## Source evidence

- rereview-batch12-findings.json
- rereview-batch13-findings.json

Original lane record: [lanes/rereview.json](../lanes/rereview.json). Source snapshots stay immutable; plan.json adds the explicit coordinator dependency overlay.
