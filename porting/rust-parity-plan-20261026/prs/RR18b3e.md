# RR18b3e: Bound long navigation labels and preserve visible focus after repaint

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Source preparation prerequisites

- RR18b3c
- RR18b3b

Existing PR prerequisites: [#61](https://github.com/ajhcs/syson/pull/61), [#65](https://github.com/ajhcs/syson/pull/65), [#112](https://github.com/ajhcs/syson/pull/112)

## Implementation readiness

- RR18b3c
- RR18b3b

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- rust/crates/syson-web/src/lib.rs
- rust/crates/syson-web/web/loader.js
- rust/crates/syson-web/web/style.css
- rust/crates/syson-web/tests/rr18b3e_regression.rs
- porting/rust-parity-evidence/RR18b3e/

## Tasks

- Bound the visual extent of long model-navigation names while preserving the complete exact semantic value, accessible name and a usable full-text/detail path. Keep class/kind and selection discoverable; avoid silent truncation, nested scroll traps or a many-screen button.
- Preserve visible focused draft/control after refusal and background/font repaint, not only activeElement and caret equality. Respect deliberate panning/scrolling and avoid unsolicited page jumps. Coordinate loader request ownership/cancellation invariants with the existing presentation authority.

## Verification

- Reproduce the recorded4096-character-name mobile/desktop screenshot wall; verify bounded readable navigation, full text access and exact model bytes through selection/rename/undo/redo.
- After rejected import/background font cancellation, assert focused control viewport intersection, usable caret and native keyboard access at320/1440. Screenshot the actual focused surface before/after, collect AX names/roles and body/inner scroll bounds.
- Retain sparse and512-child fixtures plus all required recovery/cancel/same-ID journeys. Actual screenshots and interaction evidence are required; logical focus alone does not close usability.
- Measure focused-control bounding rectangle and viewport intersection immediately after repaint, before element.screenshot(), click(), focus() or another automation action can scroll it into view and hide the bug.

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
