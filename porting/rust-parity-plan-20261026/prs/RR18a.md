# RR18a: Preserve active editing state on refused preview and keep recovery reachable

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Adopted completion contract

SYSON-RUST-COMPLETION/1.0.5: [requirements](../COMPLETION-CONTRACT.md), [machine binding](../completion-contract.json), [acceptance packet](../acceptance-packet.schema.json). Primary acceptance family: repairs. Common gates and cross-family caller/dependency obligations apply.

## Source preparation prerequisites

- RR04

Existing PR prerequisites: [#65](https://github.com/ajhcs/syson/pull/65), [#61](https://github.com/ajhcs/syson/pull/61)

## Implementation readiness

- RR04

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- rust/crates/syson-web/src/lib.rs
- rust/crates/syson-web/src/layout_controller.rs
- rust/crates/syson-web/tests/rr18_regression.rs
- porting/rust-parity-evidence/RR18a/

## Coordinator-owned wiring

- **paths:** - rust/crates/syson-web/web/loader.js
- **policy:** Root-only same-focused-control caret restoration on draft-preserving repaint, after actual baseline reproduction. Author does not edit loader. Generic recovery-shell draft storage remains RR18b.

## Tasks

- Admit imported document preview on a private candidate before replacing active Editor/history/canvas/selection. Keep export, recover and explicit discard reachable when preview is refused. Use bounded RR04 model output and avoid serialization when canvas frame is absent. Full HTML before-allocation rendering remains RR18b.

## Verification

- Reproduce actual <=1MiB escaped-name destructive-import counterexample on exact baseline, then demonstrate prior document/history/selection/drafts survive refusal in real default WASM UI at desktop/mobile with keyboard/console/long-task observations. Verify fallback model download and recover/discard actions.
- Freeze exact prerequisite/source/input hashes, preserve failing attempts and final-source tests with independent review; no held raw/graph/factory execution.

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
